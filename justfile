


# List of recipes available
default:
        @just --list

# Start Server. Note: blocks shell
serve LOG_LEVEL='debug':
        RUST_LOG={{LOG_LEVEL}} cargo run --bin server

# Run the 'hello-redis' example, writing and requesting a key:value pair.  (Wants a Server to talk to.)
hi LOG_LEVEL='debug':
        RUST_LOG={{LOG_LEVEL}} cargo run --example hello-redis
