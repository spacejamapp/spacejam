//! ABI for host functions

use crate::host::Value;
use pvm::Argument;
use translator::Exit;

/// Host function trampoline
///
/// This function is called from JIT-compiled code to invoke host functions.
/// It casts the raw context pointer and delegates to [pvm::host::call]
pub extern "C" fn call<X: Argument>(index: u32, ctx: *mut u8) -> u8 {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    match pvm::host::call(index, context) {
        pvm::Reason::Halt => 0,
        pvm::Reason::Panic(_) => 1,
        pvm::Reason::Fault { .. } => 2,
        pvm::Reason::HostCall(_) => 3,
        pvm::Reason::OOG => 4,
        pvm::Reason::Continue => 5,
    }
}

/// Host function trampoline
///
/// This function is called from JIT-compiled code to invoke host functions.
/// It casts the raw context pointer and delegates to [pvm::host::call]
pub extern "C" fn sbrk<X: Argument>(ctx: *mut u8, target: u8, increment: u8) {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    context.sbrk(target, increment);
}

/// Memory get trampoline
///
/// NOTE: this is for macos only since can't allocate memory > 2.5GB here.
pub extern "C" fn mget<X: Argument>(ctx: *mut u8, address: u32, offset: u32, ty: u8) -> *const i64 {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    let address = address + offset;
    let value = Value::from(ty);
    match context
        .read(address, value.bytes() as u32)
        .and_then(|bytes| value.as_u64(&bytes))
    {
        Ok(value) => [value as i64, Exit::Halt.code()].as_ptr(),
        Err(_) => [0, address as i64].as_ptr(),
    }
}

/// Memory set trampoline
///
/// NOTE: this is for macos only since can't allocate memory > 2.5GB here.
pub extern "C" fn mset<X: Argument>(
    ctx: *mut u8,
    address: u32,
    offset: u32,
    data: i64,
    ty: u8,
) -> i64 {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    let address = address + offset;
    let value = Value::from(ty);
    match context.write(address, &value.as_bytes(data)) {
        Ok(_) => 0,
        Err(_) => address as i64,
    }
}
