use std::error::Error;
use std::fmt;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;

#[derive(Debug)]
struct ServerError {
    message: String
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ServerError {}

impl ServerError {
    fn new(error_message: &str) -> ServerError {
        ServerError {message: error_message.to_string()}
    }
}
pub struct Server {
    pub(crate) address: String,
    pub(crate) max_retries: usize,
    pub(crate) max_delay: u64,
    pub(crate) initial_delay: u64
}

impl Server {
    pub fn new(address: String, max_retries: usize, max_delay: u64, initial_delay: u64) -> Self {
        Server {
            address,
            max_retries,
            max_delay,
            initial_delay
        }
    }

    async fn try_binding_server(&self) -> Result<TcpListener, Box<dyn Error>> {
        let mut delay = self.initial_delay;
        for attempts in 0..self.max_retries {
            match TcpListener::bind(&self.address).await {
                Ok(tcp_listener) => {
                    println!("Server is running on: {}", &self.address);
                    return Ok(tcp_listener);
                },
                Err(error) => {
                    let error_message = format!("Attempt {} : Could not bind to '{}' . Error: {} . Retrying in {}",
                                                attempts + 1, &self.address, error, delay);
                    eprintln!("{}", &error_message);
                    if attempts == self.max_retries - 1 {
                        return Err(Box::new(ServerError::new(&error_message)));
                    }
                    sleep(Duration::from_secs(delay)).await;
                    delay = (delay * 2).min(self.max_delay);
                }
            }
        }
        Err(Box::new(ServerError::new("Failed to bind after multiple attempts")))
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let tcp_listener = self.try_binding_server().await?;

        loop {
            let (mut tcp_socket, socket_address) = tcp_listener.accept().await?;
            println!("New connection from: {}", socket_address);

            tokio::spawn(async move {
                let mut data_buffer = [0; 1024];

                let server_response = b"Whats up?\n";

                loop {
                    match tcp_socket.read(&mut data_buffer).await {
                        Ok(0) => {
                            println!("Connection closed by: {}", socket_address);
                            return;
                        }
                        Ok(read_bytes) => {
                            if tcp_socket.write_all(server_response).await.is_err() {
                                eprintln!("Failed to send response to {}", socket_address);
                                return;
                            }
                        }
                        Err(error) => {
                            eprintln!("Failed to read from socket: {}", error);
                            return;
                        }
                    }
                }
            });
        }
    }
}
