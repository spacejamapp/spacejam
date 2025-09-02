//! The unix stream for fuzzing

use crate::fuzz::{
    self,
    message::{KeyValue, Message, PeerInfo, SetState},
    StreamExt,
};
use anyhow::Context;
use runtime::{
    storage::{Column, Commit, KVStorage, MemoryDb, StateStorage},
    tx,
};
use score::{safrole::ValidatorIter, Block, OpaqueHash};
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
    compiler: bool,
}

impl Target {
    /// Create a new target
    pub fn new(stream: UnixStream, compiler: bool) -> Self {
        runtime::timing::setup();
        Self {
            stream,
            data: Arc::new(MemoryDb::default()),
            imports: Vec::new(),
            compiler,
        }
    }

    /// Run the target
    pub async fn serve(socket: &Path, interp: bool) -> anyhow::Result<()> {
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
    pub async fn run(stream: UnixStream, interp: bool) -> anyhow::Result<()> {
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
    pub async fn handle(&mut self, message: Message) -> anyhow::Result<()> {
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
            Message::SetState(state) => self.set_state(state),
            Message::GetState(hash) => self.get_state(hash),
            Message::State(state) => self.state(state),
            Message::StateRoot(hash) => self.state_root(hash),
        }
    }

    /// Received info request
    pub fn info(&mut self, info: PeerInfo) -> anyhow::Result<()> {
        let this = PeerInfo {
            name: "spacejam".into(),
            version: fuzz::VERSION,
            protocol: fuzz::PROTOCOL_VERSION,
        };

        if info.protocol != fuzz::PROTOCOL_VERSION {
            anyhow::bail!(
                "protocol version mismatched, remote: {:?}, local: {:?}",
                info.protocol,
                this.protocol
            );
        }

        self.write_message(Message::Info(this))?;
        Ok(())
    }

    /// Received import block request
    #[tracing::instrument(skip_all, name = "import", parent = None)]
    pub async fn import_block(&mut self, block: Block) -> anyhow::Result<()> {
        let timer = Instant::now();
        let state = self.data.state()?;
        let new_epoch =
            block.header.slot / score::EPOCH_LENGTH > state.timeslot / score::EPOCH_LENGTH;
        let verifier = runtime::tx::ticket::lazy::verifier(
            state.timeslot / score::EPOCH_LENGTH,
            &state.safrole.validators.bandersnatch(),
        )
        .await;
        block
            .header
            .validate(new_epoch, state.entropy, &state.safrole, verifier)?;

        if self.compiler {
            tx::transit_with_state::<jastime::Jastime>(block, state, self.data.clone()).await?;
        } else {
            tx::transit_with_state::<jastime::Interpreter>(block, state, self.data.clone()).await?;
        }

        self.imports.push(timer.elapsed().as_millis() as u32);
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        Ok(())
    }

    /// Received set state request
    #[tracing::instrument(skip_all, name = "set_state")]
    pub fn set_state(&mut self, state: SetState) -> anyhow::Result<()> {
        let mut commit = Commit::default();
        for KeyValue { key, value } in state.state.into_iter() {
            commit.set(key, value);
        }

        self.data.commit(Column::State, commit)?;
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        Ok(())
    }

    /// Received get state request
    pub fn get_state(&mut self, _hash: OpaqueHash) -> anyhow::Result<()> {
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
    pub fn state(&mut self, _state: Vec<KeyValue>) -> anyhow::Result<()> {
        anyhow::bail!("Received message state which is not supported");
    }

    /// Handle the state root request
    pub fn state_root(&mut self, _root: OpaqueHash) -> anyhow::Result<()> {
        anyhow::bail!("Received message state root which is not supported");
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
