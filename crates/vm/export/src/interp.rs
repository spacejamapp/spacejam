//! SpaceVM common library

use crate::Buffer;

/// (ΨA): Accumulation invocation
#[unsafe(no_mangle)]
pub extern "C" fn interp_accumulate(args: Buffer) -> Buffer {
    crate::accumulate::<pvmi::Interpreter>(args)
}

/// (ΨR): Refine invocation
#[unsafe(no_mangle)]
pub extern "C" fn interp_refine(args: Buffer) -> Buffer {
    crate::refine::<pvmi::Interpreter>(args)
}

/// (ΨI): Is-Authorized invocation
#[unsafe(no_mangle)]
pub extern "C" fn interp_authorize(buffer: Buffer) -> Buffer {
    crate::authorize::<pvmi::Interpreter>(buffer)
}
