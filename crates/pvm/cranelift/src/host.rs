//! Host function signatures

use cranelift::{
    codegen::isa::CallConv,
    prelude::{AbiParam, Signature, types},
};
use std::sync::LazyLock;

/// The signature of the general host call
pub static ECALLI: LazyLock<Signature> = LazyLock::new(|| Signature {
    params: vec![AbiParam::new(types::I32), AbiParam::new(types::I64)],
    returns: vec![AbiParam::new(types::I8)],
    call_conv: CallConv::Fast,
});

/// The signature of the sbrk host function
pub static SBRK: LazyLock<Signature> = LazyLock::new(|| Signature {
    params: vec![
        AbiParam::new(types::I64),
        AbiParam::new(types::I8),
        AbiParam::new(types::I8),
    ],
    returns: vec![],
    call_conv: CallConv::Fast,
});

/// The signature of the mget host function
pub static MGET: LazyLock<Signature> = LazyLock::new(|| Signature {
    params: vec![
        AbiParam::new(types::I64),
        AbiParam::new(types::I64),
        AbiParam::new(types::I8),
    ],
    returns: vec![AbiParam::new(types::I64)],
    call_conv: CallConv::Fast,
});

/// The signature of the mset host function
pub static MSET: LazyLock<Signature> = LazyLock::new(|| Signature {
    params: vec![
        AbiParam::new(types::I64),
        AbiParam::new(types::I64),
        AbiParam::new(types::I64),
        AbiParam::new(types::I8),
    ],
    returns: vec![],
    call_conv: CallConv::Fast,
});
