use tokio::net::TcpListener;
use bytes::BytesMut;

#[tokio::main]
pub async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:6969").await?;
    loop {
      let (mut socket, _) = listener.accept().await?;
      println!("connection accepted {:?}", socket);
      
    }


    Ok(())
}
