use crate::Handler;
use crate::Listener;
use std::sync::Arc;
use std::sync::Mutex;
pub async fn run<K: std::clone::Clone + std::cmp::Eq + std::hash::Hash + std::default::Default + std::marker::Send + std::str::FromStr + 'static, V:std::default::Default + std::marker::Send + std::clone::Clone + 'static>(listener: &Arc<Listener<K, V>>) -> std::io::Result<()> {

    loop {
        let listener_mutex = Arc::new(Mutex::new(listener.clone()));
        let socket = listener_mutex.lock().unwrap().accept().await?;
        let mut handler = Handler::new(&*listener_mutex.lock().unwrap(), socket);

        tokio::spawn(async move {
            if let Err(err) = process_method(&mut handler).await {
                println!("Connection Error");
            }
        });
    }
}

async fn process_method<K: std::clone::Clone + std::cmp::Eq + std::hash::Hash + std::default::Default + std::marker::Send + std::str::FromStr, V: std::default::Default + std::marker::Send + std::clone::Clone>(handler: &mut Handler<'_, K, V>) -> Result<(), std::io::Error> {
    while !handler.shutdown.is_shutdown() {
        let ( command, attrs ) = tokio::select! {
            res = handler.connection.read_frame() => res?,
            _ = handler.shutdown.listen_recv() => {
                println!("return from receive listen");
                return Ok(());
            }
        };
        handler.process_query(command, attrs).await?;
    }
    Ok(())
}