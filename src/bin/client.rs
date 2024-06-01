//! Client

use mini_redis::client;
use my_redis::boilerplate::{tracing_subscribe_boilerplate, SubKind};
use tokio::sync::{mpsc, oneshot};

// I did not choose this name: "Responder" is type of "sender" half of channel
// to be given as a defacto address to receive a response at
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    use Command::*;
    tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
    // tracing_subscribe_boilerplate(SubKind::Console);
    tracing::info!("Tracing Subscriber active.");

    // tx: send
    // rx: receive
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    // Has unique access to the connection (confusingly named "client"... in this file named "client")
    // reads from queued message requests and sends them
    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await
                                                          .expect("Connect to port with server.");

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res); // discard errors, acceptable
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res); // discard errors, acceptable
                }
            }
        }
    });

    // adds request (Get) to deaddrop; manager takes and executes
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Get { key:  "foo".to_string(),
                        resp: resp_tx, };
        // send
        tx.send(cmd).await.expect("Sent or slept.");
        // await resp
        let res = resp_rx.await;
        tracing::info!(?res, "received response: ");
    });

    // adds request (Set) to deaddrop; manager takes and executes
    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Set { key:  "foo".to_string(),
                        val:  "bar".into(),
                        resp: resp_tx, };
        // send
        tx2.send(cmd).await.expect("Sent or slept");
        // await resp
        let res = resp_rx.await;
        tracing::info!(?res, "received response: ");
    });

    t1.await.expect("Key value acquired.");
    t2.await.expect("Key and its value set.");
    manager.await.expect("Manager... middlemans.");
}

use bytes::Bytes;

/// "Commands" (requests) to(for) redis server
#[derive(Debug)]
enum Command {
    Get {
        key:  String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key:  String,
        val:  Bytes,
        resp: Responder<()>,
    },
}
