# ACT Chain - Project Status & Roadmap

## 🎯 Vision
**ACT Chain** is a custom Proof of Action blockchain with native **ACT** cryptocurrency, designed for maximum compatibility with existing blockchain ecosystems while maintaining full independence.

---

## ✅ Phase 1: Core Infrastructure (COMPLETED)

### 1. P2P Networking ✅
- **Technology**: libp2p
- **Features**:
  - Gossipsub message propagation
  - mDNS peer discovery
  - TCP transport with Noise encryption
  - Yamux multiplexing
- **Status**: Deployed on 3 Google Cloud VMs

### 2. Consensus Layer ✅
- **Type**: Proof of Action (PoA)
- **Features**:
  - 3 validators with round-robin rotation
  - Block proposal every 30 seconds
  - Validator commitment tracking
  - Block height management
- **Status**: Live and producing blocks

### 3. WASM Runtime ✅
- **Technology**: Wasmtime
- **Features**:
  - Action execution engine
  - Gas metering
  - State change tracking
  - Execution logging
- **Status**: Executing actions successfully

### 4. Storage Layer ✅
- **Technology**: RocksDB
- **Features**:
  - Persistent block storage
  - State persistence
  - Latest height tracking
- **Status**: Database active on all 3 VMs

---

## ✅ Phase 2: Native Blockchain Features (JUST COMPLETED)

### 1. Cryptography Module ✅
**File**: `crypto/src/lib.rs`
- ✅ Ed25519 keypair generation
- ✅ Signature creation and verification
- ✅ ACT address format: `ACT-{base58(pubkey_hash)}`
- ✅ SHA-256 hashing utilities

### 2. Native Currency (ACT) ✅
**File**: `types/src/lib.rs`
- ✅ ActAmount type (u128, 18 decimals)
- ✅ 1 ACT = 10^18 smallest units
- ✅ Account balance management
- ✅ Reward system in ACT tokens

### 3. Transaction System ✅
**File**: `types/src/lib.rs`
- ✅ Transaction types:
  - Transfer (ACT token transfers)
  - ContractDeploy (WASM contract deployment)
  - ContractCall (Contract interaction)
- ✅ Transaction structure with signature
- ✅ Gas limit and gas price fields
- ✅ Nonce-based replay protection

### 4. Wallet System ✅
**File**: `wallet/src/lib.rs`
- ✅ ACT wallet generation
- ✅ BIP-39 mnemonic support (12-word phrases)
- ✅ Wallet restoration from mnemonic
- ✅ Transaction signing
- ✅ Watch-only wallet support
- ✅ Unit conversion utilities

### 5. Account Model ✅
**File**: `types/src/lib.rs`
- ✅ Account structure:
  - ACT address
  - Balance (in ACT)
  - Nonce
  - Code hash (for contracts)
  - Storage root (for contract state)
- ✅ Contract vs. EOA (Externally Owned Account) distinction

---

## 🚧 Phase 3: State Manager and Transaction Processing (IN PROGRESS)

### ✅ Completed Components

#### 1. State Manager Module (`state/`)
**File**: `state/src/lib.rs`
- ✅ StateManager with RocksDB backend
- ✅ Account state management (balance, nonce, code, storage)
- ✅ State transitions with persistence
- ✅ Genesis block initialization
- ✅ Pre-funded genesis accounts
- ✅ Gas tracking (GasTracker with used/limit)
- ✅ Transaction validation (nonce, balance, gas)
- ✅ Account balance operations (get/update/transfer)

#### 2. Mempool (Transaction Pool) (`mempool/`)
**File**: `mempool/src/lib.rs`
- ✅ Transaction pool with hash-based indexing
- ✅ Pending transaction queue per address
- ✅ Transaction validation (signature, nonce, balance, gas)
- ✅ Gas price-based priority ordering
- ✅ Transaction selection for block inclusion
- ✅ Mempool size limits and management
- ✅ Mempool statistics (total tx, unique senders, avg gas price)

#### 3. Enhanced Crypto Module
**Updates**: `crypto/src/lib.rs`
- ✅ Serializable ActKeyPair (custom serde for SigningKey)
- ✅ Ed25519 keypair with proper entropy generation
- ✅ Wallet-compatible key storage

#### 4. Deployment Status
- ✅ All code committed to GitHub (commit 699982b)
- ✅ poa-node-1 (107.178.223.1, us-central1-a): ✅ BUILT
- ✅ poa-node-2 (34.70.254.28, us-central1-b): ✅ BUILT
- ✅ poa-node-3 (34.118.200.106, us-central1-c): ✅ BUILT

### 🔄 Phase 3 Remaining Work

#### 1. Node Integration (Priority)
- [ ] Integrate StateManager into `node/src/main.rs`
  - Initialize with genesis accounts
  - Pass to consensus module
