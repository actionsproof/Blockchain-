import { ActClient } from './client';
import { Wallet } from './wallet';
/**
 * Contract class for deploying and interacting with WASM smart contracts
 */
export declare class Contract {
    private client;
    private wallet;
    readonly address?: string | undefined;
    constructor(client: ActClient, wallet: Wallet, address?: string | undefined);
    /**
     * Deploy a new contract
     */
    static deploy(client: ActClient, wallet: Wallet, wasmCode: Uint8Array | string, constructorArgs?: any[]): Promise<Contract>;
    /**
     * Call contract method (state-changing)
     */
    call(method: string, args?: any[], value?: string): Promise<string>;
    /**
     * Query contract method (read-only)
     */
    query(method: string, args?: any[]): Promise<any>;
    /**
     * Get contract address
     */
    getAddress(): string;
}
/**
 * ACT-20 Token Contract Helper
 */
export declare class Act20Contract extends Contract {
    /**
     * Deploy new ACT-20 token
     */
    static deployToken(client: ActClient, wallet: Wallet, wasmCode: Uint8Array | string, name: string, symbol: string, decimals: number, initialSupply: string, mintable?: boolean, burnable?: boolean): Promise<Act20Contract>;
    /**
     * Get token name
     */
    name(): Promise<string>;
    /**
     * Get token symbol
     */
    symbol(): Promise<string>;
    /**
     * Get token decimals
     */
    decimals(): Promise<number>;
    /**
     * Get total supply
     */
    totalSupply(): Promise<string>;
    /**
     * Get balance of address
     */
    balanceOf(address: string): Promise<string>;
    /**
     * Transfer tokens
     */
    transfer(to: string, amount: string): Promise<string>;
    /**
     * Approve spender
     */
    approve(spender: string, amount: string): Promise<string>;
    /**
     * Get allowance
     */
    allowance(owner: string, spender: string): Promise<string>;
    /**
     * Transfer from (requires approval)
     */
    transferFrom(from: string, to: string, amount: string): Promise<string>;
    /**
     * Mint tokens (if mintable)
     */
    mint(to: string, amount: string): Promise<string>;
    /**
     * Burn tokens (if burnable)
     */
    burn(amount: string): Promise<string>;
}
/**
 * DEX Contract Helper
 */
export declare class DexContract extends Contract {
    /**
     * Connect to existing DEX
     */
    static connect(client: ActClient, wallet: Wallet, address: string): Promise<DexContract>;
    /**
     * Add liquidity to pool
     */
    addLiquidity(amountA: string, amountB: string): Promise<string>;
    /**
     * Remove liquidity from pool
     */
    removeLiquidity(liquidity: string): Promise<string>;
    /**
     * Swap token A for token B
     */
    swapAForB(amountIn: string): Promise<string>;
    /**
     * Swap token B for token A
     */
    swapBForA(amountIn: string): Promise<string>;
    /**
     * Get quote for swap A to B
     */
    getQuoteAForB(amountIn: string): Promise<string>;
    /**
     * Get quote for swap B to A
     */
    getQuoteBForA(amountIn: string): Promise<string>;
    /**
     * Get pool reserves
     */
    getReserves(): Promise<{
        reserveA: string;
        reserveB: string;
    }>;
    /**
     * Get LP token balance
     */
    getLPBalance(address: string): Promise<string>;
    /**
     * Get price of token A in terms of token B
     */
    getPriceAInB(): Promise<string>;
    /**
     * Get price of token B in terms of token A
     */
    getPriceBInA(): Promise<string>;
}
