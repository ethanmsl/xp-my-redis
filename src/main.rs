use boilerplate::{tracing_subscribe_boilerplate, SubKind};
use mini_redis::Frame;
use my_redis::boilerplate;
use tokio::net::{TcpListener, TcpStream};
use tracing;

#[tokio::main]
async fn main() {
      tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
      // tracing_subscribe_boilerplate(SubKind::Console);
      tracing::info!("Tracing Subscriber active.");

      // bind "listener" to an address
      tracing::debug!("Binding Listener to ip & port...");
      let listener = TcpListener::bind("127.0.0.1:6379")
            .await
            .expect("Listener binds.");
      tracing::debug!("listener bound.");

      loop {
            // The Second item contains the IP and port of the new connection.
            // -- presumably "accept" is "accept if asked, wait otherwise"
            tracing::debug!("Awaiting socket receipt...");
            let (socket, _) = listener.accept().await.expect("Socket acquired.");
            tracing::debug!("Socket received; Spawning thread to process...");
            tokio::spawn(async move {
                  tracing::debug!("Thread for socket processing spawned.");
                  tracing::debug!("Processing socket...");
                  process(socket).await;
                  tracing::debug!("Socket processed.");
            });
      }
}

async fn process(socket: TcpStream) {
      use std::collections::HashMap;

      use mini_redis::Command::{self, Get, Set};

      let mut db = HashMap::new();
      // "mini_redis specific" Read&Write "frames" instead of working with byte streams
      let mut connection = mini_redis::Connection::new(socket);

      while let Some(frame) = connection.read_frame().await.expect("frame read") {
            tracing::info!("GOT: {:?}", frame);
            let response = match Command::from_frame(frame).unwrap() {
                  Set(cmd) => {
                        // value stored as Vec<u8>
                        db.insert(cmd.key().to_string(), cmd.value().to_vec());
                        Frame::Simple("OK".to_string())
                  }
                  Get(cmd) =>
                        if let Some(value) = db.get(cmd.key()) {
                              Frame::Bulk(value.clone().into())
                        } else {
                              Frame::Null
                        },
                  cmd => unimplemented!("{:?}", cmd),
            };
            // write response to client
            connection
                  .write_frame(&response)
                  .await
                  .expect("Write to client");
      }
}
