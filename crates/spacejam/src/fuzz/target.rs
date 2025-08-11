//! The unix stream for fuzzing

use crate::fuzz::{
    self,
    message::{KeyValue, Message, PeerInfo, SetState},
    StreamExt,
};
use anyhow::Context;
use pvmi::Interpreter;
use runtime::{
    storage::{Column, Commit, KVStorage, MemoryDb, StateStorage},
    tx,
};
use score::{Block, OpaqueHash};
use std::{
    ops::{Deref, DerefMut},
    os::unix::net::UnixStream,
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

    imports: Vec<u32>,
}

impl Target {
    /// Create a new target
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            data: Arc::new(MemoryDb::default()),
            imports: Vec::new(),
        }
    }

    /// Run the target
    pub fn run(socket: &Path) -> anyhow::Result<()> {
        let stream = UnixStream::connect(socket)
            .context(format!("Failed to connect to the socket at {socket:?}"))?;
        let mut target = Target::new(stream);

        loop {
            let Ok(message) = target.read_message().inspect_err(|e| {
                let blocks = target.imports.len();
                tracing::info!(
                    "No more bytes from the stream({e})! average transit time for {blocks} blocks: {}ms",
                    target.imports.iter().sum::<u32>() / blocks as u32
                );
            }) else {
                return Ok(());
            };
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
    pub fn import_block(&mut self, block: Block) -> anyhow::Result<()> {
        let timer = Instant::now();
        tx::transit::<Interpreter>(block, self.data.clone())?;
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
            let buf = hex::decode(key.trim_start_matches("0x"))?;
            if buf.len() != 31 {
                anyhow::bail!("Invalid state key length: {}", buf.len());
            }
            let mut key = [0; 31];
            key.copy_from_slice(&buf);

            let value = hex::decode(value.trim_start_matches("0x"))?;
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
