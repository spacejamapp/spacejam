//! Block execution result

/// Block execution result
#[derive(Debug, Clone)]
pub enum ExecResult {
    Continue,
    Jump(u64),
    Halt,
    Trap,
}
