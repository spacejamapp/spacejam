//! Env-var driven fuzz target dispatch (jam-conformance fuzz-proto).
//!
//! Conformance contract: when `JAM_FUZZ` is set, the binary must run as a fuzz
//! target with no CLI arguments, configured entirely from environment
//! variables. See `res/jam-conformance/fuzz-proto/README.md` ("Standard Target
//! Packaging").

use super::target::Target;
use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing_subscriber::EnvFilter;

const JAM_FUZZ: &str = "JAM_FUZZ";
const JAM_FUZZ_SPEC: &str = "JAM_FUZZ_SPEC";
const JAM_FUZZ_DATA_PATH: &str = "JAM_FUZZ_DATA_PATH";
const JAM_FUZZ_SOCK_PATH: &str = "JAM_FUZZ_SOCK_PATH";
const JAM_FUZZ_LOG_LEVEL: &str = "JAM_FUZZ_LOG_LEVEL";

/// Whether the env-driven fuzz mode is requested.
pub fn is_active() -> bool {
    std::env::var_os(JAM_FUZZ).is_some()
}

/// Run the fuzz target as configured by env vars.
pub async fn run() -> Result<()> {
    init_logger(std::env::var(JAM_FUZZ_LOG_LEVEL).ok().as_deref());
    let cfg = Config::from_env()?;
    tracing::info!(
        "fuzz target starting: spec={}, data_path={}, socket={}",
        cfg.spec.as_str(),
        cfg.data_path.display(),
        cfg.socket.display(),
    );
    set_compiler_cache_dir(&cfg.data_path)?;
    // Use the compiler on linux; Target::serve falls back to interp on other platforms.
    Target::serve(&cfg.socket, /*interp=*/ false).await
}

/// Point the cranelift artifact cache at `data_path` so JIT compilation
/// results persist across fuzz sessions (per fuzz-proto README, the path is
/// host-mapped and may be reused for caching).
fn set_compiler_cache_dir(data_path: &Path) -> Result<()> {
    fs::create_dir_all(data_path)
        .with_context(|| format!("failed to create JAM_FUZZ_DATA_PATH at {data_path:?}"))?;
    let mut slot = spacevm::SPACEJAM_CACHE_DIR
        .lock()
        .map_err(|e| anyhow::anyhow!("compiler cache lock poisoned: {e:?}"))?;
    *slot = data_path.to_path_buf();
    Ok(())
}

struct Config {
    spec: Spec,
    data_path: PathBuf,
    socket: PathBuf,
}

impl Config {
    fn from_env() -> Result<Self> {
        let spec = Spec::from_str(&require_env(JAM_FUZZ_SPEC)?)?;
        let data_path = PathBuf::from(require_env(JAM_FUZZ_DATA_PATH)?);
        let socket = PathBuf::from(require_env(JAM_FUZZ_SOCK_PATH)?);
        Ok(Self {
            spec,
            data_path,
            socket,
        })
    }
}

enum Spec {
    #[cfg(all(feature = "tiny", not(feature = "full")))]
    Tiny,
    #[cfg(feature = "full")]
    Full,
}

impl Spec {
    fn as_str(&self) -> &'static str {
        match self {
            #[cfg(all(feature = "tiny", not(feature = "full")))]
            Self::Tiny => "tiny",
            #[cfg(feature = "full")]
            Self::Full => "full",
        }
    }
}

impl FromStr for Spec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            #[cfg(all(feature = "tiny", not(feature = "full")))]
            "tiny" => Ok(Self::Tiny),
            #[cfg(feature = "full")]
            "full" => Ok(Self::Full),
            #[cfg(all(feature = "tiny", not(feature = "full")))]
            "full" => bail!("JAM_FUZZ_SPEC=full requires building with --features full"),
            #[cfg(feature = "full")]
            "tiny" => bail!("JAM_FUZZ_SPEC=tiny requires building --no-default-features"),
            other => bail!("invalid JAM_FUZZ_SPEC: {other:?} (expected 'tiny' or 'full')"),
        }
    }
}

fn require_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("{name} must be set when {JAM_FUZZ} is set"))
}

// Mirror the main CLI's behavior: install a subscriber only when the operator
// explicitly opts in. Without JAM_FUZZ_LOG_LEVEL no subscriber is installed,
// so tracing macros become no-ops and the hot path stays free of logging cost.
fn init_logger(level: Option<&str>) {
    let Some(level) = level else { return };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(level))
        .with_target(false)
        .with_ansi(false)
        .try_init();
}
