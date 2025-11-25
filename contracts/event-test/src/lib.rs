#![no_std]

// Import host functions from ACT runtime
extern "C" {
    fn emit_event(topic_ptr: *const u8, topic_len: u32, data_ptr: *const u8, data_len: u32) -> i32;
    fn log(msg_ptr: *const u8, msg_len: u32);
    fn storage_write(key_ptr: *const u8, key_len: u32, val_ptr: *const u8, val_len: u32) -> i32;
    fn storage_read(key_ptr: *const u8, key_len: u32, val_ptr: *mut u8) -> i32;
}

/// Event: Transfer
/// Emitted when tokens are transferred
#[no_mangle]
pub extern "C" fn transfer(to_ptr: *const u8, to_len: u32, amount: u64) -> i32 {
    unsafe {
        // Log the transfer
        let log_msg = b"Transfer initiated";
        log(log_msg.as_ptr(), log_msg.len() as u32);
        
        // Emit Transfer event
        // Topic: "Transfer"
        let topic = b"Transfer";
        
        // Data: to_address + amount (simplified)
        let mut data = [0u8; 8];
        data[..8].copy_from_slice(&amount.to_le_bytes());
        
        let result = emit_event(
            topic.as_ptr(),
            topic.len() as u32,
            data.as_ptr(),
            data.len() as u32,
        );
        
        if result != 0 {
            return -1; // Error
        }
        
        // Update balance in storage (simplified)
        let key = b"balance";
        let value = amount.to_le_bytes();
        storage_write(key.as_ptr(), key.len() as u32, value.as_ptr(), value.len() as u32);
        
        0 // Success
    }
}

/// Event: Approval
/// Emitted when approval is granted
#[no_mangle]
pub extern "C" fn approve(spender_ptr: *const u8, spender_len: u32, amount: u64) -> i32 {
    unsafe {
        // Log the approval
        let log_msg = b"Approval granted";
        log(log_msg.as_ptr(), log_msg.len() as u32);
        
        // Emit Approval event
        let topic = b"Approval";
        let data = amount.to_le_bytes();
        
        let result = emit_event(
            topic.as_ptr(),
            topic.len() as u32,
            data.as_ptr(),
            data.len() as u32,
        );
        
        if result != 0 {
            return -1;
        }
        
        0 // Success
    }
}

/// Event: ContractCreated
/// Emitted when contract is initialized
#[no_mangle]
pub extern "C" fn execute() -> i32 {
    unsafe {
        // Log initialization
        let log_msg = b"Contract initialized";
        log(log_msg.as_ptr(), log_msg.len() as u32);
        
        // Emit ContractCreated event
        let topic = b"ContractCreated";
        let data = b"v1.0.0";
        
        emit_event(
            topic.as_ptr(),
            topic.len() as u32,
            data.as_ptr(),
            data.len() as u32,
        );
        
        0 // Success
    }
}

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
