# ACT Wallet CLI - User Guide

**ACT Wallet** is a command-line tool for managing ACT tokens and interacting with the ACT blockchain.

---

## 🚀 Installation

### Build from Source

```bash
cd actionsproof-g
cargo build --release -p cli-wallet
```

The binary will be located at `target/release/act-wallet` (or `act-wallet.exe` on Windows).

### Add to PATH (Optional)

**Linux/macOS:**
```bash
sudo cp target/release/act-wallet /usr/local/bin/
```

**Windows (PowerShell as Administrator):**
```powershell
Copy-Item target\release\act-wallet.exe C:\Windows\System32\
```

---

## 📖 Quick Start

### 1. Create a New Wallet

```bash
act-wallet create
```

**What happens:**
- Generates a new Ed25519 keypair
- Creates a 12-word BIP-39 recovery phrase
- Encrypts and saves wallet to `~/.act-wallet/default.json`
- Displays your ACT address (starts with `ACT-`)

**Example Output:**
```
🔐 Creating new ACT wallet...

Enter password to encrypt wallet: ********
Confirm password: ********
✅ Wallet created successfully!

Your wallet address:
  ACT-GEHsFmUCDAcuRxTjJ9F9aqPyh43ZWaxEiEfbpn8NYX4Q

Your recovery phrase (write this down!):

  abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

⚠️  Keep your recovery phrase safe! It's the only way to restore your wallet.

Wallet saved to: /home/user/.act-wallet/default.json
```

### 2. Check Balance

```bash
act-wallet balance
```

**With detailed account info:**
```bash
act-wallet balance --detailed
```

**Example Output:**
```
Enter wallet password: ********

💰 Account Balance

Address: ACT-GEHsFmUCDAcuRxTjJ9F9aqPyh43ZWaxEiEfbpn8NYX4Q
Balance: 1000000.000000 ACT

Detailed Account Information:
  Nonce: 0
```

### 3. Send ACT Tokens

```bash
act-wallet send --to ACT-recipient... --amount 100.5
```

**With custom gas:**
```bash
act-wallet send \
  --to ACT-9iKSwKSdeLrLnGNVWqfWxCQXYEpVm8jbwh7iiiqe3S7B \
  --amount 50.0 \
  --gas-limit 21000 \
  --gas-price 2000000000
```

**Example Output:**
```
Enter wallet password: ********

📤 Sending ACT Transaction

From:      ACT-GEHsFmUCDAcuRxTjJ9F9aqPyh43ZWaxEiEfbpn8NYX4Q
To:        ACT-9iKSwKSdeLrLnGNVWqfWxCQXYEpVm8jbwh7iiiqe3S7B
Amount:    50.000000 ACT
Gas Limit: 21000
Gas Price: 2000000000
Nonce:     0

Confirm transaction? (yes/no): yes

⏳ Broadcasting transaction...

✅ Transaction sent successfully!

Transaction hash: 0x1234567890abcdef...
```

### 4. Deploy WASM Contract

```bash
act-wallet deploy --wasm contract.wasm
```

**With initial balance:**
```bash
act-wallet deploy \
  --wasm my_contract.wasm \
  --value 10.0 \
  --gas-limit 1000000 \
  --gas-price 1000000000
```

**Example Output:**
```
Enter wallet password: ********

📦 Deploying WASM Contract

From:      ACT-GEHsFmUCDAcuRxTjJ9F9aqPyh43ZWaxEiEfbpn8NYX4Q
WASM File: my_contract.wasm
Size:      45678 bytes
Value:     10.000000 ACT
Gas Limit: 1000000
Gas Price: 1000000000
Nonce:     1

Confirm deployment? (yes/no): yes

⏳ Broadcasting contract deployment...

✅ Contract deployed successfully!

Transaction hash: 0xabcdef1234567890...

Note: Contract address will be derived from your address and nonce.
```

---

## 🔧 Commands

### `create` - Create New Wallet

Create a new wallet with a randomly generated keypair and mnemonic.

```bash
act-wallet create [OPTIONS]
```

**Options:**
- `--name <NAME>` - Wallet name (default: "default")

**Examples:**
```bash
act-wallet create
act-wallet create --name my-wallet
```

---

### `import` - Import from Mnemonic

Restore a wallet from a 12-word BIP-39 recovery phrase.

