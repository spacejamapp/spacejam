use anyhow::Result;
use git2::{build::RepoBuilder, FetchOptions};
use std::path::Path;

const REPO: &str = "https://github.com/w3f/jamtestvectors.git";
const INTO: &str = "jamtestvectors";

fn main() -> Result<()> {
    let into = Path::new(INTO);
    if into.exists() {
        return Ok(());
    }

    let mut builder = RepoBuilder::new();
    let mut opts = FetchOptions::new();

    opts.depth(1);
    builder.fetch_options(opts);
    builder.clone(REPO, into)?;
    Ok(())
}
