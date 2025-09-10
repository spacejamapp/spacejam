//! Enabled timing utilities for SpaceJam
#![cfg(feature = "timing")]

use super::{DefaultProfiler, Pass, Profiler, DESCRIPTIONS, NUM_PASSES};
use std::{
    boxed::Box,
    cell::{Cell, RefCell},
    fmt, mem,
    time::{Duration, Instant},
};

// Information about passes in a single thread.
thread_local! {
    static PROFILER: RefCell<Box<dyn Profiler>> = RefCell::new(Box::new(DefaultProfiler));
}

/// Setup the timing for the current thread.
pub fn setup() {
    self::set_thread_profiler(Box::new(DefaultProfiler));
}

/// Set the profiler for the current thread.
///
/// Returns the old profiler.
pub fn set_thread_profiler(new_profiler: Box<dyn Profiler>) -> Box<dyn Profiler> {
    PROFILER.with(|profiler| std::mem::replace(&mut *profiler.borrow_mut(), new_profiler))
}

/// Start timing `pass` as a child of the currently running pass, if any.
///
/// This function is called by the publicly exposed pass functions.
pub fn start(pass: Pass) -> Box<DefaultTimingToken> {
    PROFILER.with(|profiler| profiler.borrow().start(pass))
}

/// Accumulated timing information for a single pass.
#[derive(Default, Copy, Clone)]
struct PassTime {
    /// Total time spent running this pass including children.
    total: Duration,

    /// Time spent running in child passes.
    child: Duration,
}

/// Accumulated timing for all passes.
pub struct PassTimes {
    pass: [PassTime; NUM_PASSES],
}

impl PassTimes {
    /// Add `other` to the timings of this `PassTimes`.
    pub fn add(&mut self, other: &Self) {
        for (a, b) in self.pass.iter_mut().zip(&other.pass[..]) {
            a.total += b.total;
            a.child += b.child;
        }
    }

    /// Returns the total amount of time taken by all the passes measured.
    pub fn total(&self) -> Duration {
        self.pass.iter().map(|p| p.total - p.child).sum()
    }
}

impl Default for PassTimes {
    fn default() -> Self {
        Self {
            pass: [Default::default(); NUM_PASSES],
        }
    }
}

impl fmt::Display for PassTimes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "======== ========  ==================================")?;
        writeln!(f, "   Total     Self  Pass")?;
        writeln!(f, "-------- --------  ----------------------------------")?;
        for (time, desc) in self.pass.iter().zip(&DESCRIPTIONS[..]) {
            // Omit passes that haven't run.
            if time.total == Duration::default() {
                continue;
            }

            // Write a duration as secs.millis, trailing space.
            fn fmtdur(mut dur: Duration, f: &mut fmt::Formatter) -> fmt::Result {
                // Round to nearest ms by adding 500us.
                dur += Duration::new(0, 500_000);
                let ms = dur.subsec_millis();
                write!(f, "{:4}.{:03} ", dur.as_secs(), ms)
            }

            fmtdur(time.total, f)?;
            if let Some(s) = time.total.checked_sub(time.child) {
                fmtdur(s, f)?;
            }
            writeln!(f, " {desc}")?;
        }
        writeln!(f, "======== ========  ==================================")
    }
}

// Information about passes in a single thread.
thread_local! {
    static PASS_TIME: RefCell<PassTimes> = RefCell::new(Default::default());
}

/// Take the current accumulated pass timings and reset the timings for the current thread.
///
/// Only applies when [`DefaultProfiler`] is used.
pub fn take_current() -> PassTimes {
    PASS_TIME.with(|rc| mem::take(&mut *rc.borrow_mut()))
}

// Information about passes in a single thread.
thread_local! {
    static CURRENT_PASS: Cell<Pass> = const { Cell::new(Pass::None) };
}

impl Profiler for DefaultProfiler {
    fn start(&self, pass: Pass) -> Box<DefaultTimingToken> {
        let prev = CURRENT_PASS.with(|p| p.replace(pass));
        // tracing::trace!("timing: Starting {pass}, (during {prev})");
        Box::new(DefaultTimingToken {
            start: Instant::now(),
            pass,
            prev,
        })
    }
}

/// A timing token is responsible for timing the currently running pass. Timing starts when it
/// is created and ends when it is dropped.
pub struct DefaultTimingToken {
    /// Start time for this pass.
    start: Instant,

    // Pass being timed by this token.
    pass: Pass,

    // The previously active pass which will be restored when this token is dropped.
    prev: Pass,
}

/// Dropping a timing token indicated the end of the pass.
impl Drop for DefaultTimingToken {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        // tracing::trace!("timing: Ending {}: {}ms", self.pass, duration.as_millis());
        let old_cur = CURRENT_PASS.with(|p| p.replace(self.prev));
        debug_assert_eq!(self.pass, old_cur, "Timing tokens dropped out of order");
        PASS_TIME.with(|rc| {
            let mut table = rc.borrow_mut();
            table.pass[self.pass.idx()].total += duration;
            if let Some(parent) = table.pass.get_mut(self.prev.idx()) {
                parent.child += duration;
            }
        })
    }
}
