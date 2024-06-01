//! Nominally if all senders drop then the channel returns `None`.
//! That ... doesn't sound right.
//! I'd expect the channel to persist the value in it until emptied.
//!
//! Testing with a timed-out consumer and producers that peace out.

use std::time::Duration;

use tokio::sync::mpsc;

// local config
const SLEEP_SECS: u64 = 1;
const MSSGS_PER_WRITER: u32 = 6;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx_a = tx.clone();
    let tx_b = tx.clone();
    drop(tx);

    let reader = tokio::spawn(async move {
        while let Some(mssg) = rx.recv().await {
            println!("Reader: Sleeping for: {:?}", SLEEP_SECS);
            tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
            println!("Reader: Received message: <--- {}", mssg);
        }
    });

    let writer_a = tokio::spawn(async move {
        for n in 0..MSSGS_PER_WRITER {
            let mssg = format!("AAAAAAA #{}", n);
            println!("Writer A: Sending message: ---> {}", mssg);
            tx_a.send(mssg).await.expect("Sent or slept.");
        }
        println!("---- Writer A: Done. ----")
    });
    let writer_b = tokio::spawn(async move {
        for n in 0..MSSGS_PER_WRITER {
            let mssg = format!("BBBBBBA #{}", n);
            println!("Writer B: Sending message: ---> {}", mssg);
            tx_b.send(mssg).await.expect("Sent or slept.");
        }
        println!("---- Writer B: Done. ----")
    });

    reader.await.unwrap();
    writer_a.await.unwrap();
    writer_b.await.unwrap();
}
