# acme-bot-remote

### Installation

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
```

### Running

```bash
npm install --include=dev
trunk serve
```

### Release

```bash
npm install --include=dev
trunk build --release
```
