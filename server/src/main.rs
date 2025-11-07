use tokio::net::{ TcpListener, TcpStream };
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:6379";

    let listener: TcpListener = TcpListener::bind(addr).await?;
    println!("The server is listening on: {}", addr);

    loop {
        let (socket, client_addr) = listener.accept().await?;
        println!("Connection established from: {}", client_addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling conneciton: {}", e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<()> {
    let mut buf = [0;512];

    loop {
        let n = socket.read(&mut buf).await?;

        if n==0 {
            break;
        }

        let incoming_data = String::from_utf8_lossy(&buf[..n]);

        if incoming_data.to_uppercase().contains("PING") {
            let response = "+PONG\r\n";

            socket.write_all(response.as_bytes()).await?;
            println!("SENT PONG TO THE CLIENT");
        }
    }

    println!("CONNECTION CLOSED!");
    Ok(())
}