//! Example future implementation

use futures::task;
use my_redis::boilerplate;
use my_redis::error::Result;
use tokio::net::TcpStream;
use tracing;

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
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
            // get handle to task's waker
            let waker = cx.waker().clone();
            let when = self.when;

            // spawn a timer thread.
            // note: while illustrative I assume this would have no efficiency gains
            // unlike some signalling which is handled by outside systems that have no resources to share
            // or that perform light work waiting on 'more' more external systems to signal them in turn
            thread::spawn(move || {
                let now = Instant::now();

                // lol, okay, maybe this is efficient depending on how sleep timers work
                // may hook into something more efficient with parent system
                // (though spawning a task to spawn a thread to spawn a thread to wait on a timer
                // is, ofc, funny -- though also, ofc, meant to be illustrative)
                if now < when {
                    thread::sleep(when - now);
                }
                waker.wake();
            });
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
    let future = Delay { when, polled };
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
// //////////////////////////////////////////////

struct MiniTokio {
    tasks: VecDeque<Task>,
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

impl MiniTokio {
    fn new() -> MiniTokio {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }

    /// Spawn a future onto the mini-tokio instance.
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future));
    }

    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task);
            }
        }
    }
}

// ///////////////////////////////////////////////

#[allow(dead_code)]
/// Example of what the main function is turned into by the asyc'ing of it
mod just_for_illustration {

    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::{Duration, Instant};

    enum MainFuture {
        // initialized; never polled
        State0,
        // waiting on `Delay`
        State1(Delay),
        // future has completed
        Terminated,
    }

    impl Future for MainFuture {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            use MainFuture::*;

            loop {
                match *self {
                    State0 => {
                        // TODO: what's the 10ms spacing for?
                        let when = Instant::now() + Duration::from_millis(10);
                        let polled = 0;

                        let future = Delay { when, polled };
                        *self = State1(future);
                    }
                    State1(ref mut my_future) => match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = Terminated;
                            return Poll::Pending;
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    },
                    Terminated => panic!("future polled after completion"),
                }
            }
        }
    }
}
