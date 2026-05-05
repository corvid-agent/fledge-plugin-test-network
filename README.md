# fledge-plugin-test-network

WASM test plugin for the [fledge](https://github.com/CorvidLabs/fledge) `network` capability.

## What it tests

Verifies that WASM plugins can make outbound TCP connections when granted `network = true` in `plugin.toml`. Tests the socket API availability through WASI, and confirms that other capabilities remain blocked.

**Note**: WASI Preview 1 (`wasm32-wasip1`) does not expose a native socket API. The `network = true` capability calls `inherit_network()` on the WASI context, but actual TCP connections require WASI Preview 2 or a custom `fledge::http` host import. This plugin detects and reports the distinction between "unsupported" (WASI P1 limitation) and actual connection errors.

Runs the following test cases:

- **TCP to Google DNS** -- outbound TCP to `8.8.8.8:53`
- **TCP to Cloudflare** -- outbound TCP to `1.1.1.1:443`
- **Raw HTTP via TCP** -- sends an HTTP GET to `example.com:80` over raw TCP
- **Localhost** -- tests loopback connection to `127.0.0.1`
- **Negative tests** -- filesystem and process spawn are blocked (only `network` is granted)

## Capability exercised

```toml
[capabilities]
exec = false
store = false
metadata = false
filesystem = "none"
network = true
```

## Install and run

```bash
fledge plugins install corvid-agent/fledge-plugin-test-network
fledge plugins run test-network
```

## Build from source

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

The compiled WASM binary is written to `target/wasm32-wasip1/release/test-network.wasm`.
