//! SpaceVM common library

use crate::Buffer;

/// (ΨA): Accumulation invocation
#[unsafe(no_mangle)]
pub extern "C" fn comp_accumulate(args: Buffer) -> Buffer {
    crate::accumulate::<pvmc::Compiler>(args)
}

/// (ΨR): Refine invocation
#[unsafe(no_mangle)]
pub extern "C" fn comp_refine(args: Buffer) -> Buffer {
    crate::refine::<pvmc::Compiler>(args)
}

/// (ΨI): Is-Authorized invocation
#[unsafe(no_mangle)]
pub extern "C" fn comp_authorize(buffer: Buffer) -> Buffer {
    crate::authorize::<pvmc::Compiler>(buffer)
}
