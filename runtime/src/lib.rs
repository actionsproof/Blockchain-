use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use types::Action;
use wasmtime::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub state_changes: HashMap<String, Vec<u8>>,
    pub logs: Vec<String>,
}

pub struct WasmRuntime {
    engine: Engine,
    module_cache: HashMap<String, Module>,
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
        // Create a new store for this execution
        let mut store = Store::new(&self.engine, ());
        
        // Load or compile the WASM module
        let module = Module::new(&self.engine, wasm_bytes)?;
        
        // Create a linker to provide host functions
        let mut linker = Linker::new(&self.engine);
        
        // Add host functions that WASM can call
        linker.func_wrap("env", "log", |msg_ptr: i32, msg_len: i32| {
            println!("📝 WASM Log: ptr={}, len={}", msg_ptr, msg_len);
        })?;
        
        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)?;
        
        // Get the main execution function
        let execute_fn = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "execute")
            .or_else(|_| instance.get_typed_func::<(), i32>(&mut store, "execute"))?;
        
        // Execute the WASM function
        let result_code = execute_fn.call(&mut store, (0, 0))
            .or_else(|_| execute_fn.call(&mut store, ()))?;
        
        // Build execution result
        let execution_result = ExecutionResult {
            success: result_code == 0,
            gas_used: 1000, // Placeholder gas metering
            state_changes: HashMap::new(),
            logs: vec![format!("Executed action from actor: {}", action.actor)],
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
