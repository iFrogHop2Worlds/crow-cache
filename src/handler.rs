use std::io::ErrorKind;

use crate::{helper::buffer_to_array, Command};
use crate::{LRUCache, Listener};
use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::{broadcast, mpsc},
};

pub struct Handler<K, V> {
    pub connection: Connection,
    pub _db: LRUCache<K, V>,
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
        Connection { stream: stream }
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

impl Handler {
    pub fn new(listener: &Listener, socket: TcpStream) -> Handler {
        Handler {
            connection: Connection::new(socket),
            _db: listener._db.clone(),
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
        let _db = &self._db;

        match command {
            Command::Get => {
                let result = _db.read(&attrs);
                // entries.lock().unwrap().get(k);
                match result {
                    Ok(result) => {
                        connection.stream.write_all(&result).await?;
                    }
                    Err(_err) => {
                        connection.stream.write_all(b"").await?;
                    }
                }

                return Ok(());
            }
            Command::Set => {
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
        }
    }
}