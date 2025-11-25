# ACT Chain - EVM Compatibility Guide

## Overview
ACT Chain now supports **Ethereum-compatible transactions and addresses** alongside its native ACT format, enabling seamless integration with Ethereum tools like MetaMask, Web3.js, and Ethers.js.

---

## 🔑 Dual Address Support

### ACT Native Addresses
- **Format**: `ACT-{base58(pubkey_hash)}`
- **Signature**: Ed25519
- **Example**: `ACT-5Hpvh4K7L2Pj9Y3...`

### Ethereum Addresses
- **Format**: `0x{hex(last_20_bytes_of_keccak256)}`
- **Signature**: secp256k1 (ECDSA)
- **Example**: `0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb`

---

## 🔧 Cryptography Support

### Ed25519 (ACT Native)
```rust
use crypto::ActKeyPair;

// Generate ACT keypair
let keypair = ActKeyPair::generate();
println!("Address: {}", keypair.address());

// Sign transaction
let signature = keypair.sign(message);
```

### secp256k1 (Ethereum Compatible)
```rust
use crypto::EthKeyPair;

// Generate Ethereum keypair
let keypair = EthKeyPair::generate()?;
println!("Address: {}", keypair.address());

// Sign transaction with Ethereum signature
let signature = keypair.sign(message)?;
```

---

## 📡 Ethereum-Compatible RPC Methods

### Supported eth_* Methods

#### eth_chainId
Returns ACT Chain ID
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
```
**Response**: `"0xac7"` (2755 decimal)

#### eth_blockNumber
Returns latest block number in hex
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

#### eth_getBalance
Get account balance (works with both ACT and ETH addresses)
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_getBalance",
    "params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb", "latest"],
    "id":1
  }'
```
**Response**: Balance in hex (wei)

#### eth_getTransactionCount
Get account nonce
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_getTransactionCount",
    "params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb", "latest"],
    "id":1
  }'
```

#### eth_sendRawTransaction
Submit signed Ethereum transaction (RLP-encoded)
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_sendRawTransaction",
    "params":["0xf86c..."],
    "id":1
  }'
```

#### eth_call
Execute read-only contract call
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_call",
    "params":[{"to":"0x...","data":"0x..."}, "latest"],
    "id":1
  }'
```

#### net_version
Returns network version (same as chain ID)
```bash
curl -X POST http://107.178.223.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}'
```
**Response**: `"2755"`

---

## 💳 MetaMask Integration

### Connect MetaMask to ACT Chain

1. Open MetaMask
2. Click network dropdown → "Add Network"
3. Enter ACT Chain details:

```
Network Name: ACT Chain
RPC URL: http://107.178.223.1:8545
Chain ID: 2755 (0xAC7)
Currency Symbol: ACT
Block Explorer: http://107.178.223.1:3001
```

4. Click "Save"

### Using MetaMask with ACT Chain

- **Send Transactions**: Use MetaMask's send interface
- **Interact with DApps**: DApps can use `window.ethereum` API
- **View Balance**: MetaMask displays your ACT balance
- **Transaction History**: View on block explorer

---

## 🔄 Transaction Types

### Native ACT Transaction
```rust
TransactionType::Transfer {
    to: "ACT-5Hpvh4K7...".to_string(),
    amount: 1_000_000_000_000_000_000, // 1 ACT
}
```

### Ethereum Legacy Transaction
```rust
TransactionType::EthereumLegacy {
    to: "0x742d35Cc...".to_string(),
    value: 1_000_000_000_000_000_000, // 1 ACT
    data: vec![],
    gas_price: 1_000_000_000, // 1 Gwei
}
```

---

## 📊 Address Conversion

### From Public Key to Address

**ACT Native (Ed25519)**:
```
1. SHA-256(pubkey) → hash
2. Take first 20 bytes
3. Base58 encode → "ACT-{encoded}"
```

**Ethereum (secp256k1)**:
```
1. Keccak256(pubkey) → hash
2. Take last 20 bytes
3. Hex encode → "0x{hex}"
```

---

## 🧪 Testing with Web3.js

```javascript
const Web3 = require('web3');

// Connect to ACT Chain
const web3 = new Web3('http://107.178.223.1:8545');

// Check chain ID
const chainId = await web3.eth.getChainId();
console.log('Chain ID:', chainId); // 2755

// Get balance
const balance = await web3.eth.getBalance('0x742d35Cc...');
console.log('Balance:', web3.utils.fromWei(balance, 'ether'), 'ACT');

// Send transaction
const tx = {
    from: '0x742d35Cc...',
    to: '0x123abc...',
    value: web3.utils.toWei('1', 'ether'),
    gas: 21000,
    gasPrice: web3.utils.toWei('1', 'gwei')
};

const signedTx = await web3.eth.accounts.signTransaction(tx, privateKey);
const receipt = await web3.eth.sendSignedTransaction(signedTx.rawTransaction);
console.log('Transaction hash:', receipt.transactionHash);
```

---

## 🔐 Security Considerations

1. **Dual Key Support**: Users can have both ACT and ETH addresses
2. **Signature Verification**: System validates both Ed25519 and secp256k1
3. **Address Validation**: Checks format before processing
4. **Gas Metering**: Ethereum transactions use same gas model as ACT

---

## 🎯 Compatibility Matrix

| Feature | ACT Native | Ethereum Compatible |
|---------|-----------|---------------------|
| Address Format | `ACT-{base58}` | `0x{hex}` |
| Signature | Ed25519 | secp256k1 (ECDSA) |
| Hash Function | SHA-256 | Keccak-256 |
| RPC Methods | `act_*` | `eth_*` + `net_*` |
| Wallet Support | CLI wallet | MetaMask, Hardware |
| Transaction Encoding | JSON | RLP |

---

## 📈 Performance

- **Native ACT**: Ed25519 signatures are ~10x faster
- **Ethereum Compatible**: Full compatibility with existing tools
- **Dual Mode**: Choose based on your needs

---

## 🔮 Future Enhancements

- [ ] EIP-1559 transaction format
- [ ] EIP-2718 typed transactions
- [ ] EIP-712 structured data signing
- [ ] Solidity compiler integration
- [ ] Truffle/Hardhat plugin
- [ ] ENS-style naming service

---

## 🌐 Network Information

**ACT Chain (Mainnet)**
- **Chain ID**: 2755 (0xAC7)
- **Currency**: ACT
- **Decimals**: 18
- **RPC Endpoint**: http://107.178.223.1:8545
- **Explorer**: http://107.178.223.1:3001
- **Block Time**: 30 seconds

---

**Last Updated**: November 25, 2025
**Version**: 1.0.0
