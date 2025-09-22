//! Exit codes

use cranelift::prelude::{FunctionBuilder, InstBuilder, Value, types};
use pvm::Reason;

/// Exit codes
pub enum Exit {
    /// Halt the program
    Halt,
    /// Fault with a page number
    Fault { address: Value },
    /// Out of gas
    OOG,

    // panic range: 100 - 0x10000 (65536)
    /// Trap occurred
    Trap,
    /// Invalid start program counter
    InvalidStartPC,
    /// Invalid jump target
    InvalidJumpTarget,
    /// Program not terminated
    ProgramNotTerminated,
    /// Host call panic
    HostCallPanicked,
}

impl Exit {
    /// Get the exit code
    pub fn code(&self) -> i64 {
        match self {
            Exit::Halt => 0,
            Exit::OOG => 4,
            Exit::Trap => 100,
            Exit::InvalidStartPC => 101,
            Exit::InvalidJumpTarget => 102,
            Exit::ProgramNotTerminated => 103,
            Exit::HostCallPanicked => 104,
            // Not supported here
            Exit::Fault { address: _ } => -1,
        }
    }
    /// Convert exit code to reason
    pub fn to_reason(code: i64) -> Reason {
        match code {
            0 => Reason::Halt,
            4 => Reason::OOG,

            // exit code of panic range: 100 - 0x10000 (65536)
            100 => Reason::Panic("trap occurred".to_string()),
            101 => Reason::Panic("invalid start program counter".to_string()),
            102 => Reason::Panic("invalid jump target".to_string()),
            103 => Reason::Panic("program not terminated".to_string()),
            104 => Reason::Panic("host call panicked".to_string()),

            // exit code of fault range: 0x10000 - 0x100000000 (4294967296)
            fault if fault > 0x10000 => Reason::Fault {
                page: (fault as u64 / pvm::PAGE_SIZE) as u32,
            },
            _ => Reason::Panic(format!("unknown exit code: {code}")),
        }
    }

    /// map self to i64
    pub fn value<'b>(self, builder: &mut FunctionBuilder<'b>) -> Value {
        match self {
            Exit::Halt => builder.ins().iconst(types::I64, 0),
            Exit::OOG => builder.ins().iconst(types::I64, 4),

            // exit code of panic range: 100 - 0x10000 (65536)
            Exit::Trap => builder.ins().iconst(types::I64, 100),
            Exit::InvalidStartPC => builder.ins().iconst(types::I64, 101),
            Exit::InvalidJumpTarget => builder.ins().iconst(types::I64, 102),
            Exit::ProgramNotTerminated => builder.ins().iconst(types::I64, 103),
            Exit::HostCallPanicked => builder.ins().iconst(types::I64, 104),

            // exit code of fault range: 0x10000 - 0x100000000 (4294967296)
            Exit::Fault { address } => address,
        }
    }
}
