//! Fuzzer related implementations

use crate::fuzz::{
    StreamExt,
    message::{Initialize, KeyValue, Message, PeerInfo, Version},
};
use anyhow::{Context, Result};
use score::OpaqueHash;
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};
use testing::{Entry, Section, Test, Trace, traces};

/// The fuzzer
pub struct Fuzzer {
    /// The stream of the fuzzer
    info: PeerInfo,

    /// The report directory
    report: Option<PathBuf>,

    /// The stream of the target
    stream: UnixStream,

    /// If initialized the state
    init: bool,

    failures: Vec<(PathBuf, String)>,
}

impl Fuzzer {
    /// Run the fuzzer
    pub fn run(socket: &Path, entry: &Path, report: &Path) -> Result<()> {
        let entry = Entry::new(Section::Trace(Trace::Any), None, entry).context(format!(
            "Failed to parse traces folder, {entry:?}, should be the folder of traces, \n
            for example jam-test-vectors/traces/storage"
        ))?;

        // handle incoming connections
        let mut stream =
            UnixStream::connect(socket).context(format!("Failed to connect to {socket:?}"))?;
        let mut fuzzer = Self {
            info: Self::peer_info(&mut stream)?,
            report: Some(report.to_path_buf()),
            stream,
            init: false,
            failures: Vec::new(),
        };

        let now = std::time::Instant::now();
        fuzzer.handle(entry)?;
        if !fuzzer.failures.is_empty() {
            for (base, error) in fuzzer.failures {
                tracing::error!("Failed to process {base:?}: {error}");
            }
        }

        tracing::info!("Finished! Time taken: {:?}", now.elapsed());
        Ok(())
    }

    /// Handle a new connection
    pub fn handle(&mut self, source: Entry) -> Result<()> {
        let base = source.base.clone();
        for test in source {
            if test.name.contains("genesis") {
                continue;
            }
            tracing::info!("\tProcessing test: {}", test.name);
            if let Err(e) = self.import_block(test) {
                self.failures.push((base.clone(), e.to_string()));
                break;
            }
        }

        tracing::info!("\tdone!");
        Ok(())
    }

    /// Handle a new connection
    pub fn handle_single(&mut self, source: &Entry) -> Result<()> {
        let test = source.get(0).context("No test found")?;
        let input = traces::TestInput::from_json(&test.input)?;
        self.init_state(&input, &test.name)?;
        self.import_block(test)
    }

    /// Import a block
    pub fn import_block(&mut self, test: Test) -> Result<()> {
        let input = traces::TestInput::from_json(&test.input)?;
        let output = traces::TestOutput::from_json(&test.output)?;
        if !self.init {
            self.init_state(&input, &test.name)?;
            self.init = true;
        }

        // import block and verify
        let header = input.block.header.clone();
        self.stream
            .write_message(Message::ImportBlock(input.block))?;
        self.verify_root(
            output.post_state.state_root,
            &test.name,
            header.hash(),
            Self::to_keyvals(output.post_state.keyvals.clone()),
            output.post_state.state_root == input.pre_state.state_root,
        )?;
        Ok(())
    }

    /// Verify the state root
    pub fn verify_root(
        &mut self,
        root: OpaqueHash,
        name: &str,
        block: OpaqueHash,
        state: Vec<KeyValue>,
        exp_err: bool,
    ) -> Result<()> {
        let received = self.stream.read_message()?;
        let mut error = None;
        let remote = if let Message::StateRoot(remote) = received {
            Some(remote)
        } else if let Message::Error(err) = received {
            if exp_err {
                return Ok(());
            }
            error = Some(err.clone());
            None
        } else {
            anyhow::bail!("Expected StateRoot or Error message, got {:?}", received);
        };

        if remote == Some(root) {
            return Ok(());
        }

        // get the state from the remote peer and generate a report
        self.stream.write_message(Message::GetState(block))?;
        let received = self.stream.read_message()?;
        let Message::State(received) = received else {
            anyhow::bail!("Expected State message, got {:?}", received);
        };

        let Some(report) = &self.report else {
            return Ok(());
        };

        fs::create_dir_all(report)?;
        let output = report.join(format!("{}-{name}.json", self.info.app_name));
        fs::write(
            &output,
            serde_json::to_string_pretty(&json!({
                "expected": state,
                "received": received,
            }))?,
        )?;

        if let Some(error) = error {
            anyhow::bail!("Got error message: {error}");
        }

        anyhow::bail!(
            "Expected state root: 0x{}, got 0x{}, write the report to {output:?}",
            hex::encode(root),
            hex::encode(remote.unwrap_or_default())
        );
    }

