//! Tester for traces

use std::path::Path;
use testing::{Entry, Runner, Section, Trace};

/// Test traces
pub fn test(dir: &Path) -> anyhow::Result<()> {
    let entry = Entry::new(Section::Trace(Trace::Any), None, dir)?;
    let mut passed = 0;
    let mut failed = Vec::new();
    for test in entry {
        println!("Testing {}\n", test.name);
        if let Err(e) = Runner::step(&test) {
            failed.push((test.name, e));
        } else {
            passed += 1;
        }
    }
    println!("\n\nPassed {} tests", passed);

    if !failed.is_empty() {
        println!("\nFailed {} tests", failed.len());
        for (name, e) in failed {
            eprintln!("Test {} failed: {}", name, e);
        }
    }
    Ok(())
}
