//! Streams, async

use mini_redis::client;
use my_redis::boilerplate;
use tokio_stream::StreamExt;

const SOCKET_STR: &str = "127.0.0.1:6379";
#[tokio::main]
async fn main() -> mini_redis::Result<()> {
        boilerplate::tracing_subscribe_boilerplate(boilerplate::SubKind::Tracing(String::from(
                "trace",
        )));

        // /////////////////

        let mut stream = tokio_stream::iter(&[1, 2, 3]);
        while let Some(v) = stream
                .next()
                .await
        {
                println!("GOT = {:?}", v);
        }

        // //////////////////
        // NOTE: this requires running the **MINI**-Redis Server
        // (my-redis server does not implement the required funcgtionality)

        tokio::spawn(async { publish().await });
        subscribe().await?;

        println!("DONE");
        Ok(())
}

#[tracing::instrument]
async fn publish() -> mini_redis::Result<()> {
        tracing::info!("starting client (publisher)");
        tracing::info!(SOCKET_STR, "connecting to");
        let mut client = client::connect(SOCKET_STR).await?;

        // publishes to the "number" channel
        client.publish("numbers", "lost to time".into())
                .await?;
        client.publish("numbers", "1".into())
                .await?;
        client.publish("numbers", "two".into())
                .await?;
        client.publish("numbers", "3".into())
                .await?;
        client.publish("numbers", "four".into())
                .await?;
        client.publish("numbers", "5".into())
                .await?;
        client.publish("numbers", "six".into())
                .await?;
        Ok(())
}

#[tracing::instrument]
async fn subscribe() -> mini_redis::Result<()> {
        tracing::info!("starting subscriber");
        tracing::info!(SOCKET_STR, "connecting to");
        let client = client::connect(SOCKET_STR).await?;

        // ¿defines "numbers" channel to listen to
        let subscriber = client
                .subscribe(vec!["numbers".to_string()])
                .await?;
        let messages = subscriber
                .into_stream()
                .filter(|msg| match msg {
                        Ok(msg) if msg
                                .content
                                .len()
                                == 1 =>
                        {
                                true
                        }
                        _ => false,
                })
                .map(|msg| {
                        msg.expect("content extraction")
                                .content
                })
                .take(3);

        tokio::pin!(messages);

        while let Some(msg) = messages
                .next()
                .await
        {
                println!("got = {:?}", msg);
        }

        Ok(())
}
