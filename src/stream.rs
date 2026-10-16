use crate::proxy;
use crate::types::{Client, Config};
use std::io::Error;
use std::sync::Arc;
use tokio::net::TcpStream;

pub async fn handle_stream(
    client_stream: TcpStream,
    client: Client,
    config: Arc<Config>,
) -> Result<(), Error> {
    loop {
        let peer_addr = &client.peer_addr;

        match proxy::relay(client_stream, &client, config).await {
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
