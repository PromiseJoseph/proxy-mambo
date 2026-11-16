use crate::error::ProxyError;
use crate::request;
use crate::response::send_error_response;
use crate::types::{Client, Config, ErrorResponse};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn relay(
    mut client_stream: TcpStream,
    client: &Client,
    config: Arc<Config>,
) -> Result<(), Error> {
    // Connect to the upstream server

    let mut request_buffer = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];

    let max_header_size = config.get_max_header_size();

    // Read the request headers from the client stream until we find the end of the headers or exceed the maximum header size.
    let header_end_pos = loop {
        let bytes_read = match client_stream.read(&mut chunk).await {
            Ok(bytes) => bytes,
            Err(e) => {
                send_request_error(&mut client_stream, ProxyError::BadRequest(None), None).await?;
                return Err(Error::new(ErrorKind::Other, "Failed to read from client."));
            }
        };

        if bytes_read == 0 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Client closed before sending complete HTTP headers",
            ));
        }

        request_buffer.extend_from_slice(&chunk[..bytes_read]);

        if let Some(end) = find_header_end(&request_buffer) {
            if end > max_header_size {
                send_request_error(
                    &mut client_stream,
                    ProxyError::RequestHeaderFieldsTooLarge(None),
                    None,
                )
                .await?;

                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "HTTP headers are too large",
                ));
            }
            break end;
        } //to be addressed as a http response with status code 431 Request Header Fields Too Large..

        if request_buffer.len() > max_header_size {
            tracing::error!(
                "HTTP headers exceeded maximum size: {} bytes",
                request_buffer.len()
            );
            send_request_error(
                &mut client_stream,
                ProxyError::RequestHeaderFieldsTooLarge(None),
                None,
            )
            .await?;
            return Err(Error::new(
                ErrorKind::InvalidData,
                "HTTP headers are too large",
            ));
        }
    };

    // Modify only the HTTP headers.
    let (updated_headers, upstream_addr) =
        match request::update_request_headers(&request_buffer[..header_end_pos], &client, &config)
            .await
        {
            Ok((headers, addr)) => (headers, addr),

            Err(e) => {
                tracing::error!(error = ?e, "Failed to update request headers");

                let message = match &e {
                    ProxyError::BadRequest(msg)
                    | ProxyError::NotFound(msg)
                    | ProxyError::RequestHeaderFieldsTooLarge(msg)
                    | ProxyError::UpstreamUnavailable(msg)
                    | ProxyError::BadGateway(msg)
                    | ProxyError::InternalServerError(msg)
                    | ProxyError::Forbidden(msg) => msg.as_deref(),
                };

                send_request_error(&mut client_stream, e, message).await?;
                return Ok(());
            }
        };

    let mut upstream_stream = match connect_to_upstream(&upstream_addr).await {
        Ok(strm) => strm,
        Err(e) => {
            tracing::error!(error = %e, "Failed to connect to upstream");
            send_error_response(&mut client_stream, ErrorResponse::BadGateway, None).await?;

            return Ok(());
        }
    };

    // Send the updated headers.
    let mut forwarded = updated_headers;
    forwarded.extend_from_slice(&request_buffer[header_end_pos..]);

    upstream_stream.write_all(&forwarded).await?;

    // Relay bytes in both directions
    let (client_to_upstream, upstream_to_client) =
        copy_bidirectional(&mut client_stream, &mut upstream_stream).await?;

    tracing::info!(
        client = %client.peer_addr,
        upstream = %upstream_addr,
        "Relayed {} bytes from client to upstream and {} bytes from upstream to client",
        client_to_upstream,
        upstream_to_client
    );

    Ok(())
}

//=== handles connection to upstream ===//
async fn connect_to_upstream(upstream_addr: &str) -> Result<TcpStream, std::io::Error> {
    //for debugging purpose first basic test.
    tracing::info!("Connecting to an upstream server at {}", upstream_addr);
    TcpStream::connect(upstream_addr).await
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}

async fn send_request_error(
    client_stream: &mut TcpStream,
    error: ProxyError,
    message: Option<&str>,
) -> std::io::Result<()> {
    let response = ErrorResponse::from(error);
    send_error_response(client_stream, response, message).await
}