- [ ] Integrate Mempool into node
  - Add transaction reception from gossipsub
  - Validate and add to mempool
- [ ] Update Consensus to use Mempool
  - Select transactions for block proposal
  - Execute transactions and update state
  - Persist state after each block
- [ ] Transaction Broadcasting
  - Gossipsub topic for transactions
  - Broadcast validated transactions to peers
  - Handle incoming transaction messages

#### 2. Testing & Validation
- [ ] Test genesis block creation
- [ ] Test transaction validation flow
- [ ] Test mempool priority ordering
- [ ] Test state persistence across restarts
- [ ] Test multi-node transaction propagation

#### 3. Fee Distribution (Future Enhancement)
- [ ] Calculate block rewards (base + fees)
- [ ] Distribute fees to validators
- [ ] Treasury allocation mechanism

---

## 📋 Phase 4: Enhanced Features (UPCOMING)

### 1. Native Smart Contract System
- [ ] Contract deployment via transactions
- [ ] Contract state management
- [ ] Contract-to-contract calls
- [ ] Events and logs
- [ ] Gas metering for contracts

### 2. RPC Server
- [ ] JSON-RPC endpoint for wallets
- [ ] Query balance
- [ ] Send transaction
- [ ] Get block info
- [ ] Get transaction receipt

### 3. Block Explorer Backend
- [ ] REST API for block data
- [ ] Transaction history
- [ ] Account lookup
- [ ] Contract verification

### 4. CLI Wallet Tool
- [ ] Create wallet
- [ ] Import/export wallet
- [ ] Send ACT
- [ ] Check balance
- [ ] Deploy contracts

---

## 🌐 Phase 5: Multi-Chain Compatibility (FUTURE)

### 1. EVM Compatibility Layer
- [ ] secp256k1 signature support (Ethereum keys)
- [ ] Ethereum address format (0x...)
- [ ] RLP transaction encoding
- [ ] EVM runtime in WASM
- [ ] eth_* JSON-RPC methods
- [ ] MetaMask integration

### 2. Bitcoin Compatibility
- [ ] Bitcoin address format
- [ ] UTXO model simulation
- [ ] Bitcoin transaction format
- [ ] BTC-style signatures

### 3. Solana Compatibility
- [ ] Solana address format (Base58)
- [ ] Solana transaction format
- [ ] BPF runtime support
- [ ] Phantom wallet integration

### 4. Multi-Address Support
- [ ] ACT-native: `ACT-...`
- [ ] EVM-style: `0x...`
- [ ] BTC-style: `1...` or `bc1...`
- [ ] SOL-style: Base58

---

## 🏗️ Current Architecture

```
┌─────────────────────────────────────┐
│       ACT Blockchain Node            │
├─────────────────────────────────────┤
│  P2P (libp2p) │ Consensus (PoA)     │
├─────────────────────────────────────┤
│  WASM Runtime │ Storage (RocksDB)   │
├─────────────────────────────────────┤
│  Crypto       │ Wallet              │
├─────────────────────────────────────┤
│  Transactions │ Native ACT Currency │
└─────────────────────────────────────┘
```

## 📦 Codebase Structure

```
actionsproof-g/
├── node/          # P2P networking, main entry point
├── consensus/     # PoA consensus engine
├── runtime/       # WASM execution engine
├── storage/       # RocksDB persistence
├── crypto/        # ACT addresses, signing, verification
├── types/         # Transactions, blocks, accounts
└── wallet/        # ACT wallet with BIP-39
```

## 🚀 Deployment

**Infrastructure**: Google Cloud Platform
- **VMs**: 3 nodes (poa-node-1, poa-node-2, poa-node-3)
- **Zones**: us-central1-a, us-central1-b, us-central1-c
- **Status**: ✅ Live and producing blocks
- **Repository**: `actionsproof/Blockchain-`

---

## 🎯 Next Immediate Steps

1. **Build State Manager** (accounts, balances, nonces)
2. **Genesis Block** (initial ACT distribution)
3. **Gas System** (transaction fees)
4. **Mempool** (transaction queue)
5. **RPC Server** (wallet API)

---

## 💡 Key Decisions Made

- **Native Currency**: ACT (18 decimals)
- **Address Format**: `ACT-{base58}` (unique to ACT Chain)
- **Signature Scheme**: Ed25519 (native)
- **Account Model**: Account-based (like Ethereum, not UTXO)
- **Smart Contracts**: WASM-based
- **Consensus**: Proof of Action (PoA)
- **Block Time**: 30 seconds

---

## 🔗 Resources

- **GitHub**: https://github.com/actionsproof/Blockchain-
- **Live Nodes**: 3 VMs on Google Cloud
- **Tech Stack**: Rust + WASM + RocksDB + libp2p

---

**Last Updated**: November 24, 2025
**Current Phase**: Phase 3 - Account State Manager
