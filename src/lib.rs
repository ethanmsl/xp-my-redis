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
                              std::env::var("RUST_LOG").unwrap_or_else(|_| default.to_string()),
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
