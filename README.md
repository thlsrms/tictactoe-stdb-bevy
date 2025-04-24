

## 🛠️ WebAssembly Build

To compile the client for the web, use:

```bash
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --target wasm32-unknown-unknown
```


![Demo](media/tictactoe.gif)
