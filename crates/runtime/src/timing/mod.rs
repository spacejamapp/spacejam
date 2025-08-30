//! This module is modified from <https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/codegen/src/timing.rs>
//! for analyzing the execution time of spacejam runtime.
//!
//! Pass timing.
//!
//! This modules provides facilities for timing the execution of individual compilation passes.

pub use prelude::*;
use std::{boxed::Box, fmt};

mod disabled;
mod enabled;
mod prelude {
    #[cfg(not(feature = "timing"))]
    pub(crate) use super::disabled::*;
    #[cfg(feature = "timing")]
    pub use super::enabled::*;
}

// Each pass that can be timed is predefined with the `define_passes!` macro. Each pass has a
// snake_case name and a plain text description used when printing out the timing report.
//
// This macro defines:
//
// - A C-style enum containing all the pass names and a `None` variant.
// - A usize constant with the number of defined passes.
// - A const array of pass descriptions.
// - A public function per pass used to start the timing of that pass.
macro_rules! define_passes {
    ($($pass:ident: $desc:expr,)+) => {
        /// A single profiled pass.
        #[expect(non_camel_case_types, reason = "macro-generated code")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Pass {
            $(#[doc=$desc] $pass,)+
            /// No active pass.
            None,
        }

        /// The amount of profiled passes.
        pub const NUM_PASSES: usize = Pass::None as usize;

        const DESCRIPTIONS: [&str; NUM_PASSES] = [ $($desc),+ ];

        $(
            #[doc=$desc]
            #[must_use]
            pub fn $pass() -> Box<DefaultTimingToken> {
                start(Pass::$pass)
            }
        )+
    }
}

// Pass definitions.
define_passes! {
    entropy: "calculating entropy",
    disputes: "validating disputes",
    assignments: "updating availability assignments",
    assurances: "validating assurances",
    safrole: "rotate validators and update safrole",
    accumulate: "accumulating available work reports",
    guarantees: "validating guarantees",
    preimages: "validating preimages",
    commit: "committing the state",
}

impl Pass {
    fn idx(self) -> usize {
        self as usize
    }

    /// Description of the pass.
    pub fn description(self) -> &'static str {
        match DESCRIPTIONS.get(self.idx()) {
            Some(s) => s,
            None => "<no pass>",
        }
    }
}

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// A profiler.
pub trait Profiler {
    /// Start a profiling pass.
    fn start(&self, pass: Pass) -> Box<DefaultTimingToken>;
}

/// The default profiler. You can get the results using [`take_current`].
pub struct DefaultProfiler;
