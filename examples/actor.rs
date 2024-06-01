//! actor example using tokio

use my_redis::{boilerplate::{tracing_subscribe_boilerplate, SubKind},
               error::Result};
use tokio::{io,
            sync::{mpsc, oneshot}};

/// Receiver component to listen in on
#[derive(Debug)]
struct MyActor {
    receiver:   mpsc::Receiver<ActorMessage>,
    actor_info: Option<u32>,
}

/// Message to send to an actor
///
/// Should contaain message *for* actor
/// and sending channel for actor to *respond* with
/// Generator should hold on to receiver end, ofc.
#[derive(Debug)]
enum ActorMessage {
    SendMessage {
        message:    String,
        respond_to: oneshot::Sender<Option<u32>>,
    },
}

impl MyActor {
    /// Create a new actor, passing in a receiver
    fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        MyActor { receiver,
                  actor_info: Some(0) }
    }

    /// Take ActorMessage, opreate, and respond
    ///
    /// # NOTE:
    /// The Actor **must** respond.
    /// **OR** the receiver must be set up to deal with the Error
    /// of a dropped channel.
    async fn handle_message(&mut self, msg: ActorMessage) -> Result<()> {
        match msg {
            ActorMessage::SendMessage { message,
                                        respond_to, } => {
                match message {
                    _ if message.contains("increase") => {
                        if let Some(info) = self.actor_info.as_mut() {
                            *info += 1;
                        }
                        let _ = respond_to.send(None);
                    }
                    _ if message.contains("get") => {
                        let _ = respond_to.send(self.actor_info);
                    }
                    _ => {
                        println!("Unknown message: {}", message);
                        let _ = respond_to.send(None);
                    }
                }
                Ok(())
            }
        }
    }
}

// //////////////////////////////////// //
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));

    let (tx, rx) = mpsc::channel::<ActorMessage>(32); // number is buffer size
    let mut yorrick = MyActor::new(rx);

    // use tokio spawn to run MyActor and have it wait to receive messages
    // we don't take a yorrick_handle here; using our mpsc & oneshots solely
    let _ = tokio::spawn(async move {
        while let Some(msg) = yorrick.receiver.recv().await {
            tracing::info!("Received message: {:?}", msg);
            yorrick.handle_message(msg)
                   .await
                   .expect("Message handling failed.");
        }
        tracing::warn!("Actor has stopped receiving messages.");
    });

    let yorrick_tx = tx;

    let resp = send_message(yorrick_tx.clone(), "increase").await;
    println!("Response: {:?}", resp);
    let resp = send_message(yorrick_tx.clone(), "increase").await;
    println!("Response: {:?}", resp);
    let resp = send_message(yorrick_tx.clone(), "get").await;
    println!("Response: {:?}", resp);
    let resp = send_message(yorrick_tx.clone(), "increase").await;
    println!("Response: {:?}", resp);
    let resp = send_message(yorrick_tx.clone(), "get").await;
    println!("Response: {:?}", resp);

    Ok(())
}

async fn send_message(tx: mpsc::Sender<ActorMessage>,
                      message: impl Into<String>)
                      -> Result<Option<u32>> {
    // convert to String, if necessary
    let message = message.into();

    // oneshot channel to receive response from actor
    let (txo, rxo) = oneshot::channel();
    let msg = ActorMessage::SendMessage { message,
                                          respond_to: txo };

    tracing::info!("Sending message to actor.");
    tx.send(msg).await?;

    tracing::debug!("Waiting for response from actor.");
    Ok(rxo.await?)
}
