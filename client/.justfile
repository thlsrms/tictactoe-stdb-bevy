
set dotenv-load

debug:
    cargo build

run:
    cargo run

debug-wasm:
    cargo build --target wasm32-unknown-unknown

release:
    cargo build --release --no-default-features -F bevy_static

release-wasm:
    cargo build --release --no-default-features -F bevy_static --target wasm32-unknown-unknown

release-windows:
    cargo build --release --no-default-features -F bevy_static --target x86_64-pc-windows-gnu

release-linux:
    cargo build --release --no-default-features -F bevy_static --target x86_64-unknown-linux-gnu

serve-wasm-client: release-wasm
    RUST_LOG=error wasm-server-runner target/wasm32-unknown-unknown/release/tic-tac-toe.wasm

serve-wasm-client-dbg: debug-wasm
    RUST_LOG=error wasm-server-runner target/wasm32-unknown-unknown/debug/tic-tac-toe.wasm
