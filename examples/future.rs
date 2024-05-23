//! Example future implementation

use my_redis::boilerplate;
use my_redis::error::Result;
use tokio::net::TcpStream;
use tracing;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
    polled: u64,
}
impl Future for Delay {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        self.polled += 1;
        tracing::trace!(self.polled, "number of times polled");
        let now = Instant::now();
        if now >= self.when {
            println!("Hello world. An Instant of note has passed!");
            tracing::debug!(?self.when, "goal time");
            tracing::debug!(?now, "recorded time of action");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    boilerplate::tracing_subscribe_boilerplate(boilerplate::SubKind::Tracing(String::from(
        "trace",
    )));
    tracing::debug!("Start main.");
    let _code_as_machine = my_async_fn();

    let when = Instant::now() + Duration::from_millis(10);
    let polled = 0;
    let mut future = Delay { when, polled };
    let out = future.await;
    assert_eq!(out, "done");

    Ok(())
}

async fn my_async_fn() -> Result<()> {
    tracing::info!("hello from async");
    let _socket = TcpStream::connect("127.0.0.1:3000").await?;
    tracing::info!("async TCP operation complete");

    Ok(())
}
