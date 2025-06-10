//! JIP specified host calls

use crate::{host::Exit, Result, State};

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
            Ok(data) => Some(String::from_utf8_lossy(&data).to_string()),
            Err(reason) => return Err(reason),
        }
    } else {
        None
    };

    let lvl = match level {
        0 => log::Level::Error,
        1 => log::Level::Warn,
        2 => log::Level::Info,
        3 => log::Level::Debug,
        4 => log::Level::Trace,
        _ => log::Level::Info,
    };

    if let Some(target) = target {
        log::log!(target: &target, lvl,  "{message}");
    } else {
        log::log!(lvl, "{message}");
    }

    Ok(Exit::Ok as u64)
}
