//! The unix stream for fuzzing

use crate::fuzz::{
    StreamExt, init,
    message::{Initialize, KeyValue, Message, PeerInfo, Version},
};
use anyhow::{Context, Result};
use runtime::{
    storage::{Column, Commit, KVStorage, MemoryDb, StateStorage},
    tx,
};
use score::{Block, OpaqueHash, TimeSlot, safrole::ValidatorIter};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    ops::{Deref, DerefMut},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::Arc,
};

/// A fuzz target
pub struct Target {
    /// The connected unix stream
    stream: UnixStream,

    /// The database used in fuzzing
    data: Arc<MemoryDb>,

    /// The history of the state
    history: BTreeMap<TimeSlot, HashMap<Vec<u8>, Vec<u8>>>,

    /// If use interpreter instead
    interp: bool,
}

impl Target {
    /// Create a new target
    pub fn new(stream: UnixStream, interp: bool) -> Self {
        runtime::timing::setup();
        Self {
            stream,
            data: Arc::new(MemoryDb::default()),
            history: BTreeMap::new(),
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
            if let Ok(cache) = spacevm::SPACEJAM_CACHE_DIR.lock() {
                fs::remove_dir_all(&*cache).ok();
            }

            let stream = stream.context("Failed to accept connection")?;
            Self::run(stream, interp).await?;
        }

        Ok(())
    }

    /// Handle a new connection
    pub async fn run(stream: UnixStream, interp: bool) -> Result<()> {
        let mut target = Target::new(stream, interp);
        loop {
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
    pub async fn import_block(&mut self, mut block: Block) -> anyhow::Result<()> {
        let mut state = self.data.state()?;
        if block.header.slot <= state.timeslot
            && let Some(prev) = self.history.get(&(block.header.slot.saturating_sub(1)))
        {
            tracing::warn!("Fallback state to {}", block.header.slot.saturating_sub(1));
            self.data.reset(prev.clone());
            state = self.data.state()?;
        }

        let epoch = state.timeslot / score::EPOCH_LENGTH;
        let new_epoch = block.header.slot / score::EPOCH_LENGTH > epoch;
        let entropy = state.entropy;
        let safrole = state.safrole.clone();
        let header = block.header.clone();
        let data = self.data.clone();
        let slot = block.header.slot;
        let interp = self.interp;
        let validators = state.validators.current;
        let (vr, diff) = tokio::try_join!(
            tokio::spawn(async move {
                let verifier =
                    runtime::tx::ticket::lazy::verifier(epoch, &safrole.validators.bandersnatch())
                        .await;
                header
                    .validate(new_epoch, &validators, entropy, &safrole, verifier)
                    .await
            }),
            tokio::spawn(async move {
                if interp {
                    tx::simulate_with_state::<spacevm::Interpreter>(&mut block, state, data).await
                } else {
                    tx::simulate_with_state::<spacevm::SpaceVM>(&mut block, state, data).await
                }
            }),
        )?;
        let (_, diff) = (vr?, diff?);
        self.data.commit(Column::State, diff)?;
        {
            self.history.insert(slot, self.data.deep_clone());
            if self.history.len() > 6 {
                self.history.pop_first();
            }
        }
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        Ok(())
    }

    /// Received set state request
    #[tracing::instrument(skip_all, name = "initialize")]
    pub async fn initialize(&mut self, state: Initialize) -> Result<()> {
        self.history = Default::default();
        self.data = Arc::new(Default::default());
        let mut commit = Commit::default();
        for KeyValue { key, value } in state.state.into_iter() {
            commit.set(key, value);
        }

        self.data.commit(Column::State, commit)?;
        let state = self.data.state()?;
        let timeslot = state.timeslot;
        self.history.insert(timeslot, self.data.deep_clone());
        if let Err(e) = self.init_state().await {
            tracing::warn!("failed to initialize state: {e}");
        }
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        Ok(())
    }

    /// Received get state request
    pub fn get_state(&mut self, _hash: OpaqueHash) -> Result<()> {
        let mut state = Vec::new();
        for pair in self.data.iter(Column::State)? {
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
        let data = self.data.clone();
        let timeslot = data.timeslot()?;
        if self.interp {
            init::verifier(data).await?;
        } else {
            let (threadv, threadp) = (
                tokio::spawn(init::verifier(data.clone())),
                tokio::spawn(init::programs(data.clone())),
            );

            // blocking init only on large tests, e.g. starts from slot 0.
            if timeslot == 0 {
                let (vr, pr) = tokio::try_join!(threadv, threadp)?;
                let (_, _) = (vr?, pr?);
            }
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
