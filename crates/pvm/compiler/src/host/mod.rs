//! Host call trampoline

pub use abi::*;
use pvm::Argument;
use std::sync::Once;
pub use translator::host as sig;

/// The dispatch table of host call symbols
static mut DISPATCH_TABLE: [*const u8; 4] = [std::ptr::null(); 4];
static INIT: Once = Once::new();

/// The name of the call function
pub const CALL: &str = "call";
/// The name of the sbrk function
pub const SBRK: &str = "sbrk";
/// The name of the mget function
pub const MGET: &str = "mget";
/// The name of the mset function
pub const MSET: &str = "mset";

mod abi;

/// The table of host call symbols
pub fn table<X: Argument>() -> i64 {
    INIT.call_once(|| unsafe {
        DISPATCH_TABLE[0] = abi::ecalli::<X> as *const u8;
        DISPATCH_TABLE[1] = abi::sbrk::<X> as *const u8;
        DISPATCH_TABLE[2] = abi::mget::<X> as *const u8;
        DISPATCH_TABLE[3] = abi::mset::<X> as *const u8;
    });

    core::ptr::addr_of!(DISPATCH_TABLE) as i64
}
