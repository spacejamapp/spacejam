//! PVM instruction visitor.

use crate::format::*;
use crate::Instruction;

include!(concat!(env!("OUT_DIR"), "/visitor.rs"));
