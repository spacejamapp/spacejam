//! Fuzzer related implementations

use crate::fuzz::{
    message::{KeyValue, Message, PeerInfo, SetState},
    StreamExt, PROTOCOL_VERSION, VERSION,
};
use anyhow::{Context, Result};
use score::{block::Header, OpaqueHash};
use serde_json::json;
use spacejson::Json;
use std::{
    fs,
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
};
use testing::{traces, Entry, Section, Test, Trace};

/// The fuzzer
pub struct Fuzzer {
    /// The stream of the fuzzer
    info: PeerInfo,

    /// The report directory
    report: PathBuf,

    /// The stream of the target
    stream: UnixStream,

    /// The current block
    block: Header,

    /// The keyvals of the state
    state: Vec<KeyValue>,
}

impl Fuzzer {
    /// Run the fuzzer
    pub fn run(socket: &Path, entry: &Path, report: &Path) -> Result<()> {
        tracing::info!("Listening on {socket:?}");
        if socket.exists() {
            fs::remove_file(socket)?;
        }

        let listener =
            UnixListener::bind(socket).context(format!("Failed to bind to {socket:?}"))?;
        let entry = Entry::new(Section::Trace(Trace::Any), None, entry).context(format!(
            "Failed to parse traces folder, {entry:?}, should be the folder of traces, \n
            for example jam-test-vectors/traces/storage"
        ))?;

        // handle incoming connections
        for stream in listener.incoming() {
            let mut stream = stream.context("Failed to accept connection")?;
            let mut fuzzer = Self {
                info: Self::peer_info(&mut stream)?,
                report: report.to_path_buf(),
                stream,
                block: Header::default(),
                state: vec![],
            };

            fuzzer.handle(&entry)?;
        }

        Ok(())
    }

    /// Handle a new connection
    pub fn handle(&mut self, source: &Entry) -> Result<()> {
        // run the tests
        let mut block = 1;
        loop {
            let Ok(test) = source.test(&format!("{:08}", block)) else {
                tracing::info!("No more tests!");
                return Ok(());
            };

            tracing::info!("Processing test: {}", test.name);
            self.import_block(test)?;
            block += 1;
        }
    }

    /// Import a block
    pub fn import_block(&mut self, test: Test) -> Result<()> {
        let input = traces::TestInput::from_json(&test.input)?;
        let output = traces::TestOutput::from_json(&test.output)?;
        if input.block.header.slot == 1 {
            self.init_state(&input, &test.name)?;
        }

        // import block and verify
        let header = input.block.header.clone();
        self.stream
            .write_message(Message::ImportBlock(input.block))?;
        self.verify_root(output.post_state.state_root, &test.name)?;
        self.block = header;
        Ok(())
    }

    /// Verify the state root
    pub fn verify_root(&mut self, root: OpaqueHash, name: &str) -> Result<()> {
        let received = self.stream.read_message()?;
        let Message::StateRoot(remote) = received else {
            anyhow::bail!("Expected StateRoot message, got {:?}", received);
        };

        if remote == root {
            return Ok(());
        }

        // get the state from the remote peer and generate a report
        self.stream
            .write_message(Message::GetState(self.block.hash()?))?;
        let received = self.stream.read_message()?;
        let Message::State(received) = received else {
            anyhow::bail!("Expected State message, got {:?}", received);
        };

        fs::create_dir_all(&self.report)?;
        let output = self.report.join(format!("{}-{name}.json", self.info.name));
        fs::write(
            &output,
            serde_json::to_string_pretty(&json!({
                "header": self.block.clone().to_json(),
                "expected": self.state.clone(),
                "received": received,
            }))?,
        )?;

        anyhow::bail!(
            "Expected state root: 0x{}, got 0x{}, write the report to {output:?}",
            hex::encode(root),
            hex::encode(remote)
        );
    }

    /// Send the peer info
    pub fn peer_info(stream: &mut UnixStream) -> Result<PeerInfo> {
        let info = PeerInfo {
            name: "spacejam".into(),
            version: VERSION,
            protocol: PROTOCOL_VERSION,
        };

        stream.write_message(Message::Info(info))?;

        // receive the remote peer info
        let received = stream.read_message()?;
        let Message::Info(received) = received else {
            anyhow::bail!("Expected Info message, got {:?}", received);
        };

        // check the remote peer info
        tracing::info!("Received peer info: {received:?}");
        if received.protocol != PROTOCOL_VERSION {
            anyhow::bail!(
                "Expected protocol: {PROTOCOL_VERSION:?}, got {:?}",
                received.protocol
            );
        }

        Ok(received)
    }

    /// initialize state
    pub fn init_state(&mut self, input: &traces::TestInput, name: &str) -> Result<()> {
        self.set_keyvals(input.pre_state.keyvals.clone())?;
        let set_state = SetState {
            header: input.block.header.clone(),
            state: self.state.clone(),
        };

        // verify the state root
        self.stream.write_message(Message::SetState(set_state))?;
        self.verify_root(input.pre_state.state_root, name)
    }

    /// Get the keyvals of the state
    pub fn set_keyvals(&mut self, keyvals: Vec<traces::KeyValue>) -> Result<()> {
        let keyvals = keyvals
            .iter()
            .map(|kv| KeyValue {
                key: format!("0x{}", hex::encode(&kv.key)),
                value: format!("0x{}", hex::encode(&kv.value)),
            })
            .collect::<Vec<_>>();

        self.state = keyvals;
        Ok(())
    }
}
