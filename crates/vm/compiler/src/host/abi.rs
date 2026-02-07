//! ABI for host functions

use pvm::Argument;

/// Host function trampoline
///
/// This function is called from JIT-compiled code to invoke host functions.
/// It casts the raw context pointer and delegates to [pvm::host::call]
pub extern "C" fn ecalli<X: Argument>(index: u32, ctx: *mut u8) -> u8 {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    if context.gas < 0 {
        return 4;
    }

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

/// Memory get function trampoline
pub extern "C" fn mget<X: Argument>(ctx: *mut u8, address: u32, len: u8) -> u64 {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    let bytes = context.memory.read_bytes(address, len as u32);

    match len {
        1 => u8::from_le_bytes([bytes[0]]) as u64,
        2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u64,
        4 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64,
        8 => u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]),
        _ => panic!("invalid value length"),
    }
}

/// Memory set function trampoline
pub extern "C" fn mset<X: Argument>(ctx: *mut u8, address: u32, value: i64, len: u8) {
    let context = unsafe { &mut *(ctx as *mut pvm::Context<X, crate::Memory>) };
    let bytes = value.to_le_bytes();
    context.memory.write_bytes(address, &bytes[..len as usize]);
}
