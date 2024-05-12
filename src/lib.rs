//! Lib

pub mod boilerplate {
        use console_subscriber;
        use tracing_subscriber::EnvFilter;

        /// Start boilerplate tracing subscriber, with a default log level, when none is specified
        ///
        /// # Note:
        /// - Not optimal.  But has more info than default and allows easily setting a default log level.
        /// - We should return a `Result`, but I don't want to mess with the flow of this repo's tour by adding different error types and consequent handling strategies.
        pub fn tracing_subscribe_boilerplate(kind: SubKind) {
                // region:    --- tracing_subscriber

                match kind {
                        SubKind::Tracing(default) => {
                                let filter = EnvFilter::try_new(
                                        std::env::var("RUST_LOG")
                                                .unwrap_or_else(|_| default.to_string()),
                                )
                                .expect("Valid filter input provided.");

                                tracing_subscriber::fmt()
                                        .with_env_filter(filter)
                                        .with_file(true)
                                        .with_line_number(true)
                                        .with_thread_ids(true)
                                        .with_target(true)
                                        .init();
                        }
                        SubKind::Console => console_subscriber::init(),
                }

                // endregion: --- tracing_subscriber

                // Ok(())
        }

        /// Whether to use typical Tracing subscriber or TokioConsole
        pub enum SubKind {
                Tracing(String),
                Console,
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
                K: Eq + Hash,
                V: Default,
        {
                let mut db = Vec::with_capacity(num_shards);
                for _ in 0..num_shards {
                        db.push(Mutex::new(HashMap::new()));
                }
                Arc::new(db)
        }

        /// Determine which element of a sharded hashmap to use
        /// before making any requests or shard collection
        ///
        pub fn divine_hashmap<K, V>(db: &ShardedDb<K, V>, key: &K) -> usize
        where
                K: Eq + Hash,
                V: Default,
        {
                hash(&key).rem_euclid(db.len())
        }
}
