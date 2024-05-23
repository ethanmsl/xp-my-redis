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
}
impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world. An Instant of note has passed!");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
enum MainFuture {
    // initialized, never polled
    Sate0,
    //Waiting on `Delay`
    State1(Delay),
    // Future has completed
    Terminated,
}

// type Delay = !; // this is only available in nightly
// impl Future for MainFuture {
//     type Output = ();

//     fn poll(mut self: Pin<&mut Self>, cs: &mut Context<'_>) -> Poll<()> {
//         use MainFuture::*;

//         loop {
//             match *self {
//                 State0 => {
//                     let when = Instant::now() + Duration::from_millis(10);
//                     let future = Delay { when };
//                     *self = State1(future);
//                 }
//                 State1(ref m)
//             }
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    boilerplate::tracing_subscribe_boilerplate(boilerplate::SubKind::Tracing(String::from(
        "trace",
    )));
    tracing::debug!("Start main.");
    let _code_as_machine = my_async_fn();

    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };
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
