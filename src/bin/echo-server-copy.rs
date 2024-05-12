//! async echo server

use my_redis::boilerplate::tracing_subscribe_boilerplate;
use my_redis::boilerplate::SubKind;
use tokio::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
        tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
        // tracing_subscribe_boilerplate(SubKind::Console);
        tracing::info!("Tracing Subscriber active.");
        tracing::warn!(
                r#"This server accepts TCP, not HTTP.  If you'd like to hear something echoed back try:\n`echo "Hello, echo server" | nc 127.0.0.1 6142`"#
        );
        tracing::debug!("Attempting to listen to local port...");
        // Note: this is a different port than for reddis comms
        let listener = TcpListener::bind("127.0.0.1:6142").await?;

        loop {
                let (mut socket, _) = listener.accept().await?;
                tokio::spawn(async move {
                        let (mut rd, mut wr) = socket.split();
                        if io::copy(&mut rd, &mut wr).await.is_err() {
                                eprintln!("failed to copy");
                        }
                });
        }
        // unreachable!();
}
