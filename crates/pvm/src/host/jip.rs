//! JIP specified host calls

use crate::{host::Exit, invocation::State, Result};

/// JIP-1 logging host function implementation
///
/// Implements the JIP-1 specification for logging host calls
/// with call number 100 as defined in the JAM specification.
///
/// JIP-1 specification compliance:
/// r7 = level (0=ERROR, 1=WARN, 2=INFO, 3=DEBUG, 4=TRACE)
/// r8 = target address (optional, for structured logging)
/// r9 = target length
/// r10 = message address
/// r11 = message length
#[tracing::instrument(skip_all, name = "program", parent = None)]
pub fn log<Memory: crate::Memory>(state: &mut State<Memory>) -> Result<u64> {
    let level = state.registers[7];
    let target_addr = state.registers[8] as u32;
    let target_len = state.registers[9] as u32;
    let msg_addr = state.registers[10] as u32;
    let msg_len = state.registers[11] as u32;
    let message = match state.memory.read_bytes(msg_addr, msg_len) {
        Ok(data) => {
            let msg_str = String::from_utf8_lossy(&data).to_string();
            msg_str
        }
        Err(reason) => {
            tracing::error!(
                "Failed to read message bytes at addr=0x{:x}, len={}: {:?}",
                msg_addr,
                msg_len,
                reason
            );
            return Err(reason);
        }
    };

    // Read target if provided (for structured logging)
    let target = if target_len > 0 {
        match state.memory.read_bytes(target_addr, target_len) {
            Ok(data) => String::from_utf8_lossy(&data).to_string(),
            Err(reason) => return Err(reason),
        }
    } else {
        Default::default()
    };

    // Convert numeric level to log::Level
    match level {
        0 => tracing::error!(target = target, "{message}"),
        1 => tracing::warn!(target = target, "{message}"),
        2 => tracing::info!(target = target, "{message}"),
        3 => tracing::debug!(target = target, "{message}"),
        4 => tracing::trace!(target = target, "{message}"),
        _ => tracing::warn!(target = target, "{message}"),
    }

    Ok(Exit::Ok as u64)
}
