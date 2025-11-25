// Core types for ACT Chain SDK

export interface Block {
  number: number;
  hash: string;
  parent_hash: string;
  timestamp: number;
  proposer: string;
  transactions: Transaction[];
  state_root: string;
  receipts_root: string;
}

export interface Transaction {
  hash: string;
  from: string;
  to: string | null;
  value: string;
  data: string;
  nonce: number;
  signature: string;
  gas_limit?: number;
  gas_price?: string;
}

export interface TransactionReceipt {
  transaction_hash: string;
  block_number: number;
  block_hash: string;
  from: string;
  to: string | null;
  status: 'success' | 'failure';
  gas_used?: number;
  logs?: Log[];
  contract_address?: string;
}

export interface Log {
  address: string;
  topics: string[];
  data: string;
}

export interface Account {
  address: string;
  balance: string;
  nonce: number;
  code?: string;
}

export interface PeerInfo {
  id: string;
  address: string;
  connected: boolean;
}

export interface ValidatorInfo {
  address: string;
  stake: string;
  active: boolean;
  blocks_proposed: number;
  last_active: number;
}

export interface DelegationInfo {
  delegator: string;
  validator: string;
  amount: string;
  rewards: string;
}

export interface UnstakeRequest {
  validator: string;
  amount: string;
  unlock_time: number;
}

export interface Proposal {
  id: number;
  proposer: string;
  title: string;
  description: string;
  start_time: number;
  voting_end_time: number;
  execution_time: number;
  status: 'Pending' | 'Active' | 'Passed' | 'Rejected' | 'Executed';
  yes_votes: string;
  no_votes: string;
  action: ProposalAction;
}

export interface ProposalAction {
  type: string;
  data: any;
}

export interface Vote {
  proposal_id: number;
  voter: string;
  vote: boolean;
  weight: string;
  timestamp: number;
}

export interface ContractDeployment {
  code: string;
  constructor_args?: any[];
}

export interface ContractCall {
  address: string;
  method: string;
  args?: any[];
}

export interface RPCRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number;
}

export interface RPCResponse<T = any> {
  jsonrpc: string;
  result?: T;
  error?: RPCError;
  id: number;
}

export interface RPCError {
  code: number;
  message: string;
  data?: any;
}
