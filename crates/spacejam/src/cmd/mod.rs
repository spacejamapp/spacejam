//! Command line interface for spacejam

use crate::{
    Development,
    node::{Builder, spec::RuntimeSpecSelf},
};
use clap::{ArgAction, CommandFactory, Parser};
use tracing_subscriber::EnvFilter;

pub mod fuzz;
pub mod key;

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

    /// Disable ANSI colors
    #[arg(short, long, global = true)]
    noansi: bool,
}

impl App {
    /// Run the command
    pub async fn run() {
        let app = App::parse();
        if app.graypaper {
            println!("graypaper: {}", crate::GRAYPAPER);
            return;
        }

        app.setup_logger();
        let Some(cmd) = app.cmd else {
            return;
        };

        if let Err(e) = cmd.run::<Development>().await {
            tracing::error!("{e}");
        }
    }

    /// Setup the logger
    fn setup_logger(&self) {
        let env = EnvFilter::try_from_default_env();
        if self.verbose == 0 && env.is_err() {
            return;
        }

        let name = App::command().get_name().to_string();
        let env = env.unwrap_or(EnvFilter::new(match self.verbose {
            0 => Default::default(),
            1 => format!("{name}=info"),
            2 => format!("{name}=debug"),
            3 => format!("debug,{name}=trace"),
            4 => "trace".to_string(),
            _ => "trace".into(),
        }));

        // Initialize tracing
        let mut subscriber = tracing_subscriber::fmt()
            .with_env_filter(env)
            .with_timer(fmt::Time)
            .with_target(false)
            .with_ansi(!self.noansi);

        if self.verbose > 2 {
            subscriber = subscriber.with_target(true)
        }

        subscriber.init();
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

    /// Spacejam fuzz command
    #[command(subcommand)]
    Fuzz(fuzz::Fuzz),
}

impl Command {
    /// Run the command
    pub async fn run<C: RuntimeSpecSelf>(self) -> anyhow::Result<()> {
        match self {
            Command::Run(run) => run.build::<C>().await?.start().await,
            Command::Key(key) => key.run(),
            Command::Fuzz(fuzz) => fuzz.run().await,
        }
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
