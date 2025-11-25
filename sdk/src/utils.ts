import * as nacl from 'tweetnacl';
import * as bs58 from 'bs58';
import { Buffer } from 'buffer';

/**
 * Generate a new Ed25519 keypair for ACT Chain
 */
export function generateKeypair(): { publicKey: string; privateKey: string } {
  const keypair = nacl.sign.keyPair();
  
  return {
    publicKey: bs58.encode(keypair.publicKey),
    privateKey: bs58.encode(keypair.secretKey)
  };
}

/**
 * Sign a message with a private key
 */
export function signMessage(message: string, privateKey: string): string {
  const messageBytes = Buffer.from(message, 'utf8');
  const privateKeyBytes = bs58.decode(privateKey);
  
  const signature = nacl.sign.detached(messageBytes, privateKeyBytes);
  
  return bs58.encode(signature);
}

/**
 * Verify a signature
 */
export function verifySignature(
  message: string,
  signature: string,
  publicKey: string
): boolean {
  try {
    const messageBytes = Buffer.from(message, 'utf8');
    const signatureBytes = bs58.decode(signature);
    const publicKeyBytes = bs58.decode(publicKey);
    
    return nacl.sign.detached.verify(messageBytes, signatureBytes, publicKeyBytes);
  } catch {
    return false;
  }
}

/**
 * Derive address from public key (for ACT Chain, address = public key)
 */
export function publicKeyToAddress(publicKey: string): string {
  return publicKey;
}

/**
 * Convert ACT amount to smallest unit (1 ACT = 10^18 units)
 */
export function toBaseUnits(amount: number | string): string {
  const decimals = 18;
  const parts = amount.toString().split('.');
  const whole = parts[0];
  const fraction = (parts[1] || '').padEnd(decimals, '0').slice(0, decimals);
  
  return BigInt(whole + fraction).toString();
}

/**
 * Convert from smallest unit to ACT
 */
export function fromBaseUnits(amount: string | number): string {
  const decimals = 18;
  const str = amount.toString().padStart(decimals + 1, '0');
  const whole = str.slice(0, -decimals) || '0';
  const fraction = str.slice(-decimals).replace(/0+$/, '');
  
  return fraction ? `${whole}.${fraction}` : whole;
}

/**
 * Create transaction hash
 */
export function hashTransaction(tx: {
  from: string;
  to: string | null;
  value: string;
  data: string;
  nonce: number;
}): string {
  const txString = JSON.stringify(tx);
  const hash = nacl.hash(Buffer.from(txString, 'utf8'));
  return bs58.encode(hash);
}

/**
 * Validate address format
 */
export function isValidAddress(address: string): boolean {
  try {
    const decoded = bs58.decode(address);
    return decoded.length === 32; // Ed25519 public key is 32 bytes
  } catch {
    return false;
  }
}

/**
 * Format balance for display
 */
export function formatBalance(balance: string, decimals: number = 18): string {
  const units = fromBaseUnits(balance);
  const num = parseFloat(units);
  
  if (num >= 1000000) {
    return `${(num / 1000000).toFixed(2)}M`;
  } else if (num >= 1000) {
    return `${(num / 1000).toFixed(2)}K`;
  } else {
    return num.toFixed(4);
  }
}
