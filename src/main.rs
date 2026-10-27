use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicUsize},
    vec,
};

use crate::stream::handle_stream;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
mod algorithm;
mod config;
mod proxy;
mod request;
mod stream;
mod types;
use crate::types::{Client, Config, LoadBalancingAlgorithm, Route, Upstream, UpstreamPool};

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
            path: "/api/*".to_string(),
            pool_id: "api".to_string(),
        }, // test for wildcard route matching..
        Route {
            path: "/".to_string(),
            pool_id: "guest".to_string(),
        },
    ];

    /*weight and current weight  for  weighted round-robin upstreams */
    let weights = HashMap::from([
        ("guest-1".to_string(), 5),
        ("guest-2".to_string(), 1),
        ("guest-3".to_string(), 1),
    ]);

    let current_weights = HashMap::from([
        ("guest-1".to_string(), 0),
        ("guest-2".to_string(), 0),
        ("guest-3".to_string(), 0),
    ]);

    let upstreams = vec![
        UpstreamPool {
            id: "api".to_string(),
            algorithm: LoadBalancingAlgorithm::RoundRobin {
                current_index: Arc::new(AtomicUsize::new(0)),
            },
            upstreams: vec![Upstream {
                id: "api-1".to_string(),
                addr: main_upstream_addr.clone(),
            }],
        },
        UpstreamPool {
            id: "guest".to_string(),
            algorithm: LoadBalancingAlgorithm::WeightedRoundRobin {
                /* Initialize the weights and current weights for the weighted round-robin algorithm */
                weights: weights,
                current_weights: Mutex::new(current_weights),
            },
            upstreams: vec![
                Upstream {
                    id: "guest-1".to_string(),
                    addr: guest_upstream_addr,
                },
                Upstream {
                    id: "guest-2".to_string(),
                    addr: "127.0.0.1:8002".to_string(),
                }, //test ustream server for guest-2.
                Upstream {
                    id: "guest-3".to_string(),
                    addr: "127.0.0.1:8003".to_string(),
                }, //test ustream server for guest-3.
            ],
        },
    ];

    config.new_upstream_pools(upstreams);

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
