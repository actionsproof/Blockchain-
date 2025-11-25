# Phase 9 Deployment Guide

## 🎯 Overview
Deploy Phase 9 features (persistence, ACT-20 tokens, DEX, SDK, monitoring) to all 3 live nodes.

## ✅ Completed Locally
- ✅ Persistence methods for staking/governance
- ✅ ACT-20 token standard (397 lines, 7 tests passing)
- ✅ DEX smart contract (283 lines, 7 tests passing)
- ✅ JavaScript SDK (TypeScript, 1000+ lines)
- ✅ Prometheus metrics + monitoring endpoints

## 📋 Deployment Steps

### 1. Push Code to GitHub
```bash
git push origin main
```

### 2. Deploy to Node 1 (107.178.223.1)
```bash
# SSH to node 1
gcloud compute ssh poa-node-1 --zone=us-central1-a

# Pull latest code
cd ~/Blockchain-
git pull origin main

# Build with new features
cargo build --release

# Stop current node
sudo systemctl stop act-blockchain

# Update binary
sudo cp target/release/node /usr/local/bin/act-node

# Start node
sudo systemctl start act-blockchain

# Verify
sudo systemctl status act-blockchain
sudo journalctl -u act-blockchain -f -n 50

# Test monitoring endpoints
curl http://localhost:8545/metrics
curl http://localhost:8545/stats
curl http://localhost:8545/health

exit
```

### 3. Deploy to Node 2 (34.70.254.28)
```bash
# SSH to node 2
gcloud compute ssh poa-node-2 --zone=us-central1-b

# Pull latest code
cd ~/Blockchain-
git pull origin main

# Build with new features
cargo build --release

# Stop current node
sudo systemctl stop act-blockchain

# Update binary
sudo cp target/release/node /usr/local/bin/act-node

# Start node
sudo systemctl start act-blockchain

# Verify
sudo systemctl status act-blockchain
sudo journalctl -u act-blockchain -f -n 50

# Test monitoring endpoints
curl http://localhost:8545/metrics
curl http://localhost:8545/stats

exit
```

### 3. Deploy to Node 3 (34.118.200.106)
```bash
# SSH to node 3
gcloud compute ssh poa-node-3 --zone=us-central1-c

# Pull latest code
cd ~/Blockchain-
git pull origin main

# Build with new features
cargo build --release

# Stop current node
sudo systemctl stop act-blockchain

# Update binary
sudo cp target/release/node /usr/local/bin/act-node

# Start node
sudo systemctl start act-blockchain

# Verify
sudo systemctl status act-blockchain
sudo journalctl -u act-blockchain -f -n 50

# Test monitoring endpoints
curl http://localhost:8545/metrics
curl http://localhost:8545/stats

exit
```

## 🧪 Testing After Deployment

### Test Metrics Endpoint
```bash
# Should return Prometheus metrics
curl http://107.178.223.1:8545/metrics
curl http://34.70.254.28:8545/metrics
curl http://34.118.200.106:8545/metrics
```

### Test Stats Endpoint
```bash
# Should return JSON with validator count, staked amount, proposals
curl http://107.178.223.1:8545/stats
curl http://34.70.254.28:8545/stats
curl http://34.118.200.106:8545/stats
```

### Test Health Endpoint
```bash
curl http://107.178.223.1:8545/health
curl http://34.70.254.28:8545/health
curl http://34.118.200.106:8545/health
```

### Test SDK
```bash
cd sdk
npm install
npm run build

# Create test script
node examples/basic.ts
```

### Test ACT-20 Token
```bash
# Run tests locally
cargo test -p act20-token

# All 7 tests should pass
```

### Test DEX Contract
```bash
# Run tests locally
cargo test -p dex-contract

# All 7 tests should pass
```

### Test Persistence
```bash
# After deployment, restart a node and verify state persists
gcloud compute ssh poa-node-1 --zone=us-central1-a

# Check current state
curl http://localhost:8545 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"stake_getValidators","params":[],"id":1}'

# Restart node
sudo systemctl restart act-blockchain

# Verify state persisted
curl http://localhost:8545 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"stake_getValidators","params":[],"id":1}'
```

## 📊 Expected Results

### Metrics Endpoint
```
# HELP act_blocks_produced_total Total number of blocks produced
# TYPE act_blocks_produced_total counter
act_blocks_produced_total 850

# HELP act_current_block_number Current block number
# TYPE act_current_block_number gauge
act_current_block_number 850

# HELP act_validator_count Number of active validators
# TYPE act_validator_count gauge
act_validator_count 3

... (30+ more metrics)
```

### Stats Endpoint
```json
{
  "jsonrpc": "2.0",
  "result": {
    "validator_count": 3,
    "total_staked": "300000000000000000000000",
    "proposal_count": 0
  },
  "id": 1
}
```

### Node Logs
```
🌐 RPC server starting on http://0.0.0.0:8545
📡 Available methods:
   ACT Native: (9 methods)
   Ethereum Compatible: (7 methods)
   Staking: (11 methods)
   Governance: (7 methods)
📊 Monitoring endpoints:
   GET /health   - Node health check
   GET /metrics  - Prometheus metrics
   GET /stats    - Node statistics
```

## 🔍 Troubleshooting

### Build Failures
```bash
# Clean build
cargo clean
cargo build --release

# Check Rust version
rustc --version  # Should be 1.75+
```

### Service Issues
```bash
# Check logs
sudo journalctl -u act-blockchain -n 100 --no-pager

# Check port bindings
sudo netstat -tulpn | grep 8545

# Restart service
sudo systemctl restart act-blockchain
```

### Metrics Not Available
```bash
# Verify RPC server started
curl http://localhost:8545/health

# Check if metrics registered
grep "Prometheus" <(sudo journalctl -u act-blockchain -n 1000)
```

## ✅ Verification Checklist

- [ ] All 3 nodes rebuilt with Phase 9 code
- [ ] All 3 nodes started successfully
- [ ] Metrics endpoint returns data on all nodes
- [ ] Stats endpoint returns validator/staking info
- [ ] Health endpoint responds on all nodes
- [ ] Blocks continue to be produced (check block height)
- [ ] SDK builds successfully
- [ ] ACT-20 tests pass (7/7)
- [ ] DEX tests pass (7/7)
- [ ] Persistence methods compile without errors

## 🎉 Phase 9 Complete!

Once all checks pass, Phase 9 is fully deployed:
- ✅ Persistence layer ready
- ✅ Token standard available
- ✅ DEX contracts tested
- ✅ JavaScript SDK built
- ✅ Monitoring active on all nodes

**Next**: Phase 10 - Advanced DeFi features, cross-chain bridges, or production scaling!
