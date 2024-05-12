use mini_redis::{client, Result};
use my_redis::boilerplate;

use crate::boilerplate::{tracing_subscribe_boilerplate, SubKind};

#[tokio::main]
async fn main() -> Result<()> {
        tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
        // tracing_subscribe_boilerplate(SubKind::Console);

        // open connection to mini-redis address
        let mut client = client::connect("127.0.0.1:6379").await?;

        // set key: "hello" with value: "world"
        client.set("hello", "world  fée".into()).await?;

        // get key: "hello"
        let result = client.get("hello").await?;

        println!("value from server: result:{:?}", result);

        Ok(())
}
