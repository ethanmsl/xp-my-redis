//! Example future implementation

use my_redis::boilerplate;
use my_redis::error::Result;
use tokio::net::TcpStream;
use tracing;

#[tokio::main]
async fn main() -> Result<()> {
    boilerplate::tracing_subscribe_boilerplate(boilerplate::SubKind::Tracing(String::from(
        "trace",
    )));
    tracing::debug!("Start main.");

    let _code_as_machine = my_async_fn();

    Ok(())
}
async fn my_async_fn() -> Result<()> {
    tracing::info!("hello from async");
    let _socket = TcpStream::connect("127.0.0.1:3000").await?;
    tracing::info!("async TCP operation complete");

    Ok(())
}
