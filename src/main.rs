use tokio::net::{ TcpListener, TcpStream };

use crate::{ db::Db, frame::Frame, connection::Connection };

mod connection;
mod frame;
mod db;

#[tokio::main]
async fn main() {
    
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Mini-Redis server listening on 127.0.0.1:6379");

    let db = Db::new();

    loop {

        let (socket, _)  = listener.accept().await.unwrap();

        let db = db.clone();

        tokio::spawn(async move {
            process(socket, db).await;
        });
    }

}

async fn process(socket: TcpStream, db: Db) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {

        let response = match frame {
            Frame::Array(ref cmd_parts) => {

                if let Some(Frame::Bulk(data)) = cmd_parts.get(0) {
                    let cmd_name = String::from_utf8(data.to_vec()).unwrap();

                    match cmd_name.as_str() {
                        "SET" => {

                            if let (Some(Frame::Bulk(key_bytes)), Some(Frame::Bulk(val_bytes))) = (cmd_parts.get(1), cmd_parts.get(2)) {
                                let key = String::from_utf8(key_bytes.to_vec()).unwrap();

                                db.set(key, val_bytes.clone());
                            }

                            Frame::Simple("OK".to_string())
                        }
                        "GET" => {
                            if let Some(Frame::Bulk(key_bytes)) = cmd_parts.get(1) {
                                let key = String::from_utf8(key_bytes.to_vec()).unwrap();
                                
                                if let Some(value) = db.get(&key) {
                                    Frame::Bulk(value)
                                } else {
                                    Frame::Null
                                }
                            } else {
                                Frame::Error("Usage: GET <key>".to_string())
                            }
                        }
                        _ => Frame::Error("Unknown Command".to_string())

                    }
                } else {
                    Frame::Error("Invalid Command Format!".to_string())
                }
            }
            _ => Frame::Error("Command must be and array".to_string())
        };

        if let Err(e) = connection.write_frame(&response).await {
            println!("failed to write response: {:?}", e);
            break; 
        }
    }
}
