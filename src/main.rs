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

    match my_server.run().await {
        Ok(()) => println!("Server terminated successfully!"),
        Err(error) => eprintln!("Server error: {}", error)
    }
}
