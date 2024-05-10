use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
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
