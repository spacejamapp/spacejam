//! Spacejam testing library

#![allow(unused_imports)]

pub use specjam::{Entry, Scale, Section, Test, Trace};

pub mod accumulate;
pub mod assurances;
pub mod authorizations;
pub mod codec;
pub mod disputes;
pub mod erasure;
pub mod history;
pub mod preimage;
pub mod pvmc;
pub mod pvmi;
pub mod reports;
pub mod safrole;
// pub mod seq;
pub mod shuffle;
pub mod statistics;
pub mod traces;
pub mod trie;

/// The `Runner` struct which is used to run the tests.
pub struct Runner;

impl Runner {
    /// Step a test.
    pub async fn step(test: &Test) -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .without_time()
            .with_ansi(false)
            .with_thread_names(false)
            .with_file(false)
            // .with_level(false)
            .with_target(false)
            .try_init();

        match test.section {
            Section::Accumulate => crate::accumulate::run(test).await?,
            Section::Assurances => crate::assurances::run(test)?,
            Section::Authorizations => crate::authorizations::run(test)?,
            Section::Disputes => crate::disputes::run(test)?,
            Section::Erasure => crate::erasure::run(test).await?,
            Section::History => crate::history::run(test)?,
            Section::Preimages => crate::preimage::run(test)?,
            Section::Reports => crate::reports::run(test)?,
            Section::Safrole => crate::safrole::run(test)?,
            Section::Statistics => crate::statistics::run(test)?,
            Section::Pvm => crate::pvmi::run(test)?,
            Section::Trace(_) => {
                crate::traces::run(test).await?;
            }
            Section::Codec | Section::Shuffle | Section::Trie => {}
        }

        Ok(())
    }
}
