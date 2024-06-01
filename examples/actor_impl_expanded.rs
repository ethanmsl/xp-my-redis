//! actor example using tokio

use std::time::Duration;

use my_redis::boilerplate::{tracing_subscribe_boilerplate, SubKind};
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
/// Should contain message *for* actor
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
    async fn handle_message(&mut self, msg: ActorMessage) -> io::Result<()> {
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

    async fn run_my_actor(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }

    fn off_on_my_own(mut self) {
        tokio::spawn(async move {
            tracing::warn!("Woo! Spawned from function!");
            while let Some(msg) = self.receiver.recv().await {
                tracing::info!("Received message: {:?}", msg);
                self.handle_message(msg)
                    .await
                    .expect("Message handling failed.");
            }
            tracing::error!("Actor has stopped receiving messages.");
        });
    }

    async fn off_on_my_own_async(mut self) {
        tokio::spawn(async move {
            tracing::warn!("Woo! Spawned from function!");
            while let Some(msg) = self.receiver.recv().await {
                tracing::info!("Received message: {:?}", msg);
                self.handle_message(msg)
                    .await
                    .expect("Message handling failed.");
            }
            // NOTE: because this function is async, it will get dropped/cancelled
            //       by tokio as part of clean up.
            //       Resulting in the error below showing, unlike in the blocking versions.
            //       (at least that's current understanding)
            tracing::error!("(async run) Actor has stopped receiving messages.");
        });
    }
}

// //////////////////////////////////// //
#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));

    let (tx, rx) = mpsc::channel::<ActorMessage>(32); // number is buffer size
    let mut yorrick = MyActor::new(rx);

    // method based tokio_spawn
    // both blocking and async work
    yorrick.off_on_my_own();
    // yorrick.off_on_my_own_async().await;

    // oneshot channel to receive response from actor
    let (txo, rxo) = oneshot::channel();
    let msg = ActorMessage::SendMessage { message:    String::from("increase"),
                                          respond_to: txo, };
    // send message to actor
    tracing::info!("Sending message to actor.");
    tx.send(msg).await.expect("Message sent.");
    tracing::info!("Message sent to actor successfully.");
    // wait for oneshot response
    tracing::debug!("Waiting for response from actor.");
    let response = rxo.await.expect("Response received.");
    println!("Response: {:?}", response);

    // oneshot channel to receive response from actor
    let (txo, rxo) = oneshot::channel();
    let msg = ActorMessage::SendMessage { message:    String::from("increase"),
                                          respond_to: txo, };
    // send message to actor
    tracing::info!("Sending message to actor.");
    tx.send(msg).await.expect("Message sent.");
    tracing::info!("Message sent to actor successfully.");
    // wait for oneshot response
    tracing::debug!("Waiting for response from actor.");
    let response = rxo.await.expect("Response received.");
    println!("Response: {:?}", response);

    // oneshot channel to receive response from actor
    let (txo, rxo) = oneshot::channel();
    let msg = ActorMessage::SendMessage { message:    String::from("get"),
                                          respond_to: txo, };
    // send message to actor
    tracing::info!("Sending message to actor.");
    tx.send(msg).await.expect("Message sent.");
    tracing::info!("Message sent to actor successfully.");
    // wait for oneshot response
    tracing::debug!("Waiting for response from actor.");
    let response = rxo.await.expect("Response received.");
    println!("Response: {:?}", response);

    // /////////////////////// //

    let (tx, rx) = mpsc::channel::<ActorMessage>(32); // number is buffer size
    let mut yorrick_2 = MyActor::new(rx);
    // method based tokio_spawn
    // both blocking and async work
    // yorrick.off_on_my_own();
    yorrick_2.off_on_my_own_async().await;

    // oneshot channel to receive response from actor
    let (txo, rxo) = oneshot::channel();
    let msg = ActorMessage::SendMessage { message:    String::from("increase"),
                                          respond_to: txo, };
    // send message to actor
    tracing::info!("Sending message to actor.");
    tx.send(msg).await.expect("Message sent.");
    tracing::info!("Message sent to actor successfully.");
    // wait for oneshot response
    tracing::debug!("Waiting for response from actor.");
    let response = rxo.await.expect("Response received.");
    println!("Response: {:?}", response);

    // oneshot channel to receive response from actor
    let (txo, rxo) = oneshot::channel();
    let msg = ActorMessage::SendMessage { message:    String::from("get"),
                                          respond_to: txo, };
    // send message to actor
    tracing::info!("Sending message to actor.");
    tx.send(msg).await.expect("Message sent.");
    tracing::info!("Message sent to actor successfully.");
    // wait for oneshot response
    tracing::debug!("Waiting for response from actor.");
    let response = rxo.await.expect("Response received.");
    println!("Response: {:?}", response);

    // sleep 3 seconds
    tokio::time::sleep(Duration::from_secs(3)).await;

    Ok(())
}
