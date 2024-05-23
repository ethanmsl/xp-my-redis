# Justfile (Convenience Command Runner)

# local vars
LOCAL_VAR_EXAMPLE:='yes, I am!'

# rust vars
RUST_LOG:= 'debug'
RUST_BACKTRACE:= '1'
RUSTFLAGS:='--cfg tokio_unstable'

# home_dir := env_var('HOME')
local_root := justfile_directory()
invocd_from := invocation_directory()
invoc_is_root := if invocd_from == local_root { "true" } else { "false" }
## ANSI Color Codes for use with echo command
GRN := '\033[0;32m' # Green
BLU := '\033[0;34m' # Blue
PRP := '\033[0;35m' # Purple
BRN := '\033[0;33m' # Brown
CYN := '\033[0;36m' # Cyan
NC := '\033[0m'     # No Color

# Default, lists commands.
_default:
        @ just --list --unsorted

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
         echo "{{WORDS}}" | nc 127.0.0.1 6142# Initialize repository.

init: && deps-ext
    cargo build    
    cargo doc

# Linting, formatting, typo checking, etc.
check:
    -cargo clippy
    -cargo fmt
    -typos
    -committed

# Auto-fix errors picked up by check.
[confirm]
check-fix:
     typos --exclude 'data/*' --write-changes

# Clean up cargo build artifacts.
[confirm]
teardown:
    cargo clean

# Watch a file: compile & run on changes.
watch file_to_run:
    cargo watch --quiet --clear --exec 'run --quiet --example {{file_to_run}}'

# List dependencies. (This command has dependencies.)
deps-ext:
    @echo "{{CYN}}List of external dependencies for this command runner and repo:"
    xsv table ext_dependencies.csv

