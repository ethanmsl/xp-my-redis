//! main blocking, running async code

use my_redis::boilerplate;
use tokio::runtime::Builder;
use tokio::time::{sleep, Duration};

fn main() -> my_redis::error::Result<()> {
        boilerplate::tracing_subscribe_boilerplate(boilerplate::SubKind::Tracing(
                "debug".to_string(),
        ));

        // gen a runtime with a *single* thread
        let runtime = Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("tokio runtime constructed");

        // spawn tasks on a shared tokio runtime (so one shared thread in this case)
        let mut handles = Vec::with_capacity(10);
        for i in 0..10 {
                handles.push(runtime.spawn(my_bg_task(i)));
        }

        std::thread::sleep(Duration::from_millis(750));
        println!("Finished time-consuming task");

        // wait for all tasks to complete
        for handle in handles {
                // awaiting (via block_on) the join_handles (futures) of our tasks
                runtime.block_on(handle)
                        .expect("task completion");
        }
        Ok(())
}

async fn my_bg_task(i: u64) {
        let millis = 1000 - 50 * i;
        println!("Task {} sleeping for {} ms", i, millis);

        sleep(Duration::from_millis(millis)).await;
        println!("Task {} stopping", i);
}
