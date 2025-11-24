# ACT Blockchain - Live RPC Access

## 🌐 Live Endpoints

Your ACT blockchain is running on Google Cloud with public RPC access:

### Node 1 (Primary)
- **External IP**: http://107.178.223.1:8545
- **Internal IP**: http://10.10.0.5:8545
- **Status**: ✅ LIVE & PRODUCING BLOCKS

### Node 2
- **External IP**: http://34.70.254.28:8545
- **Internal IP**: http://10.10.0.6:8545
- **Status**: Building...

### Node 3
- **External IP**: http://34.118.200.106:8545
- **Internal IP**: http://10.10.0.7:8545
- **Status**: Building...

## 🔥 Firewall Configuration

```bash
# Firewall rule created:
gcloud compute firewall-rules create act-blockchain-rpc \
  --allow tcp:8545 \
  --source-ranges 0.0.0.0/0 \
  --target-tags poa-node \
  --description "Allow ACT Blockchain RPC access"
```

## 📡 RPC Methods Available

| Method | Description | Example |
|--------|-------------|---------|
| `act_getBalance` | Get ACT balance | `{"address":"ACT-validator1"}` |
| `act_getAccount` | Get account info | `{"address":"ACT-treasury"}` |
| `act_getNonce` | Get account nonce | `{"address":"ACT-validator1"}` |
| `act_sendTransaction` | Submit transaction | `{"transaction":{...}}` |
| `act_getTransaction` | Get tx by hash | `{"tx_hash":"0x..."}` |
| `act_getPendingTransactions` | Get pending txs | `{"address":"ACT-..."}` |
| `act_getMempoolStatus` | Get mempool stats | `{}` |

## 🧪 Testing from Command Line

### Health Check
```bash
curl http://107.178.223.1:8545/health
```

### Get Balance (JSON-RPC)
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "act_getBalance",
    "params": {
      "address": "ACT-validator1"
    },
    "id": 1
  }'
```

### Get Account Info
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "act_getAccount",
    "params": {
      "address": "ACT-treasury"
    },
    "id": 2
  }'
```

### Get Mempool Status
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "act_getMempoolStatus",
    "params": {},
    "id": 3
  }'
```

## 🔧 Testing from Google Cloud (Always Works)

If external access has issues, you can always test from within Google Cloud:

```bash
# SSH into any node
gcloud compute ssh poa-node-2 --zone=us-central1-b

# Test RPC
curl http://10.10.0.5:8545/health
curl -X POST http://10.10.0.5:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"act_getBalance","params":{"address":"ACT-validator1"},"id":1}'
```

## 🌍 Web3 Integration

### JavaScript (ethers.js compatible)
```javascript
const provider = new ethers.providers.JsonRpcProvider("http://107.178.223.1:8545");

// Get balance
const balance = await provider.send("act_getBalance", [{
  address: "ACT-validator1"
}]);

// Get account
const account = await provider.send("act_getAccount", [{
  address: "ACT-treasury"
}]);
```

### Python
```python
import requests

rpc_url = "http://107.178.223.1:8545"

# Health check
response = requests.get(f"{rpc_url}/health")
print(response.json())

# Get balance
payload = {
    "jsonrpc": "2.0",
    "method": "act_getBalance",
    "params": {"address": "ACT-validator1"},
    "id": 1
}
response = requests.post(rpc_url, json=payload)
print(response.json())
```

## 📊 Genesis Accounts (13M ACT Total)

| Address | Balance | Purpose |
|---------|---------|---------|
| ACT-validator1 | 1,000,000 ACT | Validator node 1 |
| ACT-validator2 | 1,000,000 ACT | Validator node 2 |
| ACT-validator3 | 1,000,000 ACT | Validator node 3 |
| ACT-treasury | 10,000,000 ACT | Treasury/ecosystem fund |

## 🚀 Live Status

Your blockchain is:
- ✅ Producing blocks every 30 seconds
- ✅ Processing transactions via mempool
- ✅ Persisting to RocksDB
- ✅ Broadcasting via P2P (libp2p gossipsub)
- ✅ Accessible via HTTP JSON-RPC on port 8545

## 🔐 Security Notes

- RPC is public (0.0.0.0/0) for development/testing
- For production, restrict source ranges:
  ```bash
  gcloud compute firewall-rules update act-blockchain-rpc \
    --source-ranges YOUR_IP/32
  ```
- Consider adding authentication/API keys for production use
