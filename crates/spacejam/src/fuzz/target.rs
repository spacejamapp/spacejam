//! The unix stream for fuzzing

use crate::fuzz::{
    StreamExt, init,
    message::{Initialize, KeyValue, Message, PeerInfo, Version},
};
use anyhow::{Context, Result};
use runtime::{
    storage::{Column, KVStorage},
    tx::{block::TestChain, ticket::lazy},
};
use score::{Block, OpaqueHash};
use std::{
    fs,
    ops::{Deref, DerefMut},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};

/// If the target is running on Linux
const IS_LINUX: bool = cfg!(target_os = "linux");

/// A fuzz target
pub struct Target {
    /// The connected unix stream
    stream: UnixStream,

    /// The chain of blocks
    chain: TestChain,

    /// If use interpreter instead
    interp: bool,
}

impl Target {
    /// Create a new target
    pub fn new(stream: UnixStream, interp: bool) -> Self {
        runtime::timing::setup();
        Self {
            stream,
            chain: TestChain::default(),
            interp,
        }
    }

    /// Run the target
    pub async fn serve(socket: &Path, interp: bool) -> Result<()> {
        fs::remove_file(socket).ok();
        let listener = UnixListener::bind(socket)
            .context(format!("Failed to bind to the socket at {socket:?}"))?;
        tracing::info!("Listening on {socket:?}");

        for stream in listener.incoming() {
            let stream = stream.context("Failed to accept connection")?;
            Self::run(stream, interp).await?;
        }

        Ok(())
    }

    /// Handle a new connection
    pub async fn run(stream: UnixStream, interp: bool) -> Result<()> {
        let mut target = Target::new(stream, interp);
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let pid = sysinfo::Pid::from_u32(std::process::id());

        loop {
            let _ = sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
            if let Some(process) = sys.process(pid) {
                tracing::debug!(
                    target: "mdbg",
                    "Memory usage: {} MB",
                    process.memory() / 1024 / 1024
                );
            }

            let Ok(message) = target.read_message() else {
                tracing::info!("Disconnected from the fuzzer");
                return Ok(());
            };

            if let Err(e) = target.handle(message).await {
                tracing::warn!("failed to process message: {e}, waiting for the next message ...");
            }
        }
    }

    /// Handle a incoming message
    pub async fn handle(&mut self, message: Message) -> Result<()> {
        match message {
            Message::Info(info) => self.info(info),
            Message::ImportBlock(block) => {
                if let Err(e) = self.import_block(block).await {
                    tracing::warn!("failed to import block: {e}");
                    self.write_message(Message::Error(e.to_string()))?;
                }
                Ok(())
            }
            Message::Initialize(state) => self.initialize(state).await,
            Message::GetState(hash) => self.get_state(hash),
            Message::State(state) => self.state(state),
            Message::StateRoot(hash) => self.state_root(hash),
            Message::Error(error) => self.error(error),
        }
    }

    /// Received info request
    pub fn info(&mut self, info: PeerInfo) -> anyhow::Result<()> {
        let this = PeerInfo::default();
        if info.jam_version != Version::PROTOCOL {
            anyhow::bail!(
                "protocol version mismatched, remote: {:?}, local: {:?}",
                info.jam_version,
                this.jam_version
            );
        }

        self.write_message(Message::Info(this))?;
        Ok(())
    }

    /// Received import block request
    #[tracing::instrument(skip_all, name = "import", parent = None)]
    pub async fn import_block(&mut self, block: Block) -> anyhow::Result<()> {
        let root = if self.interp || !IS_LINUX {
            self.chain.import::<spacevm::Interpreter>(block)?
        } else {
            self.chain.import::<spacevm::SpaceVM>(block)?
        };

        let message = Message::StateRoot(root);
        self.write_message(message)?;
        Ok(())
    }

    /// Received set state request
    #[tracing::instrument(skip_all, name = "initialize")]
    pub async fn initialize(&mut self, state: Initialize) -> Result<()> {
        lazy::clear().await;
        self.chain = Default::default();
        let root = self.chain.init(state.keyvals())?;
        if let Err(e) = self.init_state().await {
            tracing::warn!("failed to initialize state: {e}");
        }
        let message = Message::StateRoot(root);
        self.write_message(message)?;
        Ok(())
    }

    /// Received get state request
    pub fn get_state(&mut self, hash: OpaqueHash) -> Result<()> {
        let mut state = Vec::new();
        let iter: Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>)>>> =
            if let Some(fork) = self.chain.forks.get(&hash) {
                Box::new(fork.iter(Column::State)?)
            } else {
                Box::new(self.chain.data.iter(Column::State)?)
            };

        for pair in iter {
            let (vkey, value) = pair?;
            let mut key = [0; 31];
            key.copy_from_slice(&vkey);
            state.push(KeyValue { key, value });
        }

        self.write_message(Message::State(state))
    }

    /// Handle the state request
    pub fn state(&mut self, _state: Vec<KeyValue>) -> Result<()> {
        anyhow::bail!("Received message state which is not supported");
    }

    /// Handle the state root request
    pub fn state_root(&mut self, _root: OpaqueHash) -> Result<()> {
        anyhow::bail!("Received message state root which is not supported");
    }

    /// Handle the state root request
    pub fn error(&mut self, _error: String) -> Result<()> {
        anyhow::bail!("Received message error which is not supported");
    }

    /// Initialize the target
    #[tracing::instrument(skip_all, name = "init", parent = None)]
    async fn init_state(&self) -> Result<()> {
        let data = self.chain.data.clone();
        if self.interp || !IS_LINUX {
            init::verifier(data)?;
        } else {
            let (vr, pr) = rayon::join(
                || init::verifier(data.clone()),
                || init::programs(data.clone()),
            );
            let _ = (vr?, pr?);
        }

        Ok(())
    }
}

impl Deref for Target {
    type Target = UnixStream;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for Target {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
