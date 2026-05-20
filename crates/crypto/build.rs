use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress};
use ark_vrf::suites::bandersnatch::{PcsParams, RingProofParams};
use std::{io::Result, path::Path, process::Command};

const REPO: &str = "https://github.com/davxy/bandersnatch-vrfs-spec.git";
const INTO: &str = "bandersnatch-vrfs-spec";

fn ring_size() -> usize {
    if std::env::var("CARGO_FEATURE_FULL").is_ok() {
        1023
    } else {
        6
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let into = Path::new(INTO);
    if !into.exists() {
        Command::new("git").args(["clone", REPO, INTO]).status()?;
        Command::new("git")
            .args(["checkout", "cc99f5c"])
            .current_dir(into)
            .status()?;
    }

    let ring_size = ring_size();
    let output =
        format!("bandersnatch-vrfs-spec/assets/example/data/size-{ring_size}-with-zcash-srs.bin");

    if !Path::new(&output).exists() {
        let buf = std::fs::read(
            "bandersnatch-vrfs-spec/assets/example/data/zcash-srs-2-11-uncompressed.bin",
        )
        .expect("Failed to read srs file");
        let pcs_params = PcsParams::deserialize_uncompressed_unchecked(&mut &buf[..])
            .expect("Failed to deserialize SRS parameters");
        let res = RingProofParams::from_pcs_params(ring_size, pcs_params)
            .expect("Failed to create ring context");
        let mut bytes = vec![];
        let _ = res.serialize_with_mode(&mut bytes, Compress::No);
        std::fs::write(&output, bytes).expect("Failed to create params serialize file");
    }

    Ok(())
}
