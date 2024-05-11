use mini_redis::Frame;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // bind "listener" to an address
    let listener = TcpListener::bind("127.0.0.1:6379").await
                                                      .expect("Listener binds.");

    loop {
        // The Second item contains the IP and port of the new connection.
        // -- presumably "accept" is "accept if asked, wait otherwise"
        let (socket, _) = listener.accept()
                                  .await
                                  .expect("Socket acquired.");
        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    // "mini_redis specific" Read&Write "frames" instead of working with byte streams
    let mut connection = mini_redis::Connection::new(socket);

    if let Some(frame) = connection.read_frame()
                                   .await
                                   .expect("Frame read.")
    {
        println!("GOT: {:?}", frame);

        // Respond withn an error  <-- similar to `todo!();` without crashing program
        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response)
                  .await
                  .expect("Frame write.");
    }
}
