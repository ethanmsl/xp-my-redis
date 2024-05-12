


# List of recipes available
default:
        @just --list

# Start Server. Note: blocks shell
serve LOG_LEVEL='debug':
        RUST_LOG={{LOG_LEVEL}} cargo run --bin server

# Run the 'hello-redis' example, writing and requesting a key:value pair.  (Wants a Server to talk to.)
hi LOG_LEVEL='debug':
        RUST_LOG={{LOG_LEVEL}} cargo run --example hello-redis

# Run simple client set & get.
client LOG_LEVEL='debug':
        RUST_LOG={{LOG_LEVEL}} cargo run --bin client

# Run echo server. (Listens for raw bytestreams by TCP and returns them.)
echo-serv LOG_LEVEL='debug':
        RUST_LOG={{LOG_LEVEL}} cargo run --bin echo-server-copy

# Send raw bytes (by TCP) to echo server.
yell *WORDS:
         echo "{{WORDS}}" | nc 127.0.0.1 6142