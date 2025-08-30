//! ABI for host functions
#![allow(improper_ctypes_definitions)]

use pvm::Argument;

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
