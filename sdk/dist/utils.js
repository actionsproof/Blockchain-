"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.generateKeypair = generateKeypair;
exports.signMessage = signMessage;
exports.verifySignature = verifySignature;
exports.publicKeyToAddress = publicKeyToAddress;
exports.toBaseUnits = toBaseUnits;
exports.fromBaseUnits = fromBaseUnits;
exports.hashTransaction = hashTransaction;
exports.isValidAddress = isValidAddress;
exports.formatBalance = formatBalance;
const nacl = __importStar(require("tweetnacl"));
const bs58 = __importStar(require("bs58"));
const buffer_1 = require("buffer");
/**
 * Generate a new Ed25519 keypair for ACT Chain
 */
function generateKeypair() {
    const keypair = nacl.sign.keyPair();
    return {
        publicKey: bs58.encode(keypair.publicKey),
        privateKey: bs58.encode(keypair.secretKey)
    };
}
/**
 * Sign a message with a private key
 */
function signMessage(message, privateKey) {
    const messageBytes = buffer_1.Buffer.from(message, 'utf8');
    const privateKeyBytes = bs58.decode(privateKey);
    const signature = nacl.sign.detached(messageBytes, privateKeyBytes);
    return bs58.encode(signature);
}
/**
 * Verify a signature
 */
function verifySignature(message, signature, publicKey) {
    try {
        const messageBytes = buffer_1.Buffer.from(message, 'utf8');
        const signatureBytes = bs58.decode(signature);
        const publicKeyBytes = bs58.decode(publicKey);
        return nacl.sign.detached.verify(messageBytes, signatureBytes, publicKeyBytes);
    }
    catch {
        return false;
    }
}
/**
 * Derive address from public key (for ACT Chain, address = public key)
 */
function publicKeyToAddress(publicKey) {
    return publicKey;
}
/**
 * Convert ACT amount to smallest unit (1 ACT = 10^18 units)
 */
function toBaseUnits(amount) {
    const decimals = 18;
    const parts = amount.toString().split('.');
    const whole = parts[0];
    const fraction = (parts[1] || '').padEnd(decimals, '0').slice(0, decimals);
    return BigInt(whole + fraction).toString();
}
/**
 * Convert from smallest unit to ACT
 */
function fromBaseUnits(amount) {
    const decimals = 18;
    const str = amount.toString().padStart(decimals + 1, '0');
    const whole = str.slice(0, -decimals) || '0';
    const fraction = str.slice(-decimals).replace(/0+$/, '');
    return fraction ? `${whole}.${fraction}` : whole;
}
/**
 * Create transaction hash
 */
function hashTransaction(tx) {
    const txString = JSON.stringify(tx);
    const hash = nacl.hash(buffer_1.Buffer.from(txString, 'utf8'));
    return bs58.encode(hash);
}
/**
 * Validate address format
 */
function isValidAddress(address) {
    try {
        const decoded = bs58.decode(address);
        return decoded.length === 32; // Ed25519 public key is 32 bytes
    }
    catch {
        return false;
    }
}
/**
 * Format balance for display
 */
function formatBalance(balance, decimals = 18) {
    const units = fromBaseUnits(balance);
    const num = parseFloat(units);
    if (num >= 1000000) {
        return `${(num / 1000000).toFixed(2)}M`;
    }
    else if (num >= 1000) {
        return `${(num / 1000).toFixed(2)}K`;
    }
    else {
        return num.toFixed(4);
    }
}
