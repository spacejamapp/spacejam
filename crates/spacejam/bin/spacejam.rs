use clap::{ArgAction, CommandFactory, Parser};
use spacejam::cmd::Command;
use tracing_subscriber::EnvFilter;

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

fn main() {
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
}
