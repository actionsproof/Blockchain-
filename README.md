# ACT Chain - Production Blockchain

**ACT Chain** is a high-performance Proof of Action (PoA) blockchain with Byzantine Fault Tolerance, built in Rust with WebAssembly smart contracts. Currently running on 3 validator nodes in Google Cloud.

## 🚀 Live Production Deployment

**Network Status**: ✅ LIVE on Google Cloud  
**Nodes**: 3 validators (us-central1-a/b/c)  
**Consensus**: Byzantine Fault Tolerant Proof of Action  
**Block Time**: 30 seconds  

### Public Endpoints
- **Node 1**: 107.178.223.1 (RPC: 8545, Explorer: 3001)
- **Node 2**: 34.70.254.28 (RPC: 8545, Explorer: 3001)
- **Node 3**: 34.118.200.106 (RPC: 8545, Explorer: 3001)

## 📦 Architecture

### Core Infrastructure (13,035 lines)
```
├── node/               Main blockchain node (P2P, consensus, block production)
├── consensus/          Byzantine Fault Tolerant PoA (418 lines)
├── runtime/            WASM execution engine with gas metering (344 lines)
├── crypto/             Ed25519 + secp256k1 cryptography (344 lines)
├── storage/            RocksDB persistent storage (361 lines)
├── types/              Core blockchain types
├── state/              Account state management
├── mempool/            Transaction pool with validation
└── rpc/                JSON-RPC 2.0 server (34 methods)
```

### DeFi & Layer 2 Features (11,568 lines)
```
├── act20-token/        ERC-20 compatible token standard
├── dex/                Automated Market Maker DEX
├── bridge/             Cross-chain bridge with fraud proofs
├── act721-nft/         ERC-721 NFT standard
├── defi-lending/       Borrowing/lending protocol
├── layer2-rollup/      Optimistic rollup for L2 scaling
├── oracle-network/     Decentralized price oracles
├── staking/            Validator staking with delegation
├── governance/         On-chain governance with voting
└── explorer/           Block explorer with REST API
```

### Developer Tools
```
├── sdk/                JavaScript SDK (@actchain/sdk)
├── wallet/             BIP-39 HD wallet with Ed25519
└── cli-wallet/         Command-line wallet tool
```

## 🔧 Build & Run

### Prerequisites
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

### Build
```bash
cargo build --release
```

### Run Node
```bash
./target/release/node
```

### Run RPC Server
```bash
./target/release/act-rpc-server
```

### Run Block Explorer
```bash
./target/release/act-explorer
```

## 🎯 Features

### ✅ Production Backend (Phase 11)
- **Storage Layer**: RocksDB with column families, dual indexing
- **Consensus Engine**: BFT with stake-weighted selection, 2/3+ voting
- **Runtime**: WASM execution, gas metering, host functions
- **Crypto**: Ed25519 + secp256k1, full hash suite
- **RPC**: Ethereum-compatible JSON-RPC API
- **Explorer**: Real-time blockchain data API
- **Monitoring**: Health checks, peer tracking, sync status

### ✅ DeFi Ecosystem (Phase 9 & 10)
- **ACT-20 Tokens**: ERC-20 compatible fungible tokens
- **DEX**: Constant product AMM (x * y = k)
- **Bridge**: Cross-chain transfers with 14-day challenge period
- **NFTs**: ERC-721 compatible with metadata
- **Lending**: Over-collateralized borrowing/lending
- **Rollup**: Optimistic L2 with fraud proofs
- **Oracles**: Median price feeds from multiple sources
- **Staking**: 100k ACT minimum, 14-day unbonding
- **Governance**: Token-weighted voting with timelocks

### ✅ Developer Experience
- **SDK**: JavaScript/TypeScript client library
- **RPC**: 34 methods (ACT native + Ethereum compatible)
- **Wallet**: BIP-39 mnemonic with HD derivation
- **CLI**: Command-line wallet for testing
- **Tests**: 35+ unit tests, all passing

