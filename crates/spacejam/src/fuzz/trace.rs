//! Tester for traces

use anyhow::Context;
use serde_json::Value;
use std::{fs, path::Path, time::Instant};
use testing::{Entry, Payload, Runner, Scale, Section, Test, Trace};

/// Test a single trace file (`.json` or `.bin`).
pub async fn test(file: &Path) -> anyhow::Result<()> {
    Runner::step(&parse(file)?).await
}

/// Test every trace file in a directory (sorted by filename, `.bin` preferred over `.json`).
/// Skips files whose name stem doesn't parse as a step number (e.g. `report.bin`).
pub async fn test_dir(dir: &Path) -> anyhow::Result<()> {
    let path = dir.to_str().context("non-utf8 trace dir")?;
    let entry = Entry::seq(path)?;
    let now = Instant::now();
    let mut processed = 0usize;
    for test in entry {
        let stem = test.name.rsplit('_').next().unwrap_or(&test.name);
        if stem.parse::<u64>().is_err() {
            continue;
        }
        Runner::step(&test).await?;
        processed += 1;
    }
    tracing::info!("processed {processed} traces in {:?}", now.elapsed());
    Ok(())
}

/// Build a `Test` from a single trace file.
fn parse(file: &Path) -> anyhow::Result<Test> {
    let name = file
        .file_stem()
        .context("invalid file name")?
        .to_string_lossy()
        .to_string();
    let section = Section::Trace(Trace::Any);
    let scale = Some(Scale::Tiny);

    if file.extension().and_then(|s| s.to_str()) == Some("bin") {
        return Ok(Test {
            input: Payload::Bin(fs::read(file)?),
            output: Payload::default(),
            scale,
            section,
            name,
        });
    }

    let json: Value = serde_json::from_slice(&fs::read(file)?)?;
    let input = serde_json::json!({
        "block": json["block"],
        "pre_state": json["pre_state"],
    })
    .to_string();
    let output = serde_json::json!({
        "post_state": json["post_state"],
    })
    .to_string();

    Ok(Test {
        input: Payload::Json(input),
        output: Payload::Json(output),
        scale,
        section,
        name,
    })
}
