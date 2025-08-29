//! Tester for traces

use serde_json::Value;
use std::{fs, path::Path};
use testing::{Runner, Scale, Section, Test, Trace};

/// Test traces
pub async fn test(test: &Path) -> anyhow::Result<()> {
    let json: Value = serde_json::from_slice(&fs::read(test)?)?;
    let input = serde_json::json!({
        "block": json["block"],
        "pre_state": json["pre_state"],
    })
    .to_string();

    let output = serde_json::json!({
        "post_state": json["post_state"],
    })
    .to_string();

    let test = Test {
        input,
        output,
        scale: Some(Scale::Tiny),
        section: Section::Trace(Trace::Any),
        name: test.file_name().unwrap().to_string_lossy().to_string(),
    };

    Runner::step(&test).await
}
