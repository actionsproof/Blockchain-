"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ActClient = void 0;
const axios_1 = __importDefault(require("axios"));
/**
 * ACT Chain RPC Client - Full implementation of all 34 RPC methods
 */
class ActClient {
    constructor(endpoint) {
        this.endpoint = endpoint;
        this.rpcId = 1;
        this.client = axios_1.default.create({
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
    async call(method, params = []) {
        const request = {
            jsonrpc: '2.0',
            method,
            params,
            id: this.rpcId++
        };
        try {
            const response = await this.client.post('', request);
            if (response.data.error) {
                throw new Error(`RPC Error: ${response.data.error.message}`);
            }
            return response.data.result;
        }
        catch (error) {
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
    async getBlockNumber() {
        return this.call('act_blockNumber');
    }
    /**
     * Get block by number
     */
    async getBlock(blockNumber) {
        return this.call('act_getBlockByNumber', [blockNumber]);
    }
    /**
     * Get block by hash
     */
    async getBlockByHash(hash) {
        return this.call('act_getBlockByHash', [hash]);
    }
    /**
     * Send signed transaction
     */
    async sendTransaction(transaction) {
        return this.call('act_sendTransaction', [transaction]);
    }
    /**
     * Get transaction by hash
     */
    async getTransaction(hash) {
        return this.call('act_getTransactionByHash', [hash]);
    }
    /**
     * Get transaction receipt
     */
    async getTransactionReceipt(hash) {
        return this.call('act_getTransactionReceipt', [hash]);
    }
    /**
     * Get account balance
     */
    async getBalance(address) {
        return this.call('act_getBalance', [address]);
    }
    /**
     * Get account nonce
     */
    async getNonce(address) {
        return this.call('act_getNonce', [address]);
    }
    /**
     * Get connected peers
     */
    async getPeers() {
        return this.call('act_peers');
    }
    // ===== Ethereum Compatibility Methods =====
    /**
     * ETH - Get block number (alias)
     */
    async ethBlockNumber() {
        const num = await this.getBlockNumber();
        return '0x' + num.toString(16);
    }
    /**
     * ETH - Get balance
     */
    async ethGetBalance(address, block = 'latest') {
        const balance = await this.getBalance(address);
        return '0x' + BigInt(balance).toString(16);
    }
    /**
     * ETH - Get transaction count (nonce)
     */
    async ethGetTransactionCount(address, block = 'latest') {
        const nonce = await this.getNonce(address);
        return '0x' + nonce.toString(16);
    }
    /**
     * ETH - Send raw transaction
     */
    async ethSendRawTransaction(data) {
        // Parse and send transaction
        return this.call('eth_sendRawTransaction', [data]);
    }
    /**
     * ETH - Get transaction by hash
     */
    async ethGetTransactionByHash(hash) {
        return this.call('eth_getTransactionByHash', [hash]);
    }
    /**
     * ETH - Get transaction receipt
     */
    async ethGetTransactionReceipt(hash) {
        return this.call('eth_getTransactionReceipt', [hash]);
    }
    /**
     * ETH - Call contract (read-only)
     */
    async ethCall(call, block = 'latest') {
        return this.call('eth_call', [call, block]);
    }
    // ===== Staking Methods =====
    /**
     * Get all validators
     */
    async getValidators() {
        return this.call('staking_getValidators');
    }
    /**
     * Get validator info
     */
    async getValidator(address) {
        return this.call('staking_getValidator', [address]);
    }
    /**
     * Stake tokens to become validator
     */
    async stake(from, amount, signature) {
        return this.call('staking_stake', [from, amount, signature]);
    }
    /**
     * Unstake tokens
     */
    async unstake(validator, amount, signature) {
        return this.call('staking_unstake', [validator, amount, signature]);
    }
    /**
     * Delegate tokens to validator
     */
    async delegate(delegator, validator, amount, signature) {
        return this.call('staking_delegate', [delegator, validator, amount, signature]);
    }
    /**
     * Undelegate tokens
     */
    async undelegate(delegator, validator, amount, signature) {
        return this.call('staking_undelegate', [delegator, validator, amount, signature]);
    }
    /**
     * Claim staking rewards
     */
    async claimRewards(validator, signature) {
        return this.call('staking_claimRewards', [validator, signature]);
    }
    /**
     * Get delegations for address
     */
    async getDelegations(address) {
        return this.call('staking_getDelegations', [address]);
    }
    /**
     * Get pending unstake requests
     */
    async getUnstakeRequests(validator) {
        return this.call('staking_getUnstakeRequests', [validator]);
    }
    /**
     * Get total staked amount
     */
    async getTotalStaked() {
        return this.call('staking_getTotalStaked');
    }
    /**
     * Get staking rewards for validator
     */
    async getStakingRewards(validator) {
        return this.call('staking_getRewards', [validator]);
    }
    // ===== Governance Methods =====
    /**
     * Create governance proposal
     */
    async createProposal(proposer, title, description, action, signature) {
        return this.call('governance_createProposal', [
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
    async vote(voter, proposalId, support, signature) {
        return this.call('governance_vote', [voter, proposalId, support, signature]);
    }
    /**
     * Execute proposal
     */
    async executeProposal(proposalId, executor, signature) {
        return this.call('governance_executeProposal', [proposalId, executor, signature]);
    }
    /**
     * Get proposal by ID
     */
    async getProposal(proposalId) {
        return this.call('governance_getProposal', [proposalId]);
    }
    /**
     * Get all proposals
     */
    async getProposals() {
        return this.call('governance_getProposals');
    }
    /**
     * Get votes for proposal
     */
    async getVotes(proposalId) {
        return this.call('governance_getVotes', [proposalId]);
    }
    /**
     * Get voting power for address
     */
    async getVotingPower(address) {
        return this.call('governance_getVotingPower', [address]);
    }
    // ===== Utility Methods =====
    /**
     * Wait for transaction confirmation
     */
    async waitForTransaction(txHash, confirmations = 1, timeout = 60000) {
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
            }
            catch (error) {
                // Transaction not yet mined, continue waiting
            }
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        throw new Error('Transaction confirmation timeout');
    }
    /**
     * Get account details
     */
    async getAccount(address) {
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
    async isHealthy() {
        try {
            await this.getBlockNumber();
            return true;
        }
        catch {
            return false;
        }
    }
}
exports.ActClient = ActClient;
