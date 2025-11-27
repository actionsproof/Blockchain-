//! WASM Runtime Execution Engine
//! 
//! Provides execution environment for WebAssembly smart contracts
//! with gas metering, host functions, and reward distribution.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasmtime::*;

/// Execution result with gas tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub logs: Vec<String>,
    pub state_changes: HashMap<String, Vec<u8>>,
}

/// Gas configuration for operations
#[derive(Debug, Clone)]
pub struct GasConfig {
    pub base_cost: u64,
    pub memory_cost_per_page: u64,
    pub call_cost: u64,
    pub storage_write_cost: u64,
    pub storage_read_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            base_cost: 1000,
            memory_cost_per_page: 100,
            call_cost: 5000,
            storage_write_cost: 2000,
            storage_read_cost: 200,
        }
    }
}

/// Execution context for contract calls
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub caller: String,
    pub contract_address: String,
    pub value: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub block_height: u64,
    pub block_timestamp: u64,
    pub state: HashMap<String, Vec<u8>>,
    pub logs: Vec<String>,
}

impl ExecutionContext {
    pub fn new(caller: String, contract_address: String, gas_limit: u64) -> Self {
        Self {
            caller,
            contract_address,
            value: 0,
            gas_limit,
            gas_used: 0,
            block_height: 0,
            block_timestamp: 0,
            state: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    /// Consume gas
    pub fn consume_gas(&mut self, amount: u64) -> Result<()> {
        if self.gas_used + amount > self.gas_limit {
            return Err(anyhow!("Out of gas"));
        }
        self.gas_used += amount;
        Ok(())
    }
    
    /// Get remaining gas
    pub fn gas_remaining(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }
}

/// WASM Runtime engine
pub struct WasmRuntime {
    engine: Engine,
    gas_config: GasConfig,
}

impl WasmRuntime {
    /// Create new WASM runtime
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true); // Enable gas metering
        
        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            gas_config: GasConfig::default(),
        })
    }
    
    /// Execute WASM contract
    pub fn execute_contract(
        &self,
        wasm_bytes: &[u8],
        function: &str,
        args: &[u8],
        mut context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Create module from WASM bytes
        let module = Module::new(&self.engine, wasm_bytes)?;
        
        // Create store with gas tracking
        let mut store = Store::new(&self.engine, context.clone());
        store.set_fuel(context.gas_limit)?;
        
        // Create linker and add host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker)?;
        
        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)?;
        
        // Get the function to call
        let func = instance.get_typed_func::<(), ()>(&mut store, function)?;
        
        // Execute the function
        match func.call(&mut store, ()) {
            Ok(_) => {
                let fuel_consumed = context.gas_limit - store.get_fuel()?;
                context.consume_gas(fuel_consumed)?;
                
                Ok(ExecutionResult {
                    success: true,
                    return_data: vec![],
                    gas_used: context.gas_used,
                    logs: context.logs.clone(),
                    state_changes: context.state.clone(),
                })
            }
            Err(e) => {
                Ok(ExecutionResult {
                    success: false,
                    return_data: format!("Execution error: {}", e).into_bytes(),
                    gas_used: context.gas_limit, // Consume all gas on error
                    logs: context.logs.clone(),
                    state_changes: HashMap::new(),
                })
            }
        }
    }
    
    /// Add host functions that contracts can call
    fn add_host_functions(&self, linker: &mut Linker<ExecutionContext>) -> Result<()> {
        // Storage read
        linker.func_wrap("env", "storage_read", 
            |mut caller: Caller<'_, ExecutionContext>, key_ptr: i32, key_len: i32| -> i32 {
                let memory = caller.get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow!("Memory not found"))?;
                
                let data = memory.data(&caller);
                let key_bytes = &data[key_ptr as usize..(key_ptr + key_len) as usize];
                let key = String::from_utf8_lossy(key_bytes).to_string();
                
                let context = caller.data_mut();
                context.consume_gas(context.gas_used + 200)?; // Gas for read
                
                if let Some(value) = context.state.get(&key) {
                    return Ok(value.len() as i32);
                }
                
                Ok(0)
            })?;
        
        // Storage write
        linker.func_wrap("env", "storage_write",
            |mut caller: Caller<'_, ExecutionContext>, key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32| -> i32 {
                let memory = caller.get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow!("Memory not found"))?;
                
                let data = memory.data(&caller);
                let key_bytes = &data[key_ptr as usize..(key_ptr + key_len) as usize];
                let key = String::from_utf8_lossy(key_bytes).to_string();
                
                let val_bytes = &data[val_ptr as usize..(val_ptr + val_len) as usize];
                
                let context = caller.data_mut();
                context.consume_gas(context.gas_used + 2000)?; // Gas for write
                
                context.state.insert(key, val_bytes.to_vec());
                
                Ok(0)
            })?;
        
        // Emit log
        linker.func_wrap("env", "log",
            |mut caller: Caller<'_, ExecutionContext>, msg_ptr: i32, msg_len: i32| -> i32 {
                let memory = caller.get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow!("Memory not found"))?;
                
                let data = memory.data(&caller);
                let msg_bytes = &data[msg_ptr as usize..(msg_ptr + msg_len) as usize];
                let msg = String::from_utf8_lossy(msg_bytes).to_string();
                
                let context = caller.data_mut();
                context.consume_gas(context.gas_used + 100)?;
                
                context.logs.push(msg);
                
                Ok(0)
            })?;
        
        // Get caller address
        linker.func_wrap("env", "get_caller",
            |caller: Caller<'_, ExecutionContext>| -> i32 {
                // Return pointer to caller address in memory
                // In real implementation, write to contract memory
                0
            })?;
        
        // Get block height
        linker.func_wrap("env", "get_block_height",
            |caller: Caller<'_, ExecutionContext>| -> i64 {
                caller.data().block_height as i64
            })?;
        
        // Get block timestamp
        linker.func_wrap("env", "get_block_timestamp",
            |caller: Caller<'_, ExecutionContext>| -> i64 {
                caller.data().block_timestamp as i64
            })?;
        
        Ok(())
    }
    
    /// Deploy a new contract
    pub fn deploy_contract(
        &self,
        wasm_bytes: &[u8],
        constructor_args: &[u8],
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Verify WASM is valid
        Module::new(&self.engine, wasm_bytes)?;
        
        // Execute constructor if exists
        self.execute_contract(wasm_bytes, "constructor", constructor_args, context)
    }
}

