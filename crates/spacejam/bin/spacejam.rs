use clap::{ArgAction, CommandFactory, Parser};
use spacejam::{cmd::Command, validator::LocalValidator};
use tracing_subscriber::EnvFilter;

struct Config;

impl spacejam::Config for Config {
    type Validator = LocalValidator;
    type Db = spacejam::storage::sled::Sled;
}

/// The command line interface for SpaceJam
#[derive(Parser)]
#[command(arg_required_else_help = true)]
#[command(version)]
struct App {
    /// The command to run
    #[command(subcommand)]
    cmd: Option<Command>,

    /// The verbosity level (repeat for more verbosity)
    #[arg(short, action = ArgAction::Count, global = true)]
    verbose: u8,
}

#[tokio::main]
async fn main() {
    let app = App::parse();
    let name = App::command().get_name().to_string();
    let env = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(match app.verbose {
        0 => format!("{name}=info"),
        1 => format!("{name}=debug"),
        2 => "debug".into(),
        _ => "trace".into(),
    }));

    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter(env).init();

    let Some(cmd) = app.cmd else {
        return;
    };

    if let Err(e) = cmd.run::<Config>().await {
        eprintln!("Failed to run spacejam: {e}");

        if cfg!(debug_assertions) {
            eprintln!("backtrace: {:#?}", e.backtrace());
        }
    }
}
