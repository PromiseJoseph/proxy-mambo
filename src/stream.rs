use crate::proxy;
use crate::types::Client;
use std::io::Error;
use tokio::net::TcpStream;

pub async fn handle_stream(client_stream: TcpStream, client: Client) -> Result<(), Error> {
    loop {
        let peer_addr = &client.peer_addr;

        match proxy::relay(client_stream, &client).await {
            Ok(()) => {
                println!("Finished proxy connection for {}", peer_addr);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Proxying for {} failed: {}", peer_addr, e);
                return Err(e);
            }
        }
    }
}
