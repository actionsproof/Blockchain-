import { Transaction } from './types';
/**
 * Wallet class for managing ACT Chain accounts
 */
export declare class Wallet {
    readonly address: string;
    readonly publicKey: string;
    private readonly privateKey;
    constructor(privateKey?: string);
    /**
     * Create a new random wallet
     */
    static createRandom(): Wallet;
    /**
     * Import wallet from private key
     */
    static fromPrivateKey(privateKey: string): Wallet;
    /**
     * Export private key (use carefully!)
     */
    exportPrivateKey(): string;
    /**
     * Sign a transaction
     */
    signTransaction(tx: {
        to: string | null;
        value: string | number;
        data?: string;
        nonce: number;
    }): Transaction;
    /**
     * Sign arbitrary data
     */
    sign(data: string): string;
    /**
     * Get wallet info
     */
    getInfo(): {
        address: string;
        publicKey: string;
    };
}