## 📊 Technical Specifications

| Attribute | Value |
|-----------|-------|
| **Consensus** | Proof of Action (PoA) + BFT |
| **Block Time** | 30 seconds |
| **Finality** | 2/3+ validator votes |
| **Native Token** | ACT (18 decimals) |
| **Smart Contracts** | WebAssembly (WASM) |
| **Storage** | RocksDB with column families |
| **P2P** | libp2p (gossipsub + mDNS) |
| **Validators** | 3 (production), configurable |
| **Address Format** | ACT-{base58} (native), 0x{hex} (Ethereum) |

## 🔗 API Examples

### JSON-RPC
```bash
# Get latest block
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get account balance
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"act_getBalance","params":["ACT-address"],"id":1}'
```

### Block Explorer API
```bash
# Get latest blocks
curl http://107.178.223.1:3001/api/blocks

# Get network stats
curl http://107.178.223.1:3001/api/stats

# Search by address
curl http://107.178.223.1:3001/api/search/ACT-address
```

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p consensus
cargo test -p storage
cargo test -p runtime

# Run with verbose output
cargo test -- --nocapture
```

## 📚 Documentation

- [PROJECT_STATUS.md](PROJECT_STATUS.md) - Complete project status and roadmap
- [PROJECT_AUDIT.md](PROJECT_AUDIT.md) - Code audit with compilation status
- [PHASE9_SUMMARY.md](PHASE9_SUMMARY.md) - DeFi features documentation
- [PHASE10.md](PHASE10.md) - Advanced DeFi & Layer 2 specification
- [STAKING_DESIGN.md](STAKING_DESIGN.md) - Staking mechanism design
- [GOVERNANCE_DESIGN.md](GOVERNANCE_DESIGN.md) - Governance system design
- [CLI_WALLET.md](CLI_WALLET.md) - Wallet command reference
- [RPC_ACCESS.md](RPC_ACCESS.md) - RPC API documentation

## 🌐 Google Cloud Deployment

### Infrastructure
- **Project**: actionsproof (272404990588)
- **Network**: Custom VPC (poa-blockchain-network)
- **Subnet**: 10.10.0.0/24
- **Regions**: us-central1 (a, b, c)
- **Machine Type**: e2-medium (2 vCPU, 4GB RAM)
- **Disk**: 30 GB pd-standard per node

### Deploy to GCloud
```bash
# SSH into node
gcloud compute ssh poa-node-1 --project=actionsproof --zone=us-central1-a

# Pull latest code
cd ~/Blockchain-
git pull

# Build and restart
. ~/.cargo/env
cargo build --release
pkill -9 node
nohup ./target/release/node > ~/node.log 2>&1 &
```

## 📈 Metrics

- **Total Code**: 13,035 lines
- **Crates**: 21 workspace members
- **Tests**: 35+ passing
- **Compilation Time**: ~25 seconds (release)
- **Binary Size**: 22 MB
- **Dependencies**: 150+ crates

## 🛣️ Roadmap

### ✅ Completed
- Phase 1-8: Core blockchain infrastructure
- Phase 9: DeFi foundation (ACT-20, DEX, SDK)
- Phase 10: Advanced DeFi (Bridge, NFTs, Lending, Rollup)
- Phase 11: Production backend (Storage, Consensus, Runtime, Crypto)

### 🚧 In Progress
- External RPC access configuration
- Load balancer setup
- Monitoring dashboard

### 📋 Planned
- Mobile wallet app
- Web wallet interface
- Blockchain explorer UI
- Performance optimization
- Security audit

## 🤝 Contributing

This is a production blockchain project. For development:

1. Fork the repository
2. Create a feature branch
3. Run tests: `cargo test --workspace`
4. Submit pull request

## 📄 License

[Add your license here]

## 🔗 Links

- **GitHub**: https://github.com/actionsproof/Blockchain-
- **Network Status**: Check PROJECT_STATUS.md
- **Latest Release**: v11.0 (Backend Infrastructure)
