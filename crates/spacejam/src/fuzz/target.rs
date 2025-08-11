//! The unix stream for fuzzing

use crate::fuzz::{
    self,
    message::{KeyValue, Message, PeerInfo, SetState},
    StreamExt,
};
use anyhow::Context;
use pvmi::Interpreter;
use runtime::{
    storage::{ArchiveStorage, Column, Commit, KVStorage, MemoryDb, StateStorage},
    tx,
};
use score::{Block, OpaqueHash};
use std::{
    ops::{Deref, DerefMut},
    os::unix::net::UnixStream,
    path::Path,
    sync::Arc,
};

/// A fuzz target
pub struct Target {
    /// The connected unix stream
    stream: UnixStream,

    /// The database used in fuzzing
    data: Arc<MemoryDb>,
}

impl Target {
    /// Create a new target
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            data: Arc::new(MemoryDb::default()),
        }
    }

    /// Run the target
    pub fn run(socket: &Path) -> anyhow::Result<()> {
        let stream = UnixStream::connect(socket)
            .context(format!("Failed to connect to the socket at {socket:?}"))?;
        let mut target = Target::new(stream);

        loop {
            let message = target.read_message()?;
            target.handle(message)?;
        }
    }

    /// Handle a incoming message
    pub fn handle(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Info(info) => self.info(info),
            Message::ImportBlock(block) => self.import_block(block),
            Message::SetState(state) => self.set_state(state),
            Message::GetState(hash) => self.get_state(hash),
            Message::State(state) => self.state(state),
            Message::StateRoot(hash) => self.state_root(hash),
        }
    }

    /// Received info request
    #[tracing::instrument(skip_all)]
    pub fn info(&mut self, info: PeerInfo) -> anyhow::Result<()> {
        let this = PeerInfo {
            name: "spacejam".into(),
            version: fuzz::VERSION,
            protocol: fuzz::PROTOCOL_VERSION,
        };

        if info.protocol != fuzz::PROTOCOL_VERSION {
            tracing::warn!(
                "protocol version mismatched, remote: {:?}, local: {:?}",
                info.protocol,
                this.protocol
            );
        }

        self.write_message(Message::Info(this))?;
        Ok(())
    }

    /// Received import block request
    pub fn import_block(&mut self, block: Block) -> anyhow::Result<()> {
        let hash = block.header.hash()?;
        tx::transit::<Interpreter>(block, self.data.clone())?;
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        self.data.archive(&hash)?;
        Ok(())
    }

    /// Received set state request
    pub fn set_state(&mut self, state: SetState) -> anyhow::Result<()> {
        let mut commit = Commit::default();
        let hash = state.header.hash()?;
        for KeyValue { key, value } in state.state.into_iter() {
            let buf = hex::decode(key.trim_start_matches("0x"))?;
            if buf.len() != 31 {
                anyhow::bail!("Invalid state key length: {}", buf.len());
            }
            let mut key = [0; 31];
            key.copy_from_slice(&buf);
            key[31] = 0;

            let value = hex::decode(value.trim_start_matches("0x"))?;
            commit.set(key, value);
        }

        self.data.commit(Column::State, commit)?;
        let message = Message::StateRoot(self.data.root()?);
        self.write_message(message)?;
        self.data.archive(&hash)?;
        Ok(())
    }

    /// Received get state request
    pub fn get_state(&mut self, hash: OpaqueHash) -> anyhow::Result<()> {
        let mut state = Vec::new();
        let iter = self.data.state_prefix_iter(&hash)?;
        for pair in iter {
            let (key, value) = pair?;
            state.push(KeyValue {
                key: hex::encode(key),
                value: hex::encode(value),
            });
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
