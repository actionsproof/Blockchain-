import { Block, Transaction, TransactionReceipt, Account, PeerInfo, ValidatorInfo, DelegationInfo, UnstakeRequest, Proposal, Vote } from './types';
/**
 * ACT Chain RPC Client - Full implementation of all 34 RPC methods
 */
export declare class ActClient {
    readonly endpoint: string;
    private rpcId;
    private client;
    constructor(endpoint: string);
    /**
     * Make RPC call
     */
    private call;
    /**
     * Get current block number
     */
    getBlockNumber(): Promise<number>;
    /**
     * Get block by number
     */
    getBlock(blockNumber: number): Promise<Block>;
    /**
     * Get block by hash
     */
    getBlockByHash(hash: string): Promise<Block>;
    /**
     * Send signed transaction
     */
    sendTransaction(transaction: Transaction): Promise<string>;
    /**
     * Get transaction by hash
     */
    getTransaction(hash: string): Promise<Transaction>;
    /**
     * Get transaction receipt
     */
    getTransactionReceipt(hash: string): Promise<TransactionReceipt>;
    /**
     * Get account balance
     */
    getBalance(address: string): Promise<string>;
    /**
     * Get account nonce
     */
    getNonce(address: string): Promise<number>;
    /**
     * Get connected peers
     */
    getPeers(): Promise<PeerInfo[]>;
    /**
     * ETH - Get block number (alias)
     */
    ethBlockNumber(): Promise<string>;
    /**
     * ETH - Get balance
     */
    ethGetBalance(address: string, block?: string): Promise<string>;
    /**
     * ETH - Get transaction count (nonce)
     */
    ethGetTransactionCount(address: string, block?: string): Promise<string>;
    /**
     * ETH - Send raw transaction
     */
    ethSendRawTransaction(data: string): Promise<string>;
    /**
     * ETH - Get transaction by hash
     */
    ethGetTransactionByHash(hash: string): Promise<any>;
    /**
     * ETH - Get transaction receipt
     */
    ethGetTransactionReceipt(hash: string): Promise<any>;
    /**
     * ETH - Call contract (read-only)
     */
    ethCall(call: {
        to: string;
        data: string;
    }, block?: string): Promise<string>;
    /**
     * Get all validators
     */
    getValidators(): Promise<ValidatorInfo[]>;
    /**
     * Get validator info
     */
    getValidator(address: string): Promise<ValidatorInfo>;
    /**
     * Stake tokens to become validator
     */
    stake(from: string, amount: string, signature: string): Promise<string>;
    /**
     * Unstake tokens
     */
    unstake(validator: string, amount: string, signature: string): Promise<string>;
    /**
     * Delegate tokens to validator
     */
    delegate(delegator: string, validator: string, amount: string, signature: string): Promise<string>;
    /**
     * Undelegate tokens
     */
    undelegate(delegator: string, validator: string, amount: string, signature: string): Promise<string>;
    /**
     * Claim staking rewards
     */
    claimRewards(validator: string, signature: string): Promise<string>;
    /**
     * Get delegations for address
     */
    getDelegations(address: string): Promise<DelegationInfo[]>;
    /**
     * Get pending unstake requests
     */
    getUnstakeRequests(validator: string): Promise<UnstakeRequest[]>;
    /**
     * Get total staked amount
     */
    getTotalStaked(): Promise<string>;
    /**
     * Get staking rewards for validator
     */
    getStakingRewards(validator: string): Promise<string>;
    /**
     * Create governance proposal
     */
    createProposal(proposer: string, title: string, description: string, action: any, signature: string): Promise<number>;
    /**
     * Vote on proposal
     */
    vote(voter: string, proposalId: number, support: boolean, signature: string): Promise<string>;
    /**
     * Execute proposal
     */
    executeProposal(proposalId: number, executor: string, signature: string): Promise<string>;
    /**
     * Get proposal by ID
     */
    getProposal(proposalId: number): Promise<Proposal>;
    /**
     * Get all proposals
     */
    getProposals(): Promise<Proposal[]>;
    /**
     * Get votes for proposal
     */
    getVotes(proposalId: number): Promise<Vote[]>;
    /**
     * Get voting power for address
     */
    getVotingPower(address: string): Promise<string>;
    /**
     * Wait for transaction confirmation
     */
    waitForTransaction(txHash: string, confirmations?: number, timeout?: number): Promise<TransactionReceipt>;
    /**
     * Get account details
     */
    getAccount(address: string): Promise<Account>;
    /**
     * Check node health
     */
    isHealthy(): Promise<boolean>;
}
