//! The unix stream for fuzzing

use crate::fuzz::{
    StreamExt, init,
    message::{KeyValue, Message, PeerInfo, SetState, Version},
};
use anyhow::{Context, Result};
use runtime::{
    storage::{Column, Commit, KVStorage, MemoryDb, StateStorage},
    tx,
};
use score::{Block, OpaqueHash, safrole::ValidatorIter};
use std::{
    fs,
    ops::{Deref, DerefMut},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::Arc,
    time::Instant,
};

/// A fuzz target
pub struct Target {
    /// The connected unix stream
    stream: UnixStream,

    /// The database used in fuzzing
    data: Arc<MemoryDb>,

    /// The import time for each block
    imports: Vec<u32>,

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
            imports: Vec::new(),
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
        loop {
            let Ok(message) = target.read_message().inspect_err(|e| {
                tracing::warn!("No more bytes from the stream: {e}!",);
            }) else {
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
                    let root = self.data.root()?;
                    self.write_message(Message::StateRoot(root))
                } else {
                    // tracing::debug!("\n{}", runtime::timing::take_current());
                    Ok(())
                }
            }
            Message::SetState(state) => self.set_state(state).await,
            Message::GetState(hash) => self.get_state(hash),
            Message::State(state) => self.state(state),
            Message::StateRoot(hash) => self.state_root(hash),
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
        let timer = Instant::now();
        let state = self.data.state()?;
        let epoch = state.timeslot / score::EPOCH_LENGTH;
        let new_epoch = block.header.slot / score::EPOCH_LENGTH > epoch;

        let entropy = state.entropy;
        let safrole = state.safrole.clone();
        let header = block.header.clone();
        let diff = tokio::try_join!(
            async {
                let verifier =
                    runtime::tx::ticket::lazy::verifier(epoch, &safrole.validators.bandersnatch())
                        .await;

                tokio::task::spawn_blocking(move || {
                    header.validate(new_epoch, entropy, &safrole, verifier)
                })
                .await?
            },
            async {
                if self.interp {
                    tx::simulate_with_state::<spacevm::Interpreter>(
                        &mut block,
                        state,
                        self.data.clone(),
                    )
                    .await
                } else {
                    tx::simulate_with_state::<spacevm::SpaceVM>(
                        &mut block,
                        state,
                        self.data.clone(),
                    )
                    .await
                }
            }
        );

        self.data.commit(Column::State, diff?.1)?;
        self.imports.push(timer.elapsed().as_millis() as u32);
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        Ok(())
    }

    /// Received set state request
    #[tracing::instrument(skip_all, name = "set_state")]
    pub async fn set_state(&mut self, state: SetState) -> Result<()> {
        let mut commit = Commit::default();
        for KeyValue { key, value } in state.state.into_iter() {
            commit.set(key, value);
        }

        self.data.commit(Column::State, commit)?;
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

    /// Initialize the target
    async fn init_state(&self) -> Result<()> {
        let data = self.data.clone();
        if self.interp {
            tokio::spawn(async move {
                let _ = init::verifier(data.clone()).await;
            });
        } else {
            let _ = tokio::try_join!(init::verifier(data.clone()), init::programs(data.clone()))?;
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
