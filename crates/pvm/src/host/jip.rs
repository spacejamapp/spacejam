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
#[tracing::instrument(skip(state))]
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
            Ok(data) => {
                tracing::debug!("message bytes: {:?}", data);
                Some(String::from_utf8_lossy(&data).to_string())
            }
            Err(reason) => return Err(reason),
        }
    } else {
        None
    };

    // Log the message with appropriate level
    match level {
        0 => {
            if let Some(target) = target {
                tracing::error!(target = target, message = message);
            } else {
                tracing::error!("{message}");
            }
        }
        1 => {
            if let Some(target) = target {
                tracing::warn!(target = target, message = message);
            } else {
                tracing::warn!("{message}");
            }
        }
        2 => {
            if let Some(target) = target {
                tracing::info!(target = target, message = message);
            } else {
                tracing::info!("{message}");
            }
        }
        3 => {
            if let Some(target) = target {
                tracing::debug!(target = target, message = message);
            } else {
                tracing::debug!("{message}");
            }
        }
        4 => {
            if let Some(target) = target {
                tracing::trace!(target = target, message = message);
            } else {
                tracing::trace!("{message}");
            }
        }
        _ => {
            // Invalid log level, treat as info
            if let Some(target) = target {
                tracing::info!(target = target, message = message);
            } else {
                tracing::info!("{message}");
            }
        }
    }

    Ok(Exit::Ok as u64)
}
