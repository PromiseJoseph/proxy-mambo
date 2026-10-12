use crate::request;
use crate::types::Client;
use std::io::{Error, ErrorKind};
use tokio::io::copy_bidirectional;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn relay(mut client_stream: TcpStream, client: &Client) -> Result<(), std::io::Error> {
    let upstream_addr = std::env::args().nth(2).unwrap();

    // Connect to the upstream server
    let stream_res = connect_to_upstream(&upstream_addr).await;

    let mut upstream_stream = match stream_res {
        Ok(strm) => strm,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return Err(Error::new(ErrorKind::Other, "Failed to connect to server.")); // test will be changed to a http response with status code 502 Bad Gateway..
        }
    };

    let mut request_buffer = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    const MAX_HEADER_SIZE: usize = 64 * 1024;

    let header_end = loop {
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
            if end > MAX_HEADER_SIZE {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "HTTP headers are too large",
                ));
            }

            break end;
        }

        if request_buffer.len() > MAX_HEADER_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "HTTP headers are too large",
            ));
        }
    };

    // Modify only the HTTP headers.
    let updated_headers = request::update_request_headers(
        &request_buffer[..header_end],
        &upstream_addr.as_str(),
        &client,
    )?;

    // Send the updated headers.
    let mut forwarded = updated_headers;
    forwarded.extend_from_slice(&request_buffer[header_end..]);

    upstream_stream.write_all(&forwarded).await?;

    // Relay bytes in both directions
    let (client_to_upstream, upstream_to_client) =
        copy_bidirectional(&mut client_stream, &mut upstream_stream).await?;

    println!(
        "Finished relaying. Client -> upstream: {} bytes, upstream -> client: {} bytes",
        client_to_upstream, upstream_to_client
    );

    Ok(())

    // // Send the request to upstream server
    // let incoming_request = update_request(request);

    // upstream_stream
    //     .write_all(incoming_request.as_bytes())
    //     .await
    //     .expect("Failed to send request to server");

    // println!("Sent request to server: \"{}\"", incoming_request);

    // let mut buffer = [0; 1024];
    // let bytes_read = match upstream_stream.read(&mut buffer).await {
    //     Ok(bytes) => bytes,
    //     Err(e) => {
    //         eprintln!("Error: {}", e);
    //         return String::from("Failed to read response from server.");
    //     }
    // };

    // if bytes_read == 0 {
    //     return String::from("Server closed the connection unexpectedly.");
    // }

    // let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    // let updated_response = update_response(&response);
    // return updated_response;
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
