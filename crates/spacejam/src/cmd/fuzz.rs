//! Fuzz related commands

use crate::fuzz::{self, target::Target};
use clap::Parser;
use std::{io::Read, os::unix::net::UnixStream, path::PathBuf, rc::Rc, sync::Mutex};

/// The fuzz command
#[derive(Parser)]
pub enum Fuzz {
    /// Fuzz the local node
    Local {
        /// The path to the unix socket
        #[clap(default_value = "/tmp/jam_target.sock")]
        socket: PathBuf,

        /// The path to the data folder
        #[clap(default_value = "spacejam_data")]
        data: PathBuf,
    },

    /// Fuzz the trace file
    Trace {
        /// The path to the trace folder
        traces: PathBuf,
    },
}

impl Fuzz {
    /// Run the fuzz command
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Local { socket, data } => {
                let stream = Rc::new(Mutex::new(UnixStream::connect(socket)?));
                let mut target = Target::new(stream.clone(), data.join("fuzz"))?;

                loop {
                    let mut tx = stream.lock().unwrap();
                    let mut length = [0; 4];
                    tx.read_exact(&mut length)?;

                    let length = u32::from_le_bytes(length) as usize;
                    let mut message_bytes = vec![0; length];
                    tx.read_exact(&mut message_bytes)?;
                    let message = codec::decode(&message_bytes)?;
                    drop(tx);

                    target.handle(message)?;
                }
            }
            Self::Trace { traces } => {
                fuzz::trace::test(traces)?;
            }
        }
        Ok(())
    }
}
