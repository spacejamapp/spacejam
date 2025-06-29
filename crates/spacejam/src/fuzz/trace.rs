//! Tester for traces

use std::path::Path;
use testing::{Entry, Runner, Section, Trace};

/// Test traces
pub fn test(dir: &Path) -> anyhow::Result<()> {
    let entry = Entry::new(Section::Trace(Trace::Any), None, dir)?;
    let mut passed = 0;
    let mut failed = Vec::new();
    for test in entry {
        tracing::info!("Testing {}", test.name);
        if let Err(e) = Runner::step(&test) {
            failed.push((test.name, e));
        } else {
            passed += 1;
        }
    }

    tracing::info!("\n\nPassed {passed} tests");
    if !failed.is_empty() {
        tracing::error!("\nFailed {} tests", failed.len());
        for (name, e) in failed {
            tracing::error!("Test {name} failed: {e}");
        }
    }
    Ok(())
}