```bash
act-wallet import [OPTIONS]
```

**Options:**
- `--name <NAME>` - Wallet name (default: "default")

**Example:**
```bash
act-wallet import --name restored-wallet
```

You'll be prompted to enter your 12-word recovery phrase.

---

### `balance` - Check Balance

Query your wallet balance from the blockchain.

```bash
act-wallet balance [OPTIONS]
```

**Options:**
- `--detailed` - Show full account information (nonce, contract code, storage)
- `--rpc <URL>` - Custom RPC endpoint (default: http://107.178.223.1:8545)
- `--wallet <PATH>` - Custom wallet file path

**Examples:**
```bash
act-wallet balance
act-wallet balance --detailed
act-wallet balance --rpc http://34.70.254.28:8545
act-wallet balance --wallet ~/.act-wallet/my-wallet.json
```

---

### `send` - Send ACT Tokens

Transfer ACT tokens to another address.

```bash
act-wallet send --to <ADDRESS> --amount <ACT> [OPTIONS]
```

**Required:**
- `--to <ADDRESS>` - Recipient ACT address (starts with `ACT-`)
- `--amount <ACT>` - Amount in ACT (e.g., 10.5)

**Options:**
- `--gas-limit <LIMIT>` - Gas limit (default: 21000)
- `--gas-price <PRICE>` - Gas price in smallest units (default: 1000000000)
- `--rpc <URL>` - Custom RPC endpoint
- `--wallet <PATH>` - Custom wallet file path

**Examples:**
```bash
# Send 100 ACT with default gas
act-wallet send --to ACT-recipient... --amount 100

# Send with custom gas settings
act-wallet send --to ACT-recipient... --amount 50 --gas-limit 30000 --gas-price 2000000000
```

---

### `deploy` - Deploy WASM Contract

Deploy a WebAssembly smart contract to the blockchain.

```bash
act-wallet deploy --wasm <FILE> [OPTIONS]
```

**Required:**
- `--wasm <FILE>` - Path to compiled WASM file

**Options:**
- `--value <ACT>` - Initial contract balance in ACT (default: 0)
- `--gas-limit <LIMIT>` - Gas limit (default: 1000000)
- `--gas-price <PRICE>` - Gas price in smallest units (default: 1000000000)
- `--rpc <URL>` - Custom RPC endpoint
- `--wallet <PATH>` - Custom wallet file path

**Examples:**
```bash
# Deploy contract with no initial balance
act-wallet deploy --wasm contract.wasm

# Deploy with 10 ACT initial balance and custom gas
act-wallet deploy --wasm contract.wasm --value 10 --gas-limit 2000000
```

---

### `list` - List All Wallets

Show all wallets stored in `~/.act-wallet/`.

```bash
act-wallet list
```

**Example Output:**
```
Available wallets:

  • default
  • my-wallet
  • test-wallet
```

---

### `export` - Export Mnemonic

Display your wallet's recovery phrase. **USE WITH CAUTION!**

```bash
act-wallet export [OPTIONS]
```

**Options:**
- `--wallet <PATH>` - Custom wallet file path

**Example:**
```bash
act-wallet export
```

**Warning:** Never share your mnemonic phrase. Anyone with access to it can control your funds.

---

## 🌐 RPC Endpoints

### Live ACT Blockchain Nodes

By default, the CLI wallet connects to **Node 1**:
```
http://107.178.223.1:8545
```

### Alternate Nodes

You can use any of the three validator nodes:

- **Node 1:** `http://107.178.223.1:8545` (us-central1-a)
- **Node 2:** `http://34.70.254.28:8545` (us-central1-b)
- **Node 3:** `http://34.118.200.106:8545` (us-central1-c)

**Set alternate RPC globally:**
```bash
act-wallet --rpc http://34.70.254.28:8545 balance
```

---

## 📁 Wallet Storage

### Default Location

Wallets are stored in:
- **Linux/macOS:** `~/.act-wallet/`
- **Windows:** `C:\Users\<username>\.act-wallet\`

### File Format

Wallets are stored as encrypted JSON files:
```
~/.act-wallet/
  ├── default.json
  ├── my-wallet.json
  └── test-wallet.json
```

### Security

- Wallets are encrypted using XOR encryption with your password
- **For production use, implement proper encryption (AES-256-GCM)**
- Never commit wallet files to version control
- Use strong passwords (minimum 12 characters recommended)

---

## 💡 Tips & Best Practices

### 1. Backup Your Mnemonic

**ALWAYS** write down your 12-word recovery phrase and store it securely:
- ✅ Write on paper and store in a safe
- ✅ Use a hardware wallet for long-term storage
- ❌ Never store digitally (screenshots, text files, cloud storage)
- ❌ Never share with anyone

### 2. Gas Costs

**Transfer Transaction:**
- Recommended gas limit: `21000`
- Typical cost: 0.000021 ACT (at 1 gwei gas price)

**Contract Deployment:**
- Recommended gas limit: `1000000` - `5000000` (depending on contract size)
- Typical cost: 0.001 - 0.005 ACT

### 3. Test Before Mainnet

Before sending large amounts:
1. Test with a small amount (0.1 ACT)
2. Verify the transaction on a block explorer
3. Confirm the recipient received the funds

### 4. Multiple Wallets

Organize your wallets by purpose:
```bash
act-wallet create --name hot-wallet     # For daily transactions
act-wallet create --name savings        # For long-term holding
act-wallet create --name contracts      # For contract deployments
```

### 5. Check Balance Before Sending

Always verify your balance and nonce:
```bash
act-wallet balance --detailed
```

---

## 🔍 Troubleshooting

### "Cannot find home directory"

**Linux/macOS:**
```bash
export HOME=/home/yourusername
act-wallet create
```

**Windows:**
```powershell
$env:USERPROFILE = "C:\Users\yourusername"
.\act-wallet.exe create
```

### "RPC request failed: 404"

Check that you're using the correct RPC endpoint:
```bash
act-wallet --rpc http://107.178.223.1:8545 balance
```

### "Passwords do not match"

Ensure you type the same password twice when creating/importing wallets.

### "Wallet already exists"

Use a different name or remove the existing wallet:
```bash
rm ~/.act-wallet/default.json
act-wallet create
```

### "Failed to get balance: RPC error -32000"

This usually means:
- Your address has no funds (balance = 0)
- The RPC node is unreachable
- Network connectivity issues

Try a different RPC endpoint:
```bash
act-wallet --rpc http://34.70.254.28:8545 balance
```

---

## 🔒 Security Notes

### Wallet Encryption

Current implementation uses **basic XOR encryption** for demonstration. For production:

1. Use AES-256-GCM encryption
2. Use proper key derivation (PBKDF2, Argon2)
3. Add salt to prevent rainbow table attacks
4. Consider hardware wallet integration

### Password Requirements

- Minimum 12 characters
- Mix of uppercase, lowercase, numbers, symbols
- Don't reuse passwords from other accounts
- Consider using a password manager

### Private Key Security

- Never export or display private keys
- Use hardware wallets for large amounts
- Enable 2FA where possible (future feature)

---

## 📊 Examples

### Complete Workflow

```bash
# 1. Create wallet
act-wallet create --name alice

# 2. Check initial balance (likely 0)
act-wallet balance --wallet ~/.act-wallet/alice.json

# 3. Receive some ACT (have someone send to your address)

# 4. Check balance again
act-wallet balance --wallet ~/.act-wallet/alice.json --detailed

# 5. Send ACT to Bob
act-wallet send \
  --wallet ~/.act-wallet/alice.json \
  --to ACT-BobsAddressHere... \
  --amount 50.0

# 6. Deploy a contract
act-wallet deploy \
  --wallet ~/.act-wallet/alice.json \
  --wasm my_contract.wasm \
  --value 10.0 \
  --gas-limit 2000000
```

---

## 🔗 Resources

- **GitHub Repository:** https://github.com/actionsproof/Blockchain-
- **Live RPC Endpoints:** See `DEPLOYMENT_STATUS.md`
- **RPC API Documentation:** See `RPC_ACCESS.md`
- **Project Status:** See `PROJECT_STATUS.md`

---

## 🆘 Support

For issues or questions:
1. Check this documentation
2. Review `DEPLOYMENT_STATUS.md` for node status
3. Check GitHub Issues
4. Verify RPC endpoint connectivity

---

**Last Updated:** November 25, 2025  
**Version:** 0.1.0  
**License:** MIT
