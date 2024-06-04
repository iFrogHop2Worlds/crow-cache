use tokio::net::TcpStream;



#[tokio::main]
pub async fn main() -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect("127.0.0.1:6969").await?;

    Ok(())
}
