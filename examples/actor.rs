//! actor example using tokio

use actor::*;
use my_redis::{boilerplate::{tracing_subscribe_boilerplate, SubKind},
               error::Result};
use rand::seq::SliceRandom;
use tokio::sync::{mpsc::{self, Sender},
                  oneshot};

// //////////////////////////////////// //
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));

    // overly complicated `new()` 1, with supplied channel:
    {
        tracing::info!("Starting actor example 1 -- where tx & rx are externally generated and provided to actor and functions.");
        let (tx, rx) = mpsc::channel::<ActorMessage>(32); // number is buffer size
        let (_, mut yorrick) = MyActor::new(Some(rx));

        // use tokio spawn to run MyActor and have it wait to receive messages
        // we don't take a yorrick_handle here; using our mpsc & oneshots solely
        let _ = tokio::spawn(async move {
            while let Some(msg) = yorrick.receiver.recv().await {
                tracing::debug!("Received message: {:?}", msg);
                yorrick.handle_message(msg)
                       .await
                       .expect("Message handling failed.");
            }
            tracing::warn!("Actor has stopped receiving messages.");
        });

        let yorrick_tx = tx;

        send_random_messages(yorrick_tx.clone(), 10).await;
        let resp = send_message(yorrick_tx.clone(), "get").await;
        println!("Response to final \"get\": {:?}", resp);
    }

    // overly complicated `new()` 2, with generated channels:
    {
        tracing::info!("Starting actor example 2 -- where actor and actorhandle are generated with tx & rx embedded within.");
        let (Some(alas), mut yorrick) = MyActor::new(None) else {
            panic!("Failed to create actor-handle.");
        };

        // *unconnected to our NewTyping ot tx, we also make a method to spawn a task for the actor here
        yorrick.off_on_my_own()?;

        alas.send_random_messages(10).await;
        let resp = alas.send_message("get").await;
        println!("Response to final \"get\": {:?}", resp);
    }
    Ok(())
}

// ////////////////////////////////////// //

mod actor {
    use super::*;

    /// Receiver component to listen in on
    #[derive(Debug)]
    pub struct MyActor {
        pub receiver:   mpsc::Receiver<ActorMessage>,
        pub actor_info: Option<u32>,
    }

    /// Message to send to an actor
    ///
    /// Should contaain message *for* actor
    /// and sending channel for actor to *respond* with
    /// Generator should hold on to receiver end, ofc.
    ///
    /// Alternate Name: Actor_Internal_State
    #[derive(Debug)]
    pub enum ActorMessage {
        SendMessage {
            message:    String,
            respond_to: oneshot::Sender<Option<u32>>,
        },
    }

    /// Handle to for external interface with actor
    ///
    /// NewType for transmitting channel to an actor
    #[derive(Debug, Clone)]
    pub struct MyActorHandle {
        tx: Sender<ActorMessage>,
    }

    impl MyActorHandle {
        pub fn new(tx: Sender<ActorMessage>) -> Self {
            MyActorHandle { tx }
        }

        pub async fn send_message(&self, message: impl Into<String>) -> Result<Option<u32>> {
            // convert to String, if necessary
            let message = message.into();

            // oneshot channel to receive response from actor
            let (txo, rxo) = oneshot::channel();
            let msg = ActorMessage::SendMessage { message,
                                                  respond_to: txo };

            tracing::trace!("Sending message to actor.");
            self.tx.send(msg).await?;

            tracing::trace!("Waiting for response from actor.");
            Ok(rxo.await?)
        }

        pub async fn send_random_messages(&self, n: u32) {
            // randomly select messages to send to actor
            let mut rng = rand::thread_rng();
            let messages = ["increase", "get"];
            for i in 0..n {
                let &message = messages.choose(&mut rng)
                                       .expect("Slice chosen over should not be empty.");
                let resp = self.send_message(message).await;
                println!("Response {}: {:?}", i + 1, resp);
            }
        }
    }

    impl MyActor {
        /// Create a new actor, passing in a receiver
        ///
        /// **Overly Complicated** `new()`, but trying
        /// out how it feels
        ///
        /// returning a (Option<Handle>, Self)
        /// feels too obtuse, however
        pub fn new(receiver: Option<mpsc::Receiver<ActorMessage>>)
                   -> (Option<MyActorHandle>, Self) {
            if let Some(rx) = receiver {
                (None,
                 MyActor { receiver:   rx,
                           actor_info: Some(0), })
            } else {
                let (tx, rx) = mpsc::channel::<ActorMessage>(32);
                (Some(MyActorHandle::new(tx)),
                 MyActor { receiver:   rx,
                           actor_info: Some(0), })
            }
        }

        /// Take ActorMessage, opreate, and respond
        ///
        /// # NOTE:
        /// The Actor **must** respond.
        /// **OR** the receiver must be set up to deal with the Error
        /// of a dropped channel.
        pub async fn handle_message(&mut self, msg: ActorMessage) -> Result<()> {
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

        async fn run_my_actor(mut self) -> Result<()> {
            while let Some(msg) = self.receiver.recv().await {
                tracing::debug!("Received message: {:?}", msg);
                self.handle_message(msg).await?;
            }
            Ok(())
        }

        pub fn off_on_my_own(self) -> Result<()> {
            tokio::spawn(async move {
                tracing::info!("(Woo! Spawned from function!)");
                let _result = self.run_my_actor().await;
                tracing::warn!("Actor has stopped receiving messages.");
            });
            Ok(())
        }
    }

    pub async fn send_message(tx: mpsc::Sender<ActorMessage>,
                              message: impl Into<String>)
                              -> Result<Option<u32>> {
        // convert to String, if necessary
        let message = message.into();

        // oneshot channel to receive response from actor
        let (txo, rxo) = oneshot::channel();
        let msg = ActorMessage::SendMessage { message,
                                              respond_to: txo };

        tracing::trace!("Sending message to actor.");
        tx.send(msg).await?;

        tracing::trace!("Waiting for response from actor.");
        Ok(rxo.await?)
    }

    pub async fn send_random_messages(tx: Sender<ActorMessage>, n: u32) {
        // randomly select messages to send to actor
        let mut rng = rand::thread_rng();
        let messages = ["increase", "get"];
        for i in 0..n {
            let &message = messages.choose(&mut rng)
                                   .expect("Slice chosen over should not be empty.");
            let resp = send_message(tx.clone(), message).await;
            println!("Response {}: {:?}", i + 1, resp);
        }
    }
}
