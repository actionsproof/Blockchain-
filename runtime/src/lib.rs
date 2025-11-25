use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use types::{Action, EventLog};
use wasmtime::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub state_changes: HashMap<String, Vec<u8>>,
    pub logs: Vec<String>,
    pub events: Vec<EventLog>,  // Event logs emitted during execution
    pub return_data: Vec<u8>,   // Data returned by contract
}

pub struct WasmRuntime {
    engine: Engine,
    module_cache: HashMap<String, Module>,
}

/// Execution context for WASM runtime
struct ExecutionContext {
    events: Vec<EventLog>,
    gas_used: u64,
    gas_limit: u64,
    contract_address: String,
    transaction_hash: String,
    block_height: u64,
    call_depth: u32,  // Track nested contract calls
}

impl ExecutionContext {
    fn new(contract_address: String, transaction_hash: String, block_height: u64, gas_limit: u64) -> Self {
        Self {
            events: Vec::new(),
            gas_used: 0,
            gas_limit,
            contract_address,
            transaction_hash,
            block_height,
            call_depth: 0,
        }
    }
    
    fn consume_gas(&mut self, amount: u64) -> Result<()> {
        self.gas_used += amount;
        if self.gas_used > self.gas_limit {
            return Err(anyhow!("Out of gas"));
        }
        Ok(())
    }
    
    fn emit_event(&mut self, topics: Vec<String>, data: Vec<u8>) {
        let log = EventLog::new(
            self.contract_address.clone(),
            topics,
            data,
            self.block_height,
            self.transaction_hash.clone(),
            self.events.len() as u32,
        );
        self.events.push(log);
    }
    
    fn can_call_contract(&self) -> bool {
        self.call_depth < 10  // Max call depth to prevent infinite recursion
    }
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_threads(false);
        config.wasm_reference_types(true);
        config.wasm_bulk_memory(true);
        
        let engine = Engine::new(&config)?;
        
