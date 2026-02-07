//! JIP specified host calls

use crate::{host::Exit, Argument, Result};
use account::Account;

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
#[tracing::instrument(skip_all, name = "service", parent = None)]
pub fn log(ctx: &mut impl Argument) -> Result<u64> {
    let level = ctx.rget(7);
    let target_addr = ctx.rget(8) as u32;
    let target_len = ctx.rget(9) as u32;
    let msg_addr = ctx.rget(10) as u32;
    let msg_len = ctx.rget(11) as u32;
    let message = {
        let mut buf = vec![0u8; msg_len as usize];
        match ctx.read_into(msg_addr, &mut buf) {
            Ok(()) => String::from_utf8_lossy(&buf).to_string(),
            Err(reason) => {
                tracing::error!(
                    "Failed to read message bytes at addr=0x{:x}, len={}: {:?}",
                    msg_addr,
                    msg_len,
                    reason
                );
                return Err(reason.into());
            }
        }
    };

    // Read target if provided (for structured logging)
    let target = if target_len > 0 {
        let mut buf = vec![0u8; target_len as usize];
        match ctx.read_into(target_addr, &mut buf) {
            Ok(()) => String::from_utf8_lossy(&buf).to_string(),
            Err(reason) => return Err(reason.into()),
        }
    } else {
        let Ok(service) = ctx.this() else {
            return Ok(Exit::What as u64);
        };
        format!("service-{}", service.index())
    };

    let level = match level {
        0 => log::Level::Error,
        1 => log::Level::Warn,
        2 => log::Level::Info,
        3 => log::Level::Debug,
        4 => log::Level::Trace,
        _ => log::Level::Warn,
    };

    let gas = ctx.gas();
    log::log!(target: &target, level, "{message} (gas={gas})");
    Ok(Exit::What as u64)
}
