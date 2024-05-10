use mini_redis::{client, Result};

use crate::boilerplate::tracing_subscribe_boilerplate;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscribe_boilerplate("debug");
    // open connection to mini-redis address
    let mut client = client::connect("127.0.0.1:6379").await?;

    // set key: "hello" with value: "world"
    client.set("hello", "world  fée".into())
          .await?;

    // get key: "hello"
    let result = client.get("hello")
                       .await?;

    println!("value from server: result:{:?}", result);

    Ok(())
}

mod boilerplate {
    use tracing_subscriber::EnvFilter;

    /// Start boilerplate tracing subscriber, with a default log level, when none is specified
    ///
    /// # Note:
    /// - Not optimal.  But has more info than default and allows easily setting a default log level.
    /// - We should return a `Result`, but I don't want to mess with the flow of this repo's tour by adding different error types and consequent handling strategies.
    pub fn tracing_subscribe_boilerplate(default: &str) {
        // region:    --- tracing_subscriber
        let filter = EnvFilter::try_new(std::env::var("RUST_LOG").unwrap_or_else(|_| {
                                                                     default.to_string()
                                                                 })).expect("Valid filter input provided.");

        tracing_subscriber::fmt().with_env_filter(filter)
                                 .with_file(true)
                                 .with_line_number(true)
                                 .with_thread_ids(true)
                                 .with_target(true)
                                 .init();
        // console_subscriber::init();
        // endregion: --- tracing_subscriber

        // Ok(())
    }
}
