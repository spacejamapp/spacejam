//! Key command

use crate::validator::LocalValidator;
use clap::Parser;
use network::peer::PeerId;
use runtime::Validator;

/// Key utils
#[derive(Parser)]
pub enum Key {
    /// Generate a new key
    Generate,

    /// Show the info of the key
    Info {
        /// The seed of the keyring, could be number and hex string
        seed: String,
    },
}

impl Key {
    /// Run the command
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Generate => Self::generate(),
            Self::Info { seed } => Self::info(seed)?,
        }

        Ok(())
    }

    /// Generate a new key
    fn generate() {
        let seed = LocalValidator::random_seed();
        let validator = LocalValidator::from(seed);
        println!("{:16}0x{}", "seed:", hex::encode(seed));
        Self::print(validator);
    }

    /// Show the info of the key
    fn info(seed: String) -> anyhow::Result<()> {
        let validator = LocalValidator::try_from(seed)?;
        Self::print(validator);
        Ok(())
    }

    /// prints the info of the validator
    fn print(validator: LocalValidator) {
        let ed25519 = validator.ed25519_public_key();
        // println!("BLS:        0x{}", hex::encode(validator.bls_public_key()));
        println!("{:16}0x{}", "ed25519:", hex::encode(ed25519));
        println!(
            "{:16}0x{}",
            "bandersnatch:",
            hex::encode(validator.bandersnatch_public_key())
        );
        println!(
            "{:16}{}",
            "peer id:",
            PeerId::from(validator.ed25519_public_key())
        );
    }
}
