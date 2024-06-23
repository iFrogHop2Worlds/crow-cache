use crate::Handler;
use crate::Listener;

pub async fn run<K: std::clone::Clone + std::cmp::Eq + std::hash::Hash + std::default::Default + std::marker::Send, V:std::default::Default + std::marker::Send>(listener: &Listener<K, V>) -> std::io::Result<()> {
    loop {
        let socket = listener.accept().await?;
        let mut handler = Handler::new(listener, socket);

        tokio::spawn(async move {
          if let Err(err) = process_method(&mut handler).await {
            println!("Connection Error");
          }
        });
    }
}

async fn process_method<K, V>(handler: &mut Handler<K, V>) -> Result<(), std::io::Error> {
    while !handler.shutdown.is_shutdown() {
        let ( command, attrs ) = tokio::select! {
            res = handler.connection.read_frame() => res?,
            _ = handler.shutdown.listen_recv() => {
                println!("return from recieve listen");
                return Ok(());
            }
        };
        handler.prcess_query(command, attrs).await?;
    }
    Ok(())
}