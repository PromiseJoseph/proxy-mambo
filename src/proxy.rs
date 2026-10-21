use crate::request;
use crate::types::{Client, Config};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn relay(
    mut client_stream: TcpStream,
    client: &Client,
    config: Arc<Config>,
) -> Result<(), std::io::Error> {
    // Connect to the upstream server

    let mut request_buffer = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];

    let max_header_size = config.get_max_header_size();

    // Read the request headers from the client stream until we find the end of the headers or exceed the maximum header size.
    let header_end_pos = loop {
        let bytes_read = match client_stream.read(&mut chunk).await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
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
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "HTTP headers are too large",
                ));
            }

            break end;
        } //to be addressed as a http response with status code 431 Request Header Fields Too Large..

        if request_buffer.len() > max_header_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "HTTP headers are too large",
            ));
        }
    };

    // Modify only the HTTP headers.
    let (updated_headers, upstream_addr) =
        request::update_request_headers(&request_buffer[..header_end_pos], &client, &config)?;

    let stream_res = connect_to_upstream(&upstream_addr).await; // using only one upstream server for now.

    let mut upstream_stream = match stream_res {
        Ok(strm) => strm,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return Err(Error::new(ErrorKind::Other, "Failed to connect to server.")); // test will be changed to a http response with status code 502 Bad Gateway..
        }
    };
    // Send the updated headers.
    let mut forwarded = updated_headers;
    forwarded.extend_from_slice(&request_buffer[header_end_pos..]);

    upstream_stream.write_all(&forwarded).await?;

    // Relay bytes in both directions
    let (client_to_upstream, upstream_to_client) =
        copy_bidirectional(&mut client_stream, &mut upstream_stream).await?;

    println!(
        "Finished relaying. Client -> upstream: {} bytes, upstream -> client: {} bytes",
        client_to_upstream, upstream_to_client
    );

    Ok(())
}

//=== handles connection to upstream ===//
async fn connect_to_upstream(upstream_addr: &str) -> Result<TcpStream, std::io::Error> {
    //for debugging purpose first basic test.
    println!("Connecting to an upstream server at {}", upstream_addr);
    TcpStream::connect(upstream_addr).await
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
}
