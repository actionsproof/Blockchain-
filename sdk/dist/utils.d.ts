/**
 * Generate a new Ed25519 keypair for ACT Chain
 */
export declare function generateKeypair(): {
    publicKey: string;
    privateKey: string;
};
/**
 * Sign a message with a private key
 */
export declare function signMessage(message: string, privateKey: string): string;
/**
 * Verify a signature
 */
export declare function verifySignature(message: string, signature: string, publicKey: string): boolean;
/**
 * Derive address from public key (for ACT Chain, address = public key)
 */
export declare function publicKeyToAddress(publicKey: string): string;
/**
 * Convert ACT amount to smallest unit (1 ACT = 10^18 units)
 */
export declare function toBaseUnits(amount: number | string): string;
/**
 * Convert from smallest unit to ACT
 */
export declare function fromBaseUnits(amount: string | number): string;
/**
 * Create transaction hash
 */
export declare function hashTransaction(tx: {
    from: string;
    to: string | null;
    value: string;
    data: string;
    nonce: number;
}): string;
/**
 * Validate address format
 */
export declare function isValidAddress(address: string): boolean;
/**
 * Format balance for display
 */
export declare function formatBalance(balance: string, decimals?: number): string;
