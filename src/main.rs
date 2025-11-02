use tokio::net::{TcpListener, TcpStream}; // ⬅️ Improved 'use' statement
use anyhow;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:6379";

    let listener: TcpListener = TcpListener::bind(addr).await?;
    println!("Server Listening on: {}", addr);

    loop {
        let (socket, client_addr) = listener.accept().await?;
        println!("Connection established from: {}", client_addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling connection: {}", e);
            }
        })
    }
}


async fn handle_connection(mut socket: TcpStream) -> anyhow::Result<()> {

    println!("Connection Closed!");
    Ok(())
}