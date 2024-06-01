// //! Blocking interface to async code

// use tokio::net::ToSocketAddrs;
// use tokio::runtime::Runtime;

// use mini_redis::client::Message;

// /// Async client and it's runtime
// struct BlockingClient {
//         inner: mini_redis::client::Client,
//         rt: Runtime,
// }

// impl BlockingClient {
//         fn connect<T: ToSocketAddrs>(addr: T) -> tokio::io::result::Runtime<Runtime> {
//                 let rt = tokio::runtime::Builder::new_current_thread()
//                         .enable_all()
//                         .build();
//                 let inner = rt.block_on(mini_redis::client::Client::connect(addr));
//         }
// }

// fn main() {
//         println!("hello");
// }

fn main() {
    unreachable!("mess of unresolvable references -- guide doesn't make sensible suggestions and I don't want to hunt through mini-redis")
}
