//! Command line interface for spacejam

use crate::{
    node::{spec::RuntimeSpecSelf, Builder},
    Development,
};
use clap::{ArgAction, CommandFactory, Parser};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

pub mod key;
pub mod state;

/// The command line interface for SpaceJam
#[derive(Parser)]
#[command(arg_required_else_help = true)]
#[command(version)]
pub struct App {
    /// The command to run
    #[command(subcommand)]
    cmd: Option<Command>,

    /// The version of matched graypaper
    #[arg(short, long)]
    graypaper: bool,

    /// The verbosity level (repeat for more verbosity)
    #[arg(short, action = ArgAction::Count, global = true)]
    verbose: u8,

    /// The path to the root data directory
    #[arg(short, long, default_value_t = default::data_path())]
    data_path: String,
}

impl App {
    /// Run the command
    pub async fn run() {
        let app = App::parse();
        if app.graypaper {
            println!("{}", crate::GRAYPAPER);
            return;
        }

        // set up logs
        let name = App::command().get_name().to_string();
        let env = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(match app.verbose {
            0 => format!("{name}=info"),
            1 => format!("{name}=debug"),
            2 => format!("{name}=trace"),
            3 => "debug".into(),
            _ => "trace".into(),
        }));

        // Initialize tracing
        let mut subscriber = tracing_subscriber::fmt()
            .with_env_filter(env)
            .with_timer(fmt::Time)
            .with_target(false);

        if app.verbose > 0 {
            subscriber = subscriber.with_target(true);
        }

        subscriber.init();

        let Some(cmd) = app.cmd else {
            return;
        };

        if let Err(e) = cmd.run::<Development>(PathBuf::from(app.data_path)).await {
            eprintln!("Failed to run spacejam: {e}");
        }
    }
}

/// The command line interface for spacejam
#[derive(Parser)]
pub enum Command {
    /// Start the SpaceJam node
    Run(Box<Builder>),

    /// SpaceJam key utils
    #[command(subcommand)]
    Key(key::Key),
    // /// chain state related utils
    // #[command(subcommand)]
    // State(state::State),
    // /// Iterate over the storage
    // Iter,
}

impl Command {
    /// Run the command
    pub async fn run<C: RuntimeSpecSelf>(self, data: PathBuf) -> anyhow::Result<()> {
        match self {
            Command::Run(run) => run.build::<C>(data).await?.start().await,
            Command::Key(key) => key.run(),
            // Command::State(state) => state.run(),
        }
    }
}

mod default {
    /// The default data path
    pub fn data_path() -> String {
        dirs::data_dir()
            .unwrap_or_default()
            .join("spacejam")
            .to_string_lossy()
            .to_string()
    }
}

mod fmt {
    use time::OffsetDateTime;
    use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

    /// The time format
    pub struct Time;

    impl FormatTime for Time {
        fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
            let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
            write!(
                w,
                "{}",
                now.format(
                    &time::format_description::parse(
                        "[year]-[month]-[day] [hour]:[minute]:[second]"
                    )
                    .expect("could not parse time format")
                )
                .expect("could not format time")
            )
        }
    }
}
