import { ActClient } from './client';
import { Wallet } from './wallet';

/**
 * Contract class for deploying and interacting with WASM smart contracts
 */
export class Contract {
  constructor(
    private client: ActClient,
    private wallet: Wallet,
    public readonly address?: string
  ) {}

  /**
   * Deploy a new contract
   */
  static async deploy(
    client: ActClient,
    wallet: Wallet,
    wasmCode: Uint8Array | string,
    constructorArgs: any[] = []
  ): Promise<Contract> {
    // Get nonce
    const nonce = await client.getNonce(wallet.address);

    // Prepare deployment data
    const code = typeof wasmCode === 'string' ? wasmCode : Buffer.from(wasmCode).toString('base64');
    const data = JSON.stringify({
      type: 'deploy',
      code,
      args: constructorArgs
    });

    // Sign and send transaction
    const tx = wallet.signTransaction({
      to: null, // null address means contract deployment
      value: '0',
      data,
      nonce
    });

    const txHash = await client.sendTransaction(tx);

    // Wait for deployment
    const receipt = await client.waitForTransaction(txHash);

    if (receipt.status !== 'success' || !receipt.contract_address) {
      throw new Error('Contract deployment failed');
    }

    return new Contract(client, wallet, receipt.contract_address);
  }

  /**
   * Call contract method (state-changing)
   */
  async call(method: string, args: any[] = [], value: string = '0'): Promise<string> {
    if (!this.address) {
      throw new Error('Contract not deployed');
    }

    const nonce = await this.client.getNonce(this.wallet.address);

    const data = JSON.stringify({
      type: 'call',
      method,
      args
    });

    const tx = this.wallet.signTransaction({
      to: this.address,
      value,
      data,
      nonce
    });

    return this.client.sendTransaction(tx);
  }

  /**
   * Query contract method (read-only)
   */
  async query(method: string, args: any[] = []): Promise<any> {
    if (!this.address) {
      throw new Error('Contract not deployed');
    }

    const data = JSON.stringify({
      type: 'query',
      method,
      args
    });

    // Use eth_call for read-only operations
    const result = await this.client.ethCall({
      to: this.address,
      data
    });

    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }

  /**
   * Get contract address
   */
  getAddress(): string {
    if (!this.address) {
      throw new Error('Contract not deployed');
    }
    return this.address;
  }
}

/**
 * ACT-20 Token Contract Helper
 */
export class Act20Contract extends Contract {
  /**
   * Deploy new ACT-20 token
   */
  static async deployToken(
    client: ActClient,
    wallet: Wallet,
    wasmCode: Uint8Array | string,
    name: string,
    symbol: string,
    decimals: number,
    initialSupply: string,
    mintable: boolean = true,
    burnable: boolean = true
  ): Promise<Act20Contract> {
    const contract = await Contract.deploy(client, wallet, wasmCode, [
      name,
      symbol,
      decimals,
      initialSupply,
      wallet.address,
      mintable,
      burnable
    ]);

    return new Act20Contract(client, wallet, contract.address);
  }

  /**
   * Get token name
   */
  async name(): Promise<string> {
    return this.query('name');
  }

  /**
   * Get token symbol
   */
  async symbol(): Promise<string> {
    return this.query('symbol');
  }

  /**
   * Get token decimals
   */
  async decimals(): Promise<number> {
    return this.query('decimals');
  }

  /**
   * Get total supply
   */
  async totalSupply(): Promise<string> {
    return this.query('total_supply');
  }

  /**
   * Get balance of address
   */
  async balanceOf(address: string): Promise<string> {
    return this.query('balance_of', [address]);
  }

  /**
   * Transfer tokens
   */
  async transfer(to: string, amount: string): Promise<string> {
    return this.call('transfer', [to, amount]);
  }

  /**
   * Approve spender
   */
  async approve(spender: string, amount: string): Promise<string> {
    return this.call('approve', [spender, amount]);
  }

  /**
   * Get allowance
   */
  async allowance(owner: string, spender: string): Promise<string> {
    return this.query('allowance', [owner, spender]);
  }

  /**
   * Transfer from (requires approval)
   */
  async transferFrom(from: string, to: string, amount: string): Promise<string> {
    return this.call('transfer_from', [from, to, amount]);
  }

  /**
   * Mint tokens (if mintable)
   */
  async mint(to: string, amount: string): Promise<string> {
    return this.call('mint', [to, amount]);
  }

  /**
   * Burn tokens (if burnable)
   */
  async burn(amount: string): Promise<string> {
    return this.call('burn', [amount]);
  }
}

/**
 * DEX Contract Helper
 */
export class DexContract extends Contract {
  /**
   * Connect to existing DEX
   */
  static async connect(
    client: ActClient,
    wallet: Wallet,
    address: string
  ): Promise<DexContract> {
    return new DexContract(client, wallet, address);
  }

  /**
   * Add liquidity to pool
   */
  async addLiquidity(amountA: string, amountB: string): Promise<string> {
    return this.call('add_liquidity', [amountA, amountB]);
  }

  /**
   * Remove liquidity from pool
   */
  async removeLiquidity(liquidity: string): Promise<string> {
    return this.call('remove_liquidity', [liquidity]);
  }

  /**
   * Swap token A for token B
   */
  async swapAForB(amountIn: string): Promise<string> {
    return this.call('swap_a_for_b', [amountIn]);
  }

  /**
   * Swap token B for token A
   */
  async swapBForA(amountIn: string): Promise<string> {
    return this.call('swap_b_for_a', [amountIn]);
  }

  /**
   * Get quote for swap A to B
   */
  async getQuoteAForB(amountIn: string): Promise<string> {
    return this.query('get_quote_a_for_b', [amountIn]);
  }

  /**
   * Get quote for swap B to A
   */
  async getQuoteBForA(amountIn: string): Promise<string> {
    return this.query('get_quote_b_for_a', [amountIn]);
  }

  /**
   * Get pool reserves
   */
  async getReserves(): Promise<{ reserveA: string; reserveB: string }> {
    const result = await this.query('get_reserves');
    return {
      reserveA: result.reserve_a,
      reserveB: result.reserve_b
    };
  }

  /**
   * Get LP token balance
   */
  async getLPBalance(address: string): Promise<string> {
    return this.query('get_lp_balance', [address]);
  }

  /**
   * Get price of token A in terms of token B
   */
  async getPriceAInB(): Promise<string> {
    return this.query('get_price_a_in_b');
  }

  /**
   * Get price of token B in terms of token A
   */
  async getPriceBInA(): Promise<string> {
    return this.query('get_price_b_in_a');
  }
}
