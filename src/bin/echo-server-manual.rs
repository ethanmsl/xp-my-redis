//! echo server, manual copy

use my_redis::boilerplate::tracing_subscribe_boilerplate;
use my_redis::boilerplate::SubKind;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
        tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
        tracing::info!("Tracing Subscriber active.");
        tracing::debug!("Attempting to listen to local port...");
        let listener = TcpListener::bind("127.0.0.1:6142").await?;
        tracing::debug!("Listener bound; entering loop...");
        loop {
                tracing::info!("Awaiting socket receipt...");
                let (mut socket, _) = listener
                        .accept()
                        .await?;
                tracing::debug!("Socket accepted; Spawning thread to process...");
                tokio::spawn(async move {
                        tracing::debug!("1024 byte buffer being allotted");
                        // NOTE_1: multiple buffer sizes fed into read seem to work. appears it won't overfill
                        // NOTE_2: we use a **Vec** here, *not* an array
                        //         an array would be stored in the Future's state-machine, potentioally bloating it.
                        //         (not sure if this would make a smaller than u64 array worthwhile. More work for the scheduler?)
                        // let mut buf = vec!(0; 1024);
                        let mut buf = vec![0; 7];
                        // let mut buf = vec!(0; 1);
                        // let mut buf = vec!(0; 0); // doesn't pass 'awaiting data from socket' stage
                        loop {
                                tracing::info!("Awaiting data from socket...");
                                // read() will return immediately if read portion is closed (not error)
                                // this means handling the closing of the read component is critical
                                match socket
                                        .read(&mut buf)
                                        .await
                                {
                                        // return value of `Ok(0)` signifies the remote closed
                                        Ok(0) => return, //**Critical**: prevents infinite loop. read() does not error on a closed 'read half' of TcpStream
                                        Ok(n) => {
                                                let ref_so_i_can_print = &buf[..n];
                                                tracing::info!(
                                                        n,
                                                        ?ref_so_i_can_print,
                                                        "Read data from socket."
                                                );
                                                // copy data back to socket
                                                if socket
                                                        .write_all(&buf[..n])
                                                        .await
                                                        .is_err()
                                                {
                                                        // unexpected socket error.
                                                        // ... we just stop
                                                        eprintln!("failed to write to socket");
                                                        return;
                                                }
                                        }
                                        Err(_) => {
                                                //unexpected socket error; still just stop
                                                return;
                                        }
                                }
                        }
                });
        }
        // unreachable!();
}
