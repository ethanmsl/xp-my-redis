use std::{
      collections::HashMap,
      sync::{Arc, Mutex},
};

use boilerplate::{tracing_subscribe_boilerplate, SubKind};
use bytes::Bytes;
use mini_redis::Frame;
use my_redis::boilerplate;
use tokio::net::{TcpListener, TcpStream};
use tracing;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
      tracing_subscribe_boilerplate(SubKind::Tracing(String::from("debug")));
      // tracing_subscribe_boilerplate(SubKind::Console);
      tracing::info!("Tracing Subscriber active.");

      // bind "listener" to an address
      tracing::debug!("Binding Listener to ip & port...");
      let listener = TcpListener::bind("127.0.0.1:6379")
            .await
            .expect("Listener binds.");
      tracing::debug!("listener bound.");

      let db = Arc::new(Mutex::new(HashMap::new()));

      loop {
            // The Second item contains the IP and port of the new connection.
            // -- presumably "accept" is "accept if asked, wait otherwise"
            tracing::debug!("Awaiting socket receipt...");
            let (socket, _) = listener.accept().await.expect("Socket acquired.");
            tracing::debug!("'Cloning' Arc.");
            let db = db.clone();
            tracing::debug!("Socket accepted; Spawning thread to process...");
            tokio::spawn(async move {
                  tracing::debug!("Thread for socket processing spawned.");
                  tracing::debug!("Processing socket...");
                  process(socket, db).await;
                  tracing::debug!("Socket processed.");
            });
      }
}

async fn process(socket: TcpStream, db: Db) {
      use mini_redis::Command::{self, Get, Set};

      // "mini_redis specific" Read&Write "frames" instead of working with byte streams
      let mut connection = mini_redis::Connection::new(socket);

      while let Some(frame) = connection.read_frame().await.expect("frame read") {
            tracing::info!("GOT: {:?}", frame);
            let response = match Command::from_frame(frame).expect("Unpoisoned mutex.") {
                  Set(cmd) => {
                        // value stored as Vec<u8>
                        tracing::debug!("Acquiring mutex lock...");
                        let mut db = db.lock().unwrap();
                        tracing::debug!("Mutex lock acquired.");
                        db.insert(cmd.key().to_string(), cmd.value().clone());
                        Frame::Simple("OK".to_string())
                  }
                  Get(cmd) => {
                        let db = db.lock().expect("Unpoisoned mutex.");
                        if let Some(value) = db.get(cmd.key()) {
                              Frame::Bulk(value.clone())
                        } else {
                              Frame::Null
                        }
                  }
                  cmd => unimplemented!("{:?}", cmd),
            };
            // write response to client
            connection
                  .write_frame(&response)
                  .await
                  .expect("Write to client");
      }
}

#[allow(dead_code)]
mod shard_hash {
      use std::{
            collections::HashMap,
            hash::{DefaultHasher, Hash, Hasher},
            sync::{Arc, Mutex},
      };

      type ShardedDb<K, V> = Arc<Vec<Mutex<HashMap<K, V>>>>;

      /// Hash a thing
      /// (paritcularly a string)
      fn hash<T: Hash>(t: &T) -> usize {
            let mut hasher = DefaultHasher::new();
            t.hash(&mut hasher);
            hasher.finish() as usize
      }

      /// Create an Arc-wrapped vector of Mutexed Hashmaps.
      ///
      /// An attempt to decrease contention for HashMap functionality.
      ///
      /// Warn: I'm not clear on how we determine which Hashmap we belong to without going through them all, as written.  Which would seem to defeat the point -- unless reading + locking is that much speedier a process...
      fn new_sharded_db<K, V>(num_shards: usize) -> ShardedDb<K, V>
      where
            K: Eq+Hash,
            V: Default, {
            let mut db = Vec::with_capacity(num_shards);
            for _ in 0..num_shards {
                  db.push(Mutex::new(HashMap::new()));
            }
            Arc::new(db)
      }

      /// Determine which element of a sharded hashmap to use
      /// before making any requests or shard collection
      ///
      /// ## Improvement:
      /// This could produce generic code, by refering to T in A<V<M<H<T,_>>>
      /// , but ... not sure of precise syntax
      pub fn divine_hashmap<K, V>(db: &ShardedDb<K, V>, key: &K) -> usize
      where
            K: Eq+Hash,
            V: Default, {
            hash(&key).rem_euclid(db.len())
      }
}
