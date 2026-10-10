use crate::proxy;
use crate::types::Client;
use std::println;

use std::io::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn handle_stream(
    mut reader: OwnedReadHalf,
    mut writer: OwnedWriteHalf,
    client: Client,
) -> Result<(), Error> {
    let mut buffer = [0; 1024]; //for test
    loop {
        let peer_addr = &client.peer_addr;

        match reader.read(&mut buffer).await {
            Ok(0) => {
                println!("Client {} disconnected", peer_addr);
                break;
            }
            Ok(bytes) => {
                let request = String::from_utf8_lossy(&buffer[..bytes]);
                println!("Received a request from  {}", peer_addr);

                let upstream_response = proxy::relay(&request).await;

                println!(
                    "Received a response from upstream server for client {}: {}",
                    peer_addr, upstream_response
                );
                // Send the response back to the client
                writer.write_all(&upstream_response.as_bytes()).await?;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(())
}
