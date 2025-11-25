/**
 * Complete example demonstrating ACT Chain SDK usage
 */

import { ActClient, Wallet, Act20Contract, DexContract, toBaseUnits, fromBaseUnits } from '@actchain/sdk';

async function main() {
  console.log('🚀 ACT Chain SDK Example\n');

  // 1. Create wallet
  console.log('1️⃣ Creating wallet...');
  const wallet = Wallet.createRandom();
  console.log('Address:', wallet.address);
  console.log('Public Key:', wallet.publicKey);
  console.log();

  // 2. Connect to node
  console.log('2️⃣ Connecting to ACT Chain node...');
  const client = new ActClient('http://107.178.223.1:8545');
  
  const isHealthy = await client.isHealthy();
  console.log('Node healthy:', isHealthy);
  
  const blockNumber = await client.getBlockNumber();
  console.log('Current block:', blockNumber);
  console.log();

  // 3. Check balance
  console.log('3️⃣ Checking account balance...');
  const account = await client.getAccount(wallet.address);
  console.log('Balance:', fromBaseUnits(account.balance), 'ACT');
  console.log('Nonce:', account.nonce);
  console.log();

  // 4. Send transaction (if you have balance)
  if (BigInt(account.balance) > 0) {
    console.log('4️⃣ Sending transaction...');
    const recipientAddress = 'some-recipient-address';
    
    const tx = wallet.signTransaction({
      to: recipientAddress,
      value: toBaseUnits(1), // 1 ACT
      nonce: account.nonce
    });

    try {
      const txHash = await client.sendTransaction(tx);
      console.log('Transaction hash:', txHash);
      
      const receipt = await client.waitForTransaction(txHash, 1);
      console.log('Status:', receipt.status);
      console.log();
    } catch (error: any) {
      console.log('Transaction failed:', error.message);
      console.log();
    }
  }

  // 5. Query blockchain
  console.log('5️⃣ Querying blockchain data...');
  const latestBlock = await client.getBlock(blockNumber);
  console.log('Latest block hash:', latestBlock.hash);
  console.log('Transactions in block:', latestBlock.transactions.length);
  console.log('Block proposer:', latestBlock.proposer);
  console.log();

  // 6. Staking information
  console.log('6️⃣ Checking staking information...');
  try {
    const validators = await client.getValidators();
    console.log('Total validators:', validators.length);
    
    if (validators.length > 0) {
      const validator = validators[0];
      console.log('Sample validator:');
      console.log('  Address:', validator.address);
      console.log('  Stake:', fromBaseUnits(validator.stake), 'ACT');
      console.log('  Active:', validator.active);
      console.log('  Blocks proposed:', validator.blocks_proposed);
    }

    const totalStaked = await client.getTotalStaked();
    console.log('Total staked:', fromBaseUnits(totalStaked), 'ACT');
    console.log();
  } catch (error: any) {
    console.log('Staking query failed:', error.message);
    console.log();
  }

  // 7. Governance information
  console.log('7️⃣ Checking governance...');
  try {
    const proposals = await client.getProposals();
    console.log('Total proposals:', proposals.length);
    
    if (proposals.length > 0) {
      const proposal = proposals[0];
      console.log('Sample proposal:');
      console.log('  ID:', proposal.id);
      console.log('  Title:', proposal.title);
      console.log('  Status:', proposal.status);
      console.log('  Yes votes:', fromBaseUnits(proposal.yes_votes));
      console.log('  No votes:', fromBaseUnits(proposal.no_votes));
    }

    const votingPower = await client.getVotingPower(wallet.address);
    console.log('Your voting power:', fromBaseUnits(votingPower), 'ACT');
    console.log();
  } catch (error: any) {
    console.log('Governance query failed:', error.message);
    console.log();
  }

  // 8. Network peers
  console.log('8️⃣ Checking network peers...');
  try {
    const peers = await client.getPeers();
    console.log('Connected peers:', peers.length);
    peers.forEach((peer, i) => {
      console.log(`  Peer ${i + 1}:`, peer.address);
    });
    console.log();
  } catch (error: any) {
    console.log('Peers query failed:', error.message);
    console.log();
  }

  // 9. ETH compatibility
  console.log('9️⃣ Testing Ethereum RPC compatibility...');
  try {
    const ethBlockNumber = await client.ethBlockNumber();
    console.log('Block number (hex):', ethBlockNumber);
    
    const ethBalance = await client.ethGetBalance(wallet.address);
    console.log('Balance (hex):', ethBalance);
    console.log();
  } catch (error: any) {
    console.log('ETH RPC failed:', error.message);
    console.log();
  }

  console.log('✅ Example completed successfully!');
}

// Example: ACT-20 Token Interaction
async function tokenExample() {
  console.log('\n💰 ACT-20 Token Example\n');

  const client = new ActClient('http://107.178.223.1:8545');
  const wallet = Wallet.createRandom();

  // Assuming you have a deployed token contract
  const tokenAddress = 'your-token-contract-address';
  
  // Note: In production, you would deploy with Act20Contract.deploy()
  // For this example, we're showing interaction with an existing token

  console.log('Token address:', tokenAddress);
  console.log('Wallet address:', wallet.address);
  console.log();

  console.log('This example shows the SDK interface.');
  console.log('To run it, deploy a token contract first.');
}

// Example: DEX Trading
async function dexExample() {
  console.log('\n🔄 DEX Trading Example\n');

  const client = new ActClient('http://107.178.223.1:8545');
  const wallet = Wallet.createRandom();

  // Assuming you have a deployed DEX contract
  const dexAddress = 'your-dex-contract-address';

  console.log('DEX address:', dexAddress);
  console.log('Wallet address:', wallet.address);
  console.log();

  console.log('This example shows the SDK interface.');
  console.log('To run it, deploy a DEX contract first.');
}

// Run examples
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error('Error:', error);
      process.exit(1);
    });
}

export { main, tokenExample, dexExample };
