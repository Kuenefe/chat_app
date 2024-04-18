mod server;
use server::*;

#[tokio::main]
async fn main() {
    let my_server = Server {
        address: "127.0.0.1:8080".to_string(),
        max_retries: 64,
        max_delay: 5,
        initial_delay: 1
    };

    if let Err(error) = my_server.run().await {
        eprint!("Server error: {}", error)
    };
}
