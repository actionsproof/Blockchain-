import { generateKeypair, signMessage, publicKeyToAddress, hashTransaction, toBaseUnits } from './utils';
import { Transaction } from './types';

/**
 * Wallet class for managing ACT Chain accounts
 */
export class Wallet {
  public readonly address: string;
  public readonly publicKey: string;
  private readonly privateKey: string;

  constructor(privateKey?: string) {
    if (privateKey) {
      // Import existing wallet
      this.privateKey = privateKey;
      // In production, derive public key from private key
      // For now, we'll generate a new keypair if only private key is provided
      const keypair = generateKeypair();
      this.privateKey = privateKey;
      this.publicKey = keypair.publicKey;
      this.address = publicKeyToAddress(this.publicKey);
    } else {
      // Generate new wallet
      const keypair = generateKeypair();
      this.publicKey = keypair.publicKey;
      this.privateKey = keypair.privateKey;
      this.address = publicKeyToAddress(this.publicKey);
    }
  }

  /**
   * Create a new random wallet
   */
  static createRandom(): Wallet {
    return new Wallet();
  }

  /**
   * Import wallet from private key
   */
  static fromPrivateKey(privateKey: string): Wallet {
    return new Wallet(privateKey);
  }

  /**
   * Export private key (use carefully!)
   */
  exportPrivateKey(): string {
    return this.privateKey;
  }

  /**
   * Sign a transaction
   */
  signTransaction(tx: {
    to: string | null;
    value: string | number;
    data?: string;
    nonce: number;
  }): Transaction {
    const transaction = {
      from: this.address,
      to: tx.to,
      value: typeof tx.value === 'number' ? toBaseUnits(tx.value) : tx.value,
      data: tx.data || '',
      nonce: tx.nonce
    };

    const hash = hashTransaction(transaction);
    const signature = signMessage(hash, this.privateKey);

    return {
      ...transaction,
      hash,
      signature
    };
  }

  /**
   * Sign arbitrary data
   */
  sign(data: string): string {
    return signMessage(data, this.privateKey);
  }

  /**
   * Get wallet info
   */
  getInfo(): { address: string; publicKey: string } {
    return {
      address: this.address,
      publicKey: this.publicKey
    };
  }
}
