use tokio::net::TcpListener;
mod config;

const PROXY_SERVER_ADDR: &str = "127.0.0.1:7777";

#[tokio::main]
async fn main() {
    let server_addr = match std::env::var("PORT") {
        Ok(port) => format!("0.0.0.0:{port}"),
        Err(_) => std::env::args()
            .nth(1)
            .unwrap_or(PROXY_SERVER_ADDR.to_string()),
    };

    let tcp_listener = TcpListener::bind(&server_addr)
        .await
        .expect("Failed to bind to address");

    println!(
        "ProxyMambo is listening for incoming on {}",
        tcp_listener.local_addr().unwrap()
    );

    loop {
        let (stream, _) = tcp_listener
            .accept()
            .await
            .expect("Failed to accept incoming connection");
        println!("Accepted connection from {}", stream.peer_addr().unwrap());
        // tokio::spawn(async move {});
    }
}
