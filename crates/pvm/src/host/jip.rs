//! JIP specified host calls

use crate::{host::Exit, result::State, Result};

/// Call the host function
#[tracing::instrument(skip(state))]
pub fn log<Memory: crate::Memory>(state: &mut State<Memory>) -> Result<u64> {
    let level = state.registers[7];
    let target = if state.registers[8] == 0 && state.registers[9] == 0 {
        None
    } else {
        let mut buf = [0; 16];
        buf[..8].copy_from_slice(&state.registers[8].to_le_bytes());
        buf[8..16].copy_from_slice(&state.registers[9].to_le_bytes());
        Some(String::from_utf8_lossy(&buf).to_string())
    };

    let message = {
        let mut buf = [0; 16];
        buf[..8].copy_from_slice(&state.registers[10].to_le_bytes());
        buf[8..16].copy_from_slice(&state.registers[11].to_le_bytes());
        String::from_utf8_lossy(&buf).to_string()
    };

    if let Some(target) = target {
        tracing::info!(level = level, target = target, message = message);
    }

    tracing::info!(level = level, message = message);
    Ok(Exit::Ok as u64)
}
