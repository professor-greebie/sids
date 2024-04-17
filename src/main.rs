use tokio::net::TcpListener;

use mini_redis::{client, Result};
#[tokio::main]
async fn main() -> Result<()> {

    let listener = TcpListener::bind("localhost:8099").await.unwrap();
    println!("Listening on port 8099");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        println!("Accepted");
    }

    let mut client = client::connect("localhost:6379").await?;
    client.set("hello", "world".into()).await?;
    println!("Hello, world!");
    Ok(())
}
