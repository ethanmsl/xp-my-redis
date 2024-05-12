//! Client

use mini_redis::client;
use my_redis::boilerplate::{tracing_subscribe_boilerplate, SubKind};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
        use Command::*;
        tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));

        // tx: send
        // rx: receive
        let (tx, mut rx) = mpsc::channel(32);
        let tx2 = tx.clone();

        // Has unique access to the connection (confusingly named "client"... in this file named "client")
        // reads from queued message requests and sends them
        let manager = tokio::spawn(async move {
                let mut client = client::connect("127.0.0.1:6379")
                        .await
                        .expect("Connect to port with server.");

                while let Some(cmd) = rx.recv().await {
                        match cmd {
                                Get { key } => {
                                        client.get(&key)
                                                .await
                                                .expect("Got key's value from client.");
                                }
                                Set { key, val } => {
                                        client.set(&key, val)
                                                .await
                                                .expect("Set key's value with client.");
                                }
                        }
                }
        });

        // adds request (Get) to deaddrop; manager takes and executes
        let t1 = tokio::spawn(async move {
                let cmd = Get {
                        key: "foo".to_string(),
                };
                tx.send(cmd).await.expect("Sent or slept.");
        });

        // adds request (Set) to deaddrop; manager takes and executes
        let t2 = tokio::spawn(async move {
                let cmd = Set {
                        key: "foo".to_string(),
                        val: "bar".into(),
                };
                tx2.send(cmd).await.expect("Sent or slept");
        });

        t1.await.expect("Key value acquired.");
        t2.await.expect("Key and its value set.");
        manager.await.expect("Manager... middlemans.");
}

use bytes::Bytes;

/// "Commands" (requests) to(for) redis server
#[derive(Debug)]
enum Command {
        Get { key: String },
        Set { key: String, val: Bytes },
}
