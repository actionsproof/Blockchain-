"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Wallet = void 0;
const utils_1 = require("./utils");
/**
 * Wallet class for managing ACT Chain accounts
 */
class Wallet {
    constructor(privateKey) {
        if (privateKey) {
            // Import existing wallet
            this.privateKey = privateKey;
            // In production, derive public key from private key
            // For now, we'll generate a new keypair if only private key is provided
            const keypair = (0, utils_1.generateKeypair)();
            this.privateKey = privateKey;
            this.publicKey = keypair.publicKey;
            this.address = (0, utils_1.publicKeyToAddress)(this.publicKey);
        }
        else {
            // Generate new wallet
            const keypair = (0, utils_1.generateKeypair)();
            this.publicKey = keypair.publicKey;
            this.privateKey = keypair.privateKey;
            this.address = (0, utils_1.publicKeyToAddress)(this.publicKey);
        }
    }
    /**
     * Create a new random wallet
     */
    static createRandom() {
        return new Wallet();
    }
    /**
     * Import wallet from private key
     */
    static fromPrivateKey(privateKey) {
        return new Wallet(privateKey);
    }
    /**
     * Export private key (use carefully!)
     */
    exportPrivateKey() {
        return this.privateKey;
    }
    /**
     * Sign a transaction
     */
    signTransaction(tx) {
        const transaction = {
            from: this.address,
            to: tx.to,
            value: typeof tx.value === 'number' ? (0, utils_1.toBaseUnits)(tx.value) : tx.value,
            data: tx.data || '',
            nonce: tx.nonce
        };
        const hash = (0, utils_1.hashTransaction)(transaction);
        const signature = (0, utils_1.signMessage)(hash, this.privateKey);
        return {
            ...transaction,
            hash,
            signature
        };
    }
    /**
     * Sign arbitrary data
     */
    sign(data) {
        return (0, utils_1.signMessage)(data, this.privateKey);
    }
    /**
     * Get wallet info
     */
    getInfo() {
        return {
            address: this.address,
            publicKey: this.publicKey
        };
    }
}
exports.Wallet = Wallet;
