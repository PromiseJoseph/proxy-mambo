use std::{sync::Arc, vec};

use crate::stream::handle_stream;
use tokio::net::TcpListener;
mod config;
mod proxy;
mod request;
mod stream;
mod types;
use crate::types::{Client, Config, Route, Upstream, UpstreamPool};

#[tokio::main]
async fn main() {
    let mut config = Config::new();

    //there will be no environment variables for the upstream server address for the final proxy-mambo MVP..
    let server_addr = match std::env::var("PORT") {
        Ok(port) => format!("0.0.0.0:{port}"),
        Err(_) => std::env::args()
            .nth(1)
            .unwrap_or(config.get_proxy_addr().to_string()), // Use the proxy address from the config if no command-line argument is provided
    };

    //upstream test
    let default_upstream_addr = config
        .get_upstream_pool("default")
        .unwrap()
        .upstreams
        .first()
        .unwrap()
        .addr
        .clone();

    let guest_upstream_addr = String::from("127.0.0.1:8001"); // Default guest upstream address

    let main_upstream_addr = std::env::args()
        .nth(2)
        .unwrap_or(default_upstream_addr.clone()); // Use the upstream address from the config if no command-line argument is provided

    config.set_proxy_addr(&server_addr);

    let routes = vec![
        Route {
            path: "/api".to_string(),
            pool_id: "api".to_string(),
        },
        Route {
            path: "/".to_string(),
            pool_id: "guest".to_string(),
        },
    ];

    let upstreams = vec![
        UpstreamPool {
            id: "api".to_string(),
            upstreams: vec![Upstream {
                id: "api".to_string(),
                addr: main_upstream_addr.clone(),
            }],
        },
        UpstreamPool {
            id: "guest".to_string(),
            upstreams: vec![Upstream {
                id: "guest".to_string(),
                addr: guest_upstream_addr,
            }],
        },
    ];

    config.new_upstream_pools(upstreams.clone());

    // //temporary test for the default upstream server
    // if main_upstream_addr != default_upstream_addr {
    //     config.set_upstreams(upstreams);
    // }

    config.set_routes(routes);

    config.set_max_header_size(8192);

    let tcp_listener = TcpListener::bind(config.get_proxy_addr())
        .await
        .expect("Failed to bind to address");

    println!(
        "ProxyMambo is listening for incoming on {}",
        tcp_listener.local_addr().unwrap()
    );

    let config = Arc::new(config);

    loop {
        let (stream, peer_addr) = tcp_listener
            .accept()
            .await
            .expect("Failed to accept connection");

        println!("Accepted connection from {}", stream.peer_addr().unwrap());

        let client = Client { peer_addr };
        let config = Arc::clone(&config);

        tokio::spawn(async move {
            if let Err(e) = handle_stream(stream, client, config).await {
                eprintln!("Error handling client connection: {}", e);
            }
        });
    }
}
