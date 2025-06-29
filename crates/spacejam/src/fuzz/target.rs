//! The unix stream for fuzzing

use pvmi::Interpreter;
use runtime::{
    storage::{ArchiveStorage, Commit, KVStorage},
    tx, Storage,
};
use score::{Block, OpaqueHash};

use crate::{
    fuzz::{
        self,
        message::{KeyValue, Message, PeerInfo, SetState},
    },
    storage::Parity,
};
use std::{io::Write, os::unix::net::UnixStream, path::PathBuf, sync::Arc};

/// A fuzz target
pub struct Target<'s> {
    /// The connected unix stream
    stream: &'s mut UnixStream,

    /// The database used in fuzzing
    data: Arc<Parity>,
}

impl<'s> Target<'s> {
    /// Create a new target
    pub fn new(stream: &'s mut UnixStream, data: PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            stream,
            data: Arc::new(Parity::try_from(data)?),
        })
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

        let resp = Message::Info(this);
        self.stream.write_all(&codec::encode(&resp)?)?;
        self.stream.flush()?;
        Ok(())
    }

    /// Received import block request
    pub fn import_block(&mut self, block: Block) -> anyhow::Result<()> {
        let hash = block.header.hash()?;
        tx::transit::<Interpreter>(block, self.data.clone())?;
        let resp = Message::StateRoot(self.data.root()?);
        self.stream.write_all(&codec::encode(&resp)?)?;
        self.stream.flush()?;
        self.data.archive(hash)?;
        Ok(())
    }

    /// Received set state request
    pub fn set_state(&mut self, state: SetState) -> anyhow::Result<()> {
        let mut commit = Commit::default();
        let hash = state.header.hash()?;
        for KeyValue { key, value } in state.state.into_iter() {
            let key = hex::decode(key.trim_start_matches("0x"))?;
            let value = hex::decode(value.trim_start_matches("0x"))?;
            commit.set(key, value);
        }

        let resp = Message::StateRoot(self.data.root()?);
        self.stream.write_all(&codec::encode(&resp)?)?;
        self.stream.flush()?;
        self.data.archive(hash)?;
        Ok(())
    }

    /// Received get state request
    pub fn get_state(&mut self, hash: OpaqueHash) -> anyhow::Result<()> {
        let mut state = Vec::new();
        let mut iter = self.data.prefix_iter(hash)?;
        while let Some(pair) = iter.next() {
            let (key, value) = pair?;
            state.push(KeyValue {
                key: hex::encode(key),
                value: hex::encode(value),
            });
        }

        let resp = Message::State(state);
        self.stream.write_all(&codec::encode(&resp)?)?;
        self.stream.flush()?;
        Ok(())
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
