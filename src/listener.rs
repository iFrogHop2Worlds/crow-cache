use tokio::{
    net::{ TcpStream, TcpListener },
    sync::{ mpsc, broadcast }
};

use crate::db::LRUCache;

pub struct Listener<K: std::clone::Clone + std::cmp::Eq + std::hash::Hash, V> {
    pub db: LRUCache<K, V>,
    pub listener: TcpListener,
    pub notify_shutdown: broadcast::Sender<()>,
    pub shutdown_complete_rx: mpsc::Receiver<()>,
    pub shutdown_complete_tx: mpsc::Sender<()>,
}
// lol :*)
impl<K: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::cmp::Eq + std::marker::Send + std::default::Default + 'static, V: std::default::Default + std::marker::Send + 'static> Listener<K, V> {
    pub fn new(
        listener: TcpListener,
        notify_shutdown: broadcast::Sender<()>,
        shutdown_complete_tx: mpsc::Sender<()>,
        shutdown_complete_rx: mpsc::Receiver<()>,
    ) -> Listener<K, V> {
        Listener {
            listener,
            db: LRUCache::new(300),
            notify_shutdown,
            shutdown_complete_rx, // this is a shorthand struct initialisation
            shutdown_complete_tx,
        }
    }
    pub async fn accept(&self) -> std::result::Result<TcpStream, std::io::Error> {
        match self.listener.accept().await {
            Ok((socket, _)) => return Ok(socket),
            Err(err) => {
                return Err(err.into());
            }
        }
    }
}