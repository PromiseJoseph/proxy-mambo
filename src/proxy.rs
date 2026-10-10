use crate::request::{update_request, update_response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn relay(request: &str) -> String {
    let upstream_addr_as_arg = std::env::args().nth(2);

    // Connect to the upstream server
    let stream_res = connect_to_upstream(&upstream_addr_as_arg.unwrap()).await;

    let mut stream = match stream_res {
        Ok(strm) => strm,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return String::from("Failed to connect to server.");
        }
    };

    // Send the request to upstream server
    let incoming_request = update_request(request);

    stream
        .write_all(incoming_request.as_bytes())
        .await
        .expect("Failed to send request to server");

    println!("Sent request to server: \"{}\"", incoming_request);

    let mut buffer = [0; 1024];
    let bytes_read = match stream.read(&mut buffer).await {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {}", e);
            return String::from("Failed to read response from server.");
        }
    };

    if bytes_read == 0 {
        return String::from("Server closed the connection unexpectedly.");
    }

    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    let updated_response = update_response(&response);
    return updated_response;
}

//=== handles connection to upstream ===//
async fn connect_to_upstream(upstream_addr: &str) -> Result<TcpStream, std::io::Error> {
    //for debugging purpose first basic test.
    println!("Connecting to an upstream server at {}", upstream_addr);
    TcpStream::connect(upstream_addr).await
}
