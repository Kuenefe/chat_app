use std::error::Error;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;

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
                    eprintln!("Attempt {} : Could not bind to '{}' . Error: {} . Retrying in {}",
                              attempts + 1, &self.address, error, delay);
                    if attempts == self.max_retries -1 {
                        break;
                    }
                    sleep(Duration::from_secs(delay)).await;
                    delay = (delay * 2).min(self.max_delay);
                }
            }
        }
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to bind after multiple attempts")))
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let tcp_listener = self.try_binding_server().await?;

        loop {
            let (mut tcp_socket, socket_address) = tcp_listener.accept().await?;
            println!("New connection from: {}", socket_address);

            tokio::spawn(async move {
                let mut data_buffer = [0; 1024];

                let server_response = b"Hello my old friend";

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