        Ok(Self {
            engine,
            module_cache: HashMap::new(),
        })
    }

    pub fn execute_action(&mut self, action: &Action, wasm_bytes: &[u8]) -> Result<ExecutionResult> {
        // Create execution context
        let ctx = ExecutionContext::new(
            action.actor.clone(),
            "tx_hash_placeholder".to_string(),
            0,
            1_000_000,
        );
        
        // Create a new store with execution context
        let mut store = Store::new(&self.engine, ctx);
        
        // Load or compile the WASM module
        let module = Module::new(&self.engine, wasm_bytes)?;
        
        // Create a linker to provide host functions
        let mut linker = Linker::new(&self.engine);
        
        // Add host function: log message
        linker.func_wrap("env", "log", |mut caller: Caller<'_, ExecutionContext>, msg_ptr: i32, msg_len: i32| {
            caller.data_mut().consume_gas(100).ok();
            println!("📝 WASM Log: ptr={}, len={}", msg_ptr, msg_len);
        })?;
        
        // Add host function: emit event
        linker.func_wrap(
            "env",
            "emit_event",
            |mut caller: Caller<'_, ExecutionContext>, 
             topic_ptr: i32, 
             topic_len: i32,
             data_ptr: i32,
             data_len: i32| -> i32 {
                // Consume gas for event emission
                if caller.data_mut().consume_gas(1000).is_err() {
                    return -1; // Out of gas
                }
                
                // Read memory (simplified - in production would read from WASM memory)
                let topics = vec![format!("topic_{}_{}", topic_ptr, topic_len)];
                let data = vec![0u8; data_len as usize];
                
                caller.data_mut().emit_event(topics, data);
                println!("📢 Event emitted: topic_ptr={}, data_len={}", topic_ptr, data_len);
                
                0 // Success
            },
        )?;
        
        // Add host function: storage write
        linker.func_wrap(
            "env",
            "storage_write",
            |mut caller: Caller<'_, ExecutionContext>, key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32| -> i32 {
                if caller.data_mut().consume_gas(5000).is_err() {
                    return -1;
                }
                println!("💾 Storage write: key_ptr={}, val_ptr={}", key_ptr, val_ptr);
                0
            },
        )?;
        
        // Add host function: storage read
        linker.func_wrap(
            "env",
            "storage_read",
            |mut caller: Caller<'_, ExecutionContext>, key_ptr: i32, key_len: i32, val_ptr: i32| -> i32 {
                if caller.data_mut().consume_gas(2000).is_err() {
                    return -1;
                }
                println!("📖 Storage read: key_ptr={}", key_ptr);
                0 // Return bytes written
            },
        )?;
        
        // Add host function: call another contract
        linker.func_wrap(
            "env",
            "call_contract",
            |mut caller: Caller<'_, ExecutionContext>,
             contract_addr_ptr: i32,
             contract_addr_len: i32,
             method_ptr: i32,
             method_len: i32,
             args_ptr: i32,
             args_len: i32,
             gas: i64| -> i32 {
                // Check call depth
                if !caller.data().can_call_contract() {
                    println!("❌ Max call depth exceeded");
                    return -1;
                }
                
                // Consume gas for the call
                let gas_for_call = if gas > 0 { gas as u64 } else { 100_000 };
                if caller.data_mut().consume_gas(gas_for_call).is_err() {
                    return -2; // Out of gas
                }
                
                // Increment call depth
                caller.data_mut().call_depth += 1;
                
                println!("🔗 Contract call: addr_ptr={}, method_ptr={}, args_len={}, gas={}",
                    contract_addr_ptr, method_ptr, args_len, gas_for_call);
                
                // Simulate successful call (in production would load and execute target contract)
                caller.data_mut().call_depth -= 1;
                
                0 // Success
            },
        )?;
        
        // Add host function: get caller address
        linker.func_wrap(
            "env",
            "get_caller",
            |mut caller: Caller<'_, ExecutionContext>, out_ptr: i32| -> i32 {
                if caller.data_mut().consume_gas(100).is_err() {
                    return -1;
                }
                println!("👤 Get caller: out_ptr={}", out_ptr);
                0 // Return address length
            },
        )?;
        
        // Add host function: get contract balance
        linker.func_wrap(
            "env",
            "get_balance",
            |mut caller: Caller<'_, ExecutionContext>, addr_ptr: i32, addr_len: i32| -> i64 {
                if caller.data_mut().consume_gas(100).is_err() {
                    return -1;
                }
                println!("💰 Get balance: addr_ptr={}", addr_ptr);
                1000000000000000000 // Return 1 ACT as example
            },
        )?;
        
        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)?;
        
        // Get the main execution function
        let result_code = if let Ok(execute_fn) = instance.get_typed_func::<(), i32>(&mut store, "execute") {
            execute_fn.call(&mut store, ())?
        } else if let Ok(execute_fn) = instance.get_typed_func::<(i32, i32), i32>(&mut store, "execute") {
            execute_fn.call(&mut store, (0, 0))?
        } else {
            return Err(anyhow!("No compatible execute function found in WASM module"));
        };
        
        // Extract context data
        let ctx = store.into_data();
        
        // Build execution result
        let execution_result = ExecutionResult {
            success: result_code == 0,
            gas_used: ctx.gas_used,
            state_changes: HashMap::new(),
            logs: vec![format!("Executed action from actor: {}", action.actor)],
            events: ctx.events,
            return_data: Vec::new(),
        };
        
        Ok(execution_result)
    }

    pub fn execute_action_with_state(
        &mut self,
        action: &Action,
        _state: &HashMap<String, Vec<u8>>,
    ) -> Result<ExecutionResult> {
        // For now, simulate execution without actual WASM bytecode
        println!(
            "🎮 Executing action from actor: {}, payload size: {} bytes",
            action.actor,
            action.payload.len()
        );
        
        let mut state_changes = HashMap::new();
        state_changes.insert(
            format!("balance_{}", action.actor),
            vec![100, 0, 0, 0, 0, 0, 0, 0], // Simulated balance update
        );
        
        let execution_result = ExecutionResult {
            success: true,
            gas_used: action.payload.len() as u64 * 10,
            state_changes,
            logs: vec![
                format!("Action executed successfully for actor: {}", action.actor),
                format!("Nonce: {}", action.nonce),
                format!("Payload size: {} bytes", action.payload.len()),
            ],
            events: Vec::new(),
            return_data: Vec::new(),
        };
        
        Ok(execution_result)
    }
}

pub fn execute_action_block(action: &Action) -> Result<ExecutionResult> {
    let mut runtime = WasmRuntime::new()?;
    let state = HashMap::new();
    runtime.execute_action_with_state(action, &state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_action_execution() {
        let action = Action {
            actor: "test_actor".to_string(),
            payload: vec![1, 2, 3, 4],
            nonce: 1,
        };
        
        let result = execute_action_block(&action);
        assert!(result.is_ok());
        
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert!(exec_result.gas_used > 0);
    }
}
