# OpenTimestamps Rust Client

Client library and Cli to create and validate timestamp proofs with the OpenTimestamps protocol, written in Rust. 

Based on [rust-opentimestamps](https://github.com/opentimestamps/rust-opentimestamps) and [opentimestamps-client](https://github.com/opentimestamps/opentimestamps-client).

## Command line tool
Build
```shell
cargo install --path ots_cli
```

Run
```shell
ots_cli help
OpenTimestamps - Command line interface

Usage: ots_cli [OPTIONS] <COMMAND>

Commands:
  stamp    Timestamp files
  upgrade  Upgrade remote calendar timestamps to be locally verifiable
  info     Show information on a timestamp
  verify   Verify a timestamp
  help     Print this message or the help of the given subcommand(s)

Options:
      --bitcoin-node <BITCOIN_NODE>          Bitcoin node
      --bitcoin-username <BITCOIN_USERNAME>  Bitcoin username
      --bitcoin-password <BITCOIN_PASSWORD>  Bitcoin password
  -h, --help                                 Print help
  -V, --version                              Print version
```

## Build OTS library 

### Rust
Ots library is compiled by default with `blocking` feature, which enable `reqwest/blocking`.
```shell
cargo build -p ots_core
```
You could get an async version of the OTS library enabling the `async` feature and disable defaults.
```shell
cargo build -p ots_core --features=async --no-default-features
```
### Android kotlin bindings
Build OTS in Android kotlin bindings:
```shell
just bindings-android
```

### Swift bindings
Build OTS in Swift for iPhone and Darwin:
```shell
just swift-ios
just swift-darwin
```

### Wasm
Build OTS in wasm for browser platform:
```shell
cd ./ots_wasm
wasm-pack build --dev
```
An web example is available at `http://localhost:8080/`:
```shell
cd ./ots_wasm/www
npm run start
```
