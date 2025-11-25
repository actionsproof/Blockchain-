# ACT Chain JavaScript SDK

Official JavaScript/TypeScript SDK for interacting with ACT Chain nodes, managing wallets, deploying smart contracts, and building decentralized applications.

## Features

- 🔑 **Wallet Management** - Create, import, and manage Ed25519 wallets
- 📡 **RPC Client** - Full implementation of all 34 ACT Chain RPC methods
- 📝 **Smart Contracts** - Deploy and interact with WASM contracts
- 💰 **Token Support** - Built-in ACT-20 token helpers
- 🔄 **DEX Integration** - Trade tokens on the built-in DEX
- 🏛️ **Staking & Governance** - Stake tokens and participate in governance
- ⚡ **Ethereum Compatible** - Support for ETH RPC methods
- 🔐 **TypeScript** - Full TypeScript support with type definitions

## Installation

```bash
npm install @actchain/sdk
```

## Quick Start

### Create a Wallet

```typescript
import { Wallet } from '@actchain/sdk';

// Create new random wallet
const wallet = Wallet.createRandom();
console.log('Address:', wallet.address);
console.log('Private Key:', wallet.exportPrivateKey());

// Import existing wallet
const imported = Wallet.fromPrivateKey('your-private-key-here');
```

### Connect to Node

```typescript
import { ActClient } from '@actchain/sdk';

const client = new ActClient('http://107.178.223.1:8545');

// Get current block number
const blockNumber = await client.getBlockNumber();
console.log('Current block:', blockNumber);

// Get account balance
const balance = await client.getBalance(wallet.address);
console.log('Balance:', balance);
```

### Send Transaction

```typescript
// Get nonce
const nonce = await client.getNonce(wallet.address);

// Create and sign transaction
const tx = wallet.signTransaction({
  to: 'recipient-address',
  value: '1000000000000000000', // 1 ACT
  nonce
});

// Send transaction
const txHash = await client.sendTransaction(tx);
console.log('Transaction hash:', txHash);

// Wait for confirmation
const receipt = await client.waitForTransaction(txHash);
console.log('Transaction confirmed:', receipt);
```

### Deploy Smart Contract

```typescript
import { Contract } from '@actchain/sdk';
import { readFileSync } from 'fs';

// Load WASM bytecode
const wasmCode = readFileSync('./contract.wasm');

// Deploy contract
const contract = await Contract.deploy(
  client,
  wallet,
  wasmCode,
  ['constructor', 'args']
);

console.log('Contract deployed at:', contract.getAddress());

// Call contract method
const txHash = await contract.call('method_name', ['arg1', 'arg2']);

// Query contract (read-only)
const result = await contract.query('view_method', ['arg1']);
```

### ACT-20 Tokens

```typescript
import { Act20Contract } from '@actchain/sdk';

// Deploy new token
const token = await Act20Contract.deployToken(
  client,
  wallet,
  wasmCode,
  'My Token',      // name
  'MTK',           // symbol
  18,              // decimals
  '1000000000000000000000000', // initial supply (1M tokens)
  true,            // mintable
  true             // burnable
);

// Get token info
const name = await token.name();
const symbol = await token.symbol();
const totalSupply = await token.totalSupply();

// Transfer tokens
await token.transfer(recipientAddress, '1000000000000000000');

// Approve spender
await token.approve(spenderAddress, '1000000000000000000');

// Check balance
const balance = await token.balanceOf(wallet.address);
```

### Use DEX

```typescript
import { DexContract } from '@actchain/sdk';

// Connect to DEX
const dex = await DexContract.connect(client, wallet, dexAddress);

// Get price quote
const quote = await dex.getQuoteAForB('1000000000000000000');
console.log('Quote:', quote);

// Swap tokens
const txHash = await dex.swapAForB('1000000000000000000');

// Add liquidity
await dex.addLiquidity('1000000000000000000', '1000000000000000000');

// Get pool info
const reserves = await dex.getReserves();
console.log('Reserve A:', reserves.reserveA);
console.log('Reserve B:', reserves.reserveB);
```

### Staking

```typescript
// Get all validators
const validators = await client.getValidators();
console.log('Validators:', validators);

// Stake tokens (100,000 ACT minimum)
const stakeAmount = '100000000000000000000000';
const signature = wallet.sign(stakeAmount);
await client.stake(wallet.address, stakeAmount, signature);

// Delegate to validator
const delegateAmount = '10000000000000000000000';
const delegateSig = wallet.sign(delegateAmount);
await client.delegate(
  wallet.address,
  validatorAddress,
  delegateAmount,
  delegateSig
);

// Claim rewards
const rewardsSig = wallet.sign('claim');
await client.claimRewards(wallet.address, rewardsSig);

// Get delegations
const delegations = await client.getDelegations(wallet.address);
```

### Governance

```typescript
// Create proposal
const proposal = {
  title: 'Increase Block Rewards',
  description: 'Proposal to increase block rewards from 50 to 100 ACT',
  action: {
    type: 'parameter_change',
    data: { parameter: 'block_reward', value: '100' }
  }
};

const proposalSig = wallet.sign(JSON.stringify(proposal));
const proposalId = await client.createProposal(
  wallet.address,
  proposal.title,
  proposal.description,
  proposal.action,
  proposalSig
);

// Vote on proposal
const voteSig = wallet.sign(`vote-${proposalId}`);
await client.vote(wallet.address, proposalId, true, voteSig);

// Get proposal details
const proposalDetails = await client.getProposal(proposalId);
console.log('Proposal:', proposalDetails);

// Execute proposal (after voting period)
const executeSig = wallet.sign(`execute-${proposalId}`);
await client.executeProposal(proposalId, wallet.address, executeSig);
```

## Utility Functions

```typescript
import { toBaseUnits, fromBaseUnits, formatBalance } from '@actchain/sdk';

// Convert ACT to base units
const units = toBaseUnits(1.5); // '1500000000000000000'

// Convert base units to ACT
const act = fromBaseUnits('1500000000000000000'); // '1.5'

// Format balance for display
const formatted = formatBalance('123456789000000000000000'); // '123.46K'
```

## API Reference

### ActClient

- `getBlockNumber()` - Get current block number
- `getBlock(number)` - Get block by number
- `getTransaction(hash)` - Get transaction by hash
- `getTransactionReceipt(hash)` - Get transaction receipt
- `sendTransaction(tx)` - Send signed transaction
- `getBalance(address)` - Get account balance
- `getNonce(address)` - Get account nonce
- `getPeers()` - Get connected peers
- `getValidators()` - Get all validators
- `getProposals()` - Get all governance proposals
- Plus 24 more methods for staking, governance, and ETH compatibility

### Wallet

- `Wallet.createRandom()` - Create new wallet
- `Wallet.fromPrivateKey(key)` - Import wallet
- `signTransaction(tx)` - Sign transaction
- `sign(data)` - Sign arbitrary data
- `exportPrivateKey()` - Export private key

### Contract

- `Contract.deploy(client, wallet, code, args)` - Deploy contract
- `call(method, args, value)` - Call contract method
- `query(method, args)` - Query contract (read-only)
- `getAddress()` - Get contract address

## Testing

```bash
npm test
```

## License

MIT

## Support

- GitHub: https://github.com/actchain/sdk
- Documentation: https://docs.actchain.io
- Discord: https://discord.gg/actchain