/// Execute action block (legacy placeholder for compatibility)
pub fn execute_action_block(payload: &[u8]) -> Result<ExecutionResult> {
    // Simple execution for basic actions
    Ok(ExecutionResult {
        success: true,
        return_data: vec![],
        gas_used: 1000,
        logs: vec![format!("Executed action: {} bytes", payload.len())],
        state_changes: HashMap::new(),
    })
}

/// Calculate reward distribution for block
pub fn calculate_rewards(
    block_height: u64,
    base_reward: u64,
    transaction_fees: u64,
    validator: &str,
    delegators: &[(String, u64)], // (address, stake)
) -> HashMap<String, u64> {
    let mut rewards = HashMap::new();
    
    // Total reward = base + fees
    let total_reward = base_reward + transaction_fees;
    
    // Validator takes commission (e.g., 10%)
    let validator_commission = total_reward / 10;
    rewards.insert(validator.to_string(), validator_commission);
    
    // Distribute remaining to delegators proportionally
    let delegator_pool = total_reward - validator_commission;
    let total_stake: u64 = delegators.iter().map(|(_, stake)| stake).sum();
    
    if total_stake > 0 {
        for (address, stake) in delegators {
            let delegator_reward = (delegator_pool * stake) / total_stake;
            *rewards.entry(address.clone()).or_insert(0) += delegator_reward;
        }
    } else {
        // No delegators, validator gets everything
        *rewards.get_mut(validator).unwrap() += delegator_pool;
    }
    
    rewards
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_execution_context() {
        let mut ctx = ExecutionContext::new(
            "ACT-alice".to_string(),
            "ACT-contract".to_string(),
            10000,
        );
        
        assert_eq!(ctx.gas_remaining(), 10000);
        
        ctx.consume_gas(1000).unwrap();
        assert_eq!(ctx.gas_used, 1000);
        assert_eq!(ctx.gas_remaining(), 9000);
        
        // Should fail when out of gas
        assert!(ctx.consume_gas(10000).is_err());
    }
    
    #[test]
    fn test_reward_calculation() {
        let rewards = calculate_rewards(
            100,
            1_000_000_000, // 1 ACT base reward
            100_000_000,   // 0.1 ACT fees
            "ACT-validator1",
            &[
                ("ACT-delegator1".to_string(), 50_000_000),
                ("ACT-delegator2".to_string(), 50_000_000),
            ],
        );
        
        // Validator gets 10% commission
        assert_eq!(rewards.get("ACT-validator1").unwrap(), &110_000_000);
        
        // Each delegator gets 45% (50% of remaining 90%)
        assert_eq!(rewards.get("ACT-delegator1").unwrap(), &495_000_000);
        assert_eq!(rewards.get("ACT-delegator2").unwrap(), &495_000_000);
    }
    
    #[test]
    fn test_action_execution() {
        let result = execute_action_block(b"test payload").unwrap();
        assert!(result.success);
        assert_eq!(result.gas_used, 1000);
        assert_eq!(result.logs.len(), 1);
    }
}
