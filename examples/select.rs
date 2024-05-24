//! Select and other ways to add/use concurrency

use tokio::sync::oneshot;

use clap::Parser;
#[derive(Parser, Debug)]
#[command(version, about)]
/// Struct info
struct Args {
        // like element to search for in subdomain names
        repetitions: u32,
}

#[tokio::main]
async fn main() {
        let args = Args::parse();

        for _ in 0..args.repetitions {
                let (tx1, rx1) = oneshot::channel();
                let (tx2, rx2) = oneshot::channel();

                tokio::spawn(async {
                        let _ = tx1.send("one");
                });
                tokio::spawn(async {
                        let _ = tx2.send("two");
                });

                tokio::select! {
                        val1 = rx1 => {
                                println!("rx1 completed first with {:?}",val1);
                        }
                        val2 = rx2 => {
                                println!("rx2 completed first with {:?}",val2);
                        }
                }
        }
}
