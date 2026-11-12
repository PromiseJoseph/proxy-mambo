use crate::{config::AppSettings, stream::handle_stream};
use std::sync::Arc;

use tokio::net::TcpListener;
use tracing_subscriber;
mod algorithm;
mod config;
mod error;
mod proxy;
mod request;
mod response;
mod stream;
mod types;
use crate::types::Client;

#[tokio::main]
async fn main() {
    let settings = AppSettings::load("Config.toml").expect("Failed to load configuration");

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&settings.logging.level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    let config = Arc::new(
        settings
            .into_runtime_config()
            .expect("invalid  proxy configuration"),
    );

    let listener: TcpListener = match TcpListener::bind(config.get_proxy_addr()).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(error = %error, address = %config.get_proxy_addr(), "Failed to bind proxy listener");
            std::process::exit(1);
        }
    };

    tracing::info!(
        address = %listener.local_addr().unwrap(),
        "ProxyMambo is listening"
    );

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                tracing::error!(error = %error, "Failed to accept connection");
                continue;
            }
        };

        tracing::info!(client = %peer_addr, "Accepted client connection");

        let client = Client { peer_addr };
        let config = Arc::clone(&config);

        tokio::spawn(async move {
            if let Err(error) = handle_stream(stream, client, config).await {
                tracing::error!(
                    client = %peer_addr,
                    error = %error,
                    "Error handling client connection"
                );
            }
        });
    }
}
