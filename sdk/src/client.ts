import axios, { AxiosInstance } from 'axios';
import {
  Block,
  Transaction,
  TransactionReceipt,
  Account,
  PeerInfo,
  ValidatorInfo,
  DelegationInfo,
  UnstakeRequest,
  Proposal,
  Vote,
  RPCRequest,
  RPCResponse
} from './types';

/**
 * ACT Chain RPC Client - Full implementation of all 34 RPC methods
 */
export class ActClient {
  private rpcId: number = 1;
  private client: AxiosInstance;

  constructor(public readonly endpoint: string) {
    this.client = axios.create({
      baseURL: endpoint,
      headers: {
        'Content-Type': 'application/json'
      },
      timeout: 30000
    });
  }

  /**
   * Make RPC call
   */
  private async call<T>(method: string, params: any[] = []): Promise<T> {
    const request: RPCRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: this.rpcId++
    };

    try {
      const response = await this.client.post<RPCResponse<T>>('', request);
      
      if (response.data.error) {
        throw new Error(`RPC Error: ${response.data.error.message}`);
      }

      return response.data.result as T;
    } catch (error: any) {
      if (error.response?.data?.error) {
        throw new Error(`RPC Error: ${error.response.data.error.message}`);
      }
      throw error;
    }
  }

  // ===== Core ACT Methods =====

  /**
   * Get current block number
   */
  async getBlockNumber(): Promise<number> {
    return this.call<number>('act_blockNumber');
  }

  /**
   * Get block by number
   */
  async getBlock(blockNumber: number): Promise<Block> {
    return this.call<Block>('act_getBlockByNumber', [blockNumber]);
  }

  /**
   * Get block by hash
   */
  async getBlockByHash(hash: string): Promise<Block> {
    return this.call<Block>('act_getBlockByHash', [hash]);
  }

  /**
   * Send signed transaction
   */
  async sendTransaction(transaction: Transaction): Promise<string> {
    return this.call<string>('act_sendTransaction', [transaction]);
  }

  /**
   * Get transaction by hash
   */
  async getTransaction(hash: string): Promise<Transaction> {
    return this.call<Transaction>('act_getTransactionByHash', [hash]);
  }

  /**
   * Get transaction receipt
   */
  async getTransactionReceipt(hash: string): Promise<TransactionReceipt> {
    return this.call<TransactionReceipt>('act_getTransactionReceipt', [hash]);
  }

  /**
   * Get account balance
   */
  async getBalance(address: string): Promise<string> {
    return this.call<string>('act_getBalance', [address]);
  }

  /**
   * Get account nonce
   */
  async getNonce(address: string): Promise<number> {
    return this.call<number>('act_getNonce', [address]);
  }

  /**
   * Get connected peers
   */
  async getPeers(): Promise<PeerInfo[]> {
    return this.call<PeerInfo[]>('act_peers');
  }

  // ===== Ethereum Compatibility Methods =====

  /**
   * ETH - Get block number (alias)
   */
  async ethBlockNumber(): Promise<string> {
    const num = await this.getBlockNumber();
    return '0x' + num.toString(16);
  }

  /**
   * ETH - Get balance
   */
  async ethGetBalance(address: string, block: string = 'latest'): Promise<string> {
    const balance = await this.getBalance(address);
    return '0x' + BigInt(balance).toString(16);
  }

  /**
   * ETH - Get transaction count (nonce)
   */
  async ethGetTransactionCount(address: string, block: string = 'latest'): Promise<string> {
    const nonce = await this.getNonce(address);
    return '0x' + nonce.toString(16);
  }

  /**
   * ETH - Send raw transaction
   */
  async ethSendRawTransaction(data: string): Promise<string> {
    // Parse and send transaction
    return this.call<string>('eth_sendRawTransaction', [data]);
  }

  /**
   * ETH - Get transaction by hash
   */
  async ethGetTransactionByHash(hash: string): Promise<any> {
    return this.call<any>('eth_getTransactionByHash', [hash]);
  }

  /**
   * ETH - Get transaction receipt
   */
  async ethGetTransactionReceipt(hash: string): Promise<any> {
    return this.call<any>('eth_getTransactionReceipt', [hash]);
  }

  /**
   * ETH - Call contract (read-only)
   */
  async ethCall(call: { to: string; data: string }, block: string = 'latest'): Promise<string> {
    return this.call<string>('eth_call', [call, block]);
  }

  // ===== Staking Methods =====

  /**
   * Get all validators
   */
  async getValidators(): Promise<ValidatorInfo[]> {
    return this.call<ValidatorInfo[]>('staking_getValidators');
  }

  /**
   * Get validator info
   */
  async getValidator(address: string): Promise<ValidatorInfo> {
    return this.call<ValidatorInfo>('staking_getValidator', [address]);
  }

  /**
   * Stake tokens to become validator
   */
  async stake(from: string, amount: string, signature: string): Promise<string> {
    return this.call<string>('staking_stake', [from, amount, signature]);
  }

  /**
   * Unstake tokens
   */
  async unstake(validator: string, amount: string, signature: string): Promise<string> {
    return this.call<string>('staking_unstake', [validator, amount, signature]);
  }

  /**
   * Delegate tokens to validator
   */
  async delegate(delegator: string, validator: string, amount: string, signature: string): Promise<string> {
    return this.call<string>('staking_delegate', [delegator, validator, amount, signature]);
  }

  /**
   * Undelegate tokens
   */
  async undelegate(delegator: string, validator: string, amount: string, signature: string): Promise<string> {
    return this.call<string>('staking_undelegate', [delegator, validator, amount, signature]);
  }

  /**
   * Claim staking rewards
   */
  async claimRewards(validator: string, signature: string): Promise<string> {
    return this.call<string>('staking_claimRewards', [validator, signature]);
  }

  /**
   * Get delegations for address
   */
  async getDelegations(address: string): Promise<DelegationInfo[]> {
    return this.call<DelegationInfo[]>('staking_getDelegations', [address]);
  }

  /**
   * Get pending unstake requests
   */
  async getUnstakeRequests(validator: string): Promise<UnstakeRequest[]> {
    return this.call<UnstakeRequest[]>('staking_getUnstakeRequests', [validator]);
  }

  /**
   * Get total staked amount
   */
  async getTotalStaked(): Promise<string> {
    return this.call<string>('staking_getTotalStaked');
  }

  /**
   * Get staking rewards for validator
   */
  async getStakingRewards(validator: string): Promise<string> {
    return this.call<string>('staking_getRewards', [validator]);
  }

  // ===== Governance Methods =====

  /**
   * Create governance proposal
   */
  async createProposal(
    proposer: string,
    title: string,
    description: string,
    action: any,
    signature: string
  ): Promise<number> {
    return this.call<number>('governance_createProposal', [
      proposer,
      title,
      description,
      action,
      signature
    ]);
  }

  /**
   * Vote on proposal
   */
  async vote(voter: string, proposalId: number, support: boolean, signature: string): Promise<string> {
    return this.call<string>('governance_vote', [voter, proposalId, support, signature]);
  }

  /**
   * Execute proposal
   */
  async executeProposal(proposalId: number, executor: string, signature: string): Promise<string> {
    return this.call<string>('governance_executeProposal', [proposalId, executor, signature]);
  }

  /**
   * Get proposal by ID
   */
  async getProposal(proposalId: number): Promise<Proposal> {
    return this.call<Proposal>('governance_getProposal', [proposalId]);
  }

  /**
   * Get all proposals
   */
  async getProposals(): Promise<Proposal[]> {
    return this.call<Proposal[]>('governance_getProposals');
  }

  /**
   * Get votes for proposal
   */
  async getVotes(proposalId: number): Promise<Vote[]> {
    return this.call<Vote[]>('governance_getVotes', [proposalId]);
  }

  /**
   * Get voting power for address
   */
  async getVotingPower(address: string): Promise<string> {
    return this.call<string>('governance_getVotingPower', [address]);
  }

  // ===== Utility Methods =====

  /**
   * Wait for transaction confirmation
   */
  async waitForTransaction(
    txHash: string,
    confirmations: number = 1,
    timeout: number = 60000
  ): Promise<TransactionReceipt> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeout) {
      try {
        const receipt = await this.getTransactionReceipt(txHash);
        
        if (receipt && receipt.block_number) {
          const currentBlock = await this.getBlockNumber();
          const txConfirmations = currentBlock - receipt.block_number + 1;
          
          if (txConfirmations >= confirmations) {
            return receipt;
          }
        }
      } catch (error) {
        // Transaction not yet mined, continue waiting
      }

      await new Promise(resolve => setTimeout(resolve, 2000));
    }

    throw new Error('Transaction confirmation timeout');
  }

  /**
   * Get account details
   */
  async getAccount(address: string): Promise<Account> {
    const [balance, nonce] = await Promise.all([
      this.getBalance(address),
      this.getNonce(address)
    ]);

    return {
      address,
      balance,
      nonce
    };
  }

  /**
   * Check node health
   */
  async isHealthy(): Promise<boolean> {
    try {
      await this.getBlockNumber();
      return true;
    } catch {
      return false;
    }
  }
}
