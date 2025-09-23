use tokio::net::TcpListener;
use crate::db::Db;
use crate::commands::handle_client;

pub async fn run(addr: &str) ->Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("mini-redis is running on: {}", addr);

    let db = Db::new();

    loop {
        let (socket, _) = listener.accept().await()?;
        let db = db.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, db).await {
                eprintln!{"Client Error", e};
            }
        });
    }
}