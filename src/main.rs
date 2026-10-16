use std::sync::Arc;

use crate::stream::handle_stream;
use tokio::net::TcpListener;
mod config;
mod proxy;
mod request;
mod stream;
mod types;
use crate::types::{Client, Config, Upstream};

#[tokio::main]
async fn main() {
    let mut config = Config::new();

    let server_addr = match std::env::var("PORT") {
        Ok(port) => format!("0.0.0.0:{port}"),
        Err(_) => std::env::args()
            .nth(1)
            .unwrap_or(config.get_proxy_addr().to_string()), // Use the proxy address from the config if no command-line argument is provided
    };

    let default_upstream_addr = config.get_upstream("default").unwrap().addr.clone();
    let upstream_addr = std::env::args()
        .nth(2)
        .unwrap_or(config.get_upstream("default").unwrap().addr.clone()); // Use the upstream address from the config if no command-line argument is provided

    config.set_proxy_addr(&server_addr);

    if upstream_addr != default_upstream_addr {
        config.add_upstream(Upstream {
            id: "api".into(),
            addr: upstream_addr,
        });
    }

    config.set_max_header_size(8192);

    let tcp_listener = TcpListener::bind(config.get_proxy_addr())
        .await
        .expect("Failed to bind to address");

    println!(
        "ProxyMambo is listening for incoming on {}",
        tcp_listener.local_addr().unwrap()
    );

    loop {
        let (stream, peer_addr) = tcp_listener
            .accept()
            .await
            .expect("Failed to accept connection");

        println!("Accepted connection from {}", stream.peer_addr().unwrap());

        let client = Client { peer_addr };

        let config = Arc::new(config.clone());

        tokio::spawn(async move {
            if let Err(e) = handle_stream(stream, client, config).await {
                eprintln!("Error handling client connection: {}", e);
            }
        });
    }
}
