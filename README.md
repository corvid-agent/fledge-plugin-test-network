# fledge-plugin-test-network

WASM plugin that tests the `network` capability for [fledge](https://github.com/CorvidLabs/fledge).

Verifies that WASM plugins can make outbound TCP connections when granted `network = true`. Tests TCP to well-known DNS servers, raw HTTP requests, and localhost. Also verifies other capabilities remain blocked.

**Note**: WASI Preview 1 has limited network support. This plugin tests whether the socket API is available (vs returning "Unsupported"), which may require WASI Preview 2 for full functionality.

## Install & Run

```bash
fledge plugins install CorvidLabs/fledge-plugin-test-network
fledge plugins run test-network
```

## Requirements

- [fledge](https://github.com/CorvidLabs/fledge) with WASM runtime support
- `wasm32-wasip1` Rust target: `rustup target add wasm32-wasip1`
