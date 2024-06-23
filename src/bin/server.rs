use tokio::{
  net::TcpListener,
  sync::{broadcast, mpsc},
};
use tokio::signal;
use crow_cache::{server, Listener};

#[tokio::main]
pub async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:6969").await?;
    let shutdown = signal::ctrl_c();
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut listener = Listener::new(
      listener, 
      notify_shutdown, 
      shutdown_complete_tx, 
      shutdown_complete_rx
    );
    tokio::select! {
      res = server::run(&mut listener) => {
        if let Err(err) = res {
          println!("failed to accept new connection");
        }
      }
      _ = shutdown => {
        println!("In the shutdown loop")
      }
    }
    drop(listener.notify_shutdown);
    drop(listener.shutdown_complete_tx);
    println!("before teh final shutdown");
    let _ = listener.shutdown_complete_rx.recv().await;
    println!("after teh final shtdown");

    Ok(())
}
