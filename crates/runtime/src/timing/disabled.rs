//! Disabled timing utilities for SpaceJam
#![cfg(not(feature = "timing"))]

use super::{DefaultProfiler, Pass, Profiler};
use std::{any::Any, boxed::Box};

impl Profiler for DefaultProfiler {
    fn start(&self, _pass: Pass) -> DefaultTimingToken {
        Box::new(())
    }
}

pub(crate) fn start(_pass: Pass) -> DefaultTimingToken {
    Box::new(())
}

/// dummy timing token
pub struct DefaultTimingToken;
