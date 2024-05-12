//! async echo server

use my_redis::boilerplate::tracing_subscribe_boilerplate;
use my_redis::boilerplate::SubKind;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
        tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
        // tracing_subscribe_boilerplate(SubKind::Console);
        tracing::info!("Tracing Subscriber active.");
        tracing::debug!("Attempting to connect to local port...");
        // Stream that is R&&W. We split, so we can mutably borrow a separate component for each.
        let socket = TcpStream::connect("127.0.0.1:6142").await?;
        let (mut rd, mut wr) = io::split(socket);

        tracing::debug!("Spawning Socket writer...");
        // separate thread writing
        tokio::spawn(async move {
                wr.write_all(b"hello\r\n").await?;
                wr.write_all(b"world\r\n").await?;
                Ok::<_, io::Error>(()) // TODO: what are mechanisms and limits of type inference?
        });

        tracing::debug!("Reading from socket...");
        let mut buf = vec![0; 128];
        loop {
                let n = rd.read(&mut buf).await?;
                if n == 0 {
                        break;
                }
                tracing::info!(
                        "GOT: {:?}",
                        std::str::from_utf8(&buf[..n]).expect("valid utf8")
                );
        }
        Ok(())
}
