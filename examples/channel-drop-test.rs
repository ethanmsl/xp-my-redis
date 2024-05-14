//! Nominally if all senders drop then the channel returns `None`.
//! That ... doesn't sound right.
//! I'd expect the channel to persist the value in it until emptied.
//!
//! Testing with a timed-out consumer and producers that peace out.

use std::time::Duration;

use my_redis::boilerplate::tracing_subscribe_boilerplate;
use my_redis::boilerplate::SubKind;

use tokio::sync::mpsc;

// local config
const SLEEP_SECS: u64 = 1;
const MSSGS_PER_WRITER: u32 = 10;

#[tokio::main]
async fn main() {
    tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));

    let (tx, mut rx) = mpsc::channel(32);
    let tx_a = tx.clone();
    let tx_b = tx.clone();
    drop(tx);

    let reader = tokio::spawn(async move {
        while let Some(mssg) = rx.recv().await {
            tracing::debug!(SLEEP_SECS, "Sleeping for: ");
            tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
            tracing::info!(?mssg, "received message");
        }
    });

    let writer_a = tokio::spawn(async move {
        for n in 0..MSSGS_PER_WRITER {
            let mssg = format!("AAAAAAA #{}", n);
            tracing::info!(?mssg, "sending message");
            tx_a.send(mssg).await.expect("Sent or slept.");
        }
        tracing::info!("---- Writer A: Done. ----");
        tracing::info!("---- Writer A: Dropping send channel handle. ----");
        drop(tx_a);
    });
    let writer_b = tokio::spawn(async move {
        for n in 0..MSSGS_PER_WRITER {
            let mssg = format!("BBBBBBA #{}", n);
            tracing::info!(?mssg, "sending message");
            tx_b.send(mssg).await.expect("Sent or slept.");
        }
        tracing::info!("---- Writer B: Done. ----");
        tracing::info!("---- Writer B: Dropping send channel handle. ----");
        drop(tx_b);
    });

    reader.await.unwrap();
    writer_a.await.unwrap();
    writer_b.await.unwrap();
}
