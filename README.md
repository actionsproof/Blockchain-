# Proof of Action Blockchain (Rust/WASM)

This is a minimal scaffold for a WASM-based "Proof of Action" blockchain chain, designed for Google Cloud deployment and rapid prototyping.

## Structure
- `node/` - Main node binary (P2P, mempool, RPC)
- `consensus/` - Consensus logic (PoA voting, proposer selection)
- `runtime/` - WASM runtime and state transition logic
- `crypto/` - Cryptographic utilities (signing, verification)
- `storage/` - Persistent storage (RocksDB)
- `types/` - Shared types (blocks, actions, headers)

## Build

```sh
cargo build --release
```

## Run (example)

```sh
cd node
cargo run --release
```

## Next Steps
- Implement networking, mempool, and consensus logic in `node/`
- Add WASM execution and state management in `runtime/`
- Integrate validator voting and block finality in `consensus/`
- Expand types and storage as needed

## Cloud Deployment
- Use the provided startup script to auto-install Rust, Wasmtime, and run the node on Google Cloud VMs.

---

This scaffold is ready to push to GitHub and iterate for your "every action = block" design.
