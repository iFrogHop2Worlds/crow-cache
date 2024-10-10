use std::hash::Hash;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::Arc;
use crate::{helper::buffer_to_array, Command};
use crate::{LRUCache, Listener};
use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::{broadcast, mpsc},
};

pub struct Handler<'a, K: Eq + Hash + Clone + Default + Send + FromStr + 'static, V: Default + Send + Clone + 'static> {
    pub connection: Connection,
    pub _db: &'a LRUCache<K, V>,
    pub shutdown: Shutdown,

    _shutdown_complete: mpsc::Sender<()>,
}

pub struct Connection {
    pub stream: TcpStream,
}

pub struct Shutdown {
    shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Connection {
    fn new(stream: TcpStream) -> Connection {
        Connection { stream }
    }

    pub async fn read_frame(&mut self) -> Result<(Command, Vec<String>), std::io::Error> {
        std::thread::sleep(std::time::Duration::from_millis(5000));
        let mut buf = BytesMut::with_capacity(1024);
        self.stream.read_buf(&mut buf).await?;
        let attrs = buffer_to_array(&mut buf);
        Ok((Command::get_command(&attrs[0]), attrs))
    }
}

impl Shutdown {
    fn new(shutdown: bool, notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown { shutdown, notify }
    }

    pub async fn listen_recv(&mut self) -> Result<(), tokio::sync::broadcast::error::RecvError> {
        println!("inside Listen_recv");

        self.notify.recv().await?; // returns error of type `tokio::sync::broadcast::error::RecvError`
        self.shutdown = true;
        Ok(())
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }
}

impl<K: std::clone::Clone + std::cmp::Eq + std::hash::Hash + std::default::Default + std::marker::Send + FromStr, V: std::default::Default + std::marker::Send + std::clone::Clone> Handler<'_, K, V> {
    pub fn new(listener: &Arc<Listener<K, V>>, socket: TcpStream) -> Handler<K, V> {
        Handler {
            connection: Connection::new(socket),
            _db: listener.db.clone(),
            shutdown: Shutdown::new(false, listener.notify_shutdown.subscribe()),
            _shutdown_complete: listener.shutdown_complete_tx.clone(),
        }
    }

    pub async fn process_query(
        &mut self,
        command: Command,
        attrs: Vec<String>,
    ) -> Result<(), std::io::Error> {
        let connection = &mut self.connection;
        let mut _db = &self._db;

        match command {
            Command::Get => {
                let key: &K = match K::from_str(&attrs[0]) {
                    Ok(key) => &key,
                    Err(_) => {
                        // Handle conversion error
                        return Ok(());
                    }
                };

                let result = _db.get(key).and(None);
                // entries.lock().unwrap().get(k);
                match result {
                    Some(Ok(result)) => {
                        if let Ok(_) = connection.stream.write_all(&result).await {
                            println!("Get fired");
                        } else {
                            // Handle write error (e.g., log or send error message)
                        }
                    }
                    Some(Err(_err)) => {
                        connection.stream.write_all(b"").await?;
                    }
                }

                return Ok(());
            }
            Command::Put => {
                let resp = _db.write(&attrs);
                match resp {
                    Ok(result) => {
                        connection.stream.write_all(&result.as_bytes()).await?;
                    }
                    Err(_err) => {
                        connection.stream.write_all(b"").await?;
                    }
                }

                return Ok(());
            }
            Command::Invalid => {
                connection.stream.write_all(b"invalid command").await?;
                Err(std::io::Error::from(ErrorKind::InvalidData))
            }
            _ => {}
        }
    }
}