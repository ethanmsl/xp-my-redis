//! Example future implementation
use futures::task::{self, ArcWake};

use my_redis::boilerplate;
use my_redis::error::Result;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tracing;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

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

        // with idiomatic(ish?) Notify and delay() fn
        // not structurally related to struct `Delay`
        delay(Duration::from_millis(1000)).await;

        thread::sleep(Duration::from_millis(2000));
        Ok(())
}

/// NOTE: this is separate from struct `Delay`
/// serves as a higherlevel, as used / idiomatic
/// implementation of similar functionality
/// (vs didactic implementation using struct `Delay`)
#[tracing::instrument]
async fn delay(dur: Duration) {
        tracing::debug!(?dur, "starting delay() for dur value: ");
        let when = Instant::now();
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        thread::spawn(move || {
                tracing::debug!("spawning wait thread");
                let now = Instant::now();
                if now < when {
                        thread::sleep(when - now);
                }

                tracing::trace!("providing notification permit...");
                notify_clone.notify_one();
                tracing::trace!("provided notification permit");
        });

        tracing::debug!("awaiting notification permit");
        notify.notified()
                .await;
        tracing::info!("notification permit received!");
}

struct Delay {
        when: Instant,
        polled: u64,
        // waker: Option<Arc<Mutex<Waker>>>, // <-- not adding this, as it would require changing the running code
        //                                      same with the `impl Future for Delay`, but it's an interesting section
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
                        let waker = cx
                                .waker()
                                .clone();
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
                        cx.waker()
                                .wake_by_ref();
                        Poll::Pending
                }
        }
}

async fn my_async_fn() -> Result<()> {
        tracing::info!("hello from async");
        let _socket = TcpStream::connect("127.0.0.1:3000").await?;
        tracing::info!("async TCP operation complete");

        Ok(())
}
// //////////////////////////////////////////////

#[allow(dead_code)]
struct MiniTokio {
        scheduled: mpsc::Receiver<Arc<Task>>,
        sender: mpsc::Sender<Arc<Task>>,
}

#[allow(dead_code)]
struct Task {
        // mutex is not used in real tokio in whatever the here equivalent is
        // as only on thread would be accessingTaskFuture
        task_future: Mutex<TaskFuture>,
        executor: mpsc::Sender<Arc<Task>>,
}
#[allow(dead_code)]
impl Task {
        fn poll(self: Arc<Self>) {
                // Create a waker from the `Task` instance. This
                // uses the `ArcWake` impl from above.
                let waker = task::waker(self.clone());
                let mut cx = Context::from_waker(&waker);

                // No other thread ever tries to lock the task_future
                let mut task_future = self
                        .task_future
                        .try_lock()
                        .unwrap();

                // Poll the inner future
                task_future.poll(&mut cx);
        }

        // Spawns a new task with the given future.
        //
        // Initializes a new Task harness containing the given future and pushes it
        // onto `sender`. The receiver half of the channel will get the task and
        // execute it.
        fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
        where
                F: Future<Output = ()> + Send + 'static,
        {
                let task = Arc::new(Task {
                        task_future: Mutex::new(TaskFuture::new(future)),
                        executor: sender.clone(),
                });

                let _ = sender.send(task);
        }
        fn schedule(self: &Arc<Self>) {
                self.executor
                        .send(self.clone())
                        .expect("Task send successful");
        }
}
impl ArcWake for Task {
        fn wake_by_ref(arc_self: &Arc<Self>) {
                arc_self.schedule();
        }
}

/// A syructure holding a future and the result of the latest `poll` call
struct TaskFuture {
        future: Pin<Box<dyn Future<Output = ()> + Send>>,
        poll: Poll<()>,
}

#[allow(dead_code)]
impl MiniTokio {
        fn new() -> MiniTokio {
                let (sender, scheduled) = mpsc::channel();
                MiniTokio { scheduled, sender }
        }

        /// Spawn a future onto the mini-tokio instance.
        fn spawn<F>(&mut self, future: F)
        where
                F: Future<Output = ()> + Send + 'static,
        {
                Task::spawn(future, &self.sender);
        }

        fn run(&mut self) {
                while let Ok(task) = self
                        .scheduled
                        .recv()
                {
                        task.poll();
                }
        }
}

impl TaskFuture {
        fn new(future: impl Future<Output = ()> + Send + 'static) -> TaskFuture {
                TaskFuture {
                        future: Box::pin(future),
                        poll: Poll::Pending,
                }
        }

        fn poll(&mut self, cx: &mut Context<'_>) {
                // spurious wake-ups allowed
                // even after `Ready`
                // polling after `Ready` is **not** allowed
                if self.poll
                        .is_pending()
                {
                        self.poll = self
                                .future
                                .as_mut()
                                .poll(cx);
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
                                                let when =
                                                        Instant::now() + Duration::from_millis(10);
                                                let polled = 0;

                                                let future = Delay { when, polled };
                                                *self = State1(future);
                                        }
                                        State1(ref mut my_future) => {
                                                match Pin::new(my_future).poll(cx) {
                                                        Poll::Ready(out) => {
                                                                assert_eq!(out, "done");
                                                                *self = Terminated;
                                                                return Poll::Pending;
                                                        }
                                                        Poll::Pending => {
                                                                return Poll::Pending;
                                                        }
                                                }
                                        }
                                        Terminated => panic!("future polled after completion"),
                                }
                        }
                }
        }
}