    /// Send the peer info
    pub fn peer_info(stream: &mut UnixStream) -> Result<PeerInfo> {
        let info = PeerInfo::default();
        stream.write_message(Message::Info(info))?;

        // receive the remote peer info
        let received = stream.read_message()?;
        let Message::Info(received) = received else {
            anyhow::bail!("Expected Info message, got {:?}", received);
        };

        // check the remote peer info
        tracing::info!("Received peer info: {received:?}");
        if received.jam_version != Version::PROTOCOL {
            anyhow::bail!(
                "Expected protocol: {:?}, got {:?}",
                Version::PROTOCOL,
                received.jam_version
            );
        }

        Ok(received)
    }

    /// initialize state
    pub fn init_state(&mut self, input: &traces::TestInput, name: &str) -> Result<()> {
        let state = Self::to_keyvals(input.pre_state.keyvals.clone());
        let set_state = Initialize {
            header: input.block.header.clone(),
            state: state.clone(),
            ancestry: vec![],
        };

        // verify the state root
        self.stream.write_message(Message::Initialize(set_state))?;
        self.verify_root(
            input.pre_state.state_root,
            name,
            input.block.header.hash(),
            state,
            false,
        )
    }

    /// Get the keyvals of the state
    pub fn to_keyvals(keyvals: Vec<traces::KeyValue>) -> Vec<KeyValue> {
        keyvals
            .iter()
            .map(|kv| {
                let mut key = [0; 31];
                key.copy_from_slice(&kv.key);
                KeyValue {
                    key,
                    value: kv.value.clone(),
                }
            })
            .collect::<Vec<_>>()
    }
}

impl Fuzzer {
    /// Run the fuzzer with traces
    pub fn conformance(socket: &Path, entry: &Path, report: &Path) -> Result<()> {
        if !entry.is_dir() {
            anyhow::bail!("invalid traces folder, {entry:?}");
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(entry)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                entries.push(path);
            }
        }

        // handle incoming connections
        let mut stream =
            UnixStream::connect(socket).context(format!("Failed to connect to {socket:?}"))?;
        let mut fuzzer = Self {
            info: Self::peer_info(&mut stream)?,
            report: Some(report.to_path_buf()),
            stream,
            init: false,
            failures: Vec::new(),
        };

        let total = entries.len();
        for entry in entries {
            let entry = Entry::new(Section::Trace(Trace::Any), None, &entry).context(format!(
                "Failed to parse traces folder, {entry:?}, should be a folder of traces, \n
                for example jam-test-vectors/traces/storage"
            ))?;
            fuzzer.init = false;
            tracing::info!("processing {:?} ...", entry.base);
            fuzzer.handle(entry)?;
        }

        let failed = fuzzer.failures.len();
        if !fuzzer.failures.is_empty() {
            for (base, error) in fuzzer.failures {
                tracing::error!("Failed to process {base:?}: {error}");
            }
        }

        if failed > 0 {
            anyhow::bail!("Failed to process {failed}/{total} tests");
        } else {
            tracing::info!("{total}/{total} passed!");
        }

        Ok(())
    }

    /// Execute a single test
    pub fn execute(socket: &Path, test: &Path, report: &Path) -> Result<()> {
        let mut stream =
            UnixStream::connect(socket).context(format!("Failed to connect to {socket:?}"))?;

        let entry = Entry {
            base: Default::default(),
            section: Section::Trace(Trace::Any),
            scale: None,
            files: BTreeSet::from([test.to_path_buf()]),
            current: 0,
        };

        let mut fuzzer = Self {
            info: Self::peer_info(&mut stream)?,
            report: Some(report.to_path_buf()),
            stream,
            init: false,
            failures: Vec::new(),
        };

        fuzzer.handle_single(&entry)
    }
}
