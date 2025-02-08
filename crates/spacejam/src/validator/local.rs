//! Validator from local.

use anyhow::Result;
use crypto::{bls, ed25519, vrf};
use rand::Rng;
use score::{
    runtime::Validator, BandersnatchPublic, BandersnatchRingVrfSignature,
    BandersnatchVrfSignature, BlsPublic, Ed25519Public, ValidatorMetadata,
};

/// Validator from local.
pub struct LocalValidator {
    /// BLS key pair.
    pub bls: bls::KeyPair,

    /// Ed25519 key pair.
    pub ed25519: ed25519::KeyPair,

    /// Banersnatch key pair.
    pub banersnatch: vrf::KeyPair,
}

impl Default for LocalValidator {
    fn default() -> Self {
        Self::from([0u8; 32])
    }
}

impl LocalValidator {
    /// Create a new local validator.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();
        seed.into()
    }
}

impl Validator for LocalValidator {
    fn bls_public_key(&self) -> BlsPublic {
        self.bls.public()
    }

    fn ed25519_public_key(&self) -> Ed25519Public {
        *self.ed25519.verifying.as_bytes()
    }

    fn bandersnatch_public_key(&self) -> BandersnatchPublic {
        self.banersnatch
            .public()
            .expect("invalid bandersnatch public key")
    }

    fn bandersnatch_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> anyhow::Result<BandersnatchVrfSignature> {
        self.banersnatch.ietf_sign(keys.to_vec(), message, context)
    }

    fn bandersnatch_ring_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> anyhow::Result<BandersnatchRingVrfSignature> {
        self.banersnatch.ring_sign(keys.to_vec(), message, context)
    }

    fn bandersnatch_output(&self, message: &[u8]) -> Result<BandersnatchVrfSignature> {
        self.banersnatch.output(message)
    }

    fn metadata(&self) -> ValidatorMetadata {
        [0u8; 128]
    }

    fn ed25519(&self) -> Option<ed25519::KeyPair> {
        Some(self.ed25519.clone())
    }
}

impl From<[u8; 32]> for LocalValidator {
    fn from(seed: [u8; 32]) -> Self {
        Self {
            bls: bls::KeyPair::from(seed),
            ed25519: ed25519::KeyPair::from(seed),
            banersnatch: vrf::KeyPair::from(seed),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_config {
    use anyhow::Result;
    use crypto::{bls, ed25519, vrf};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct LocalValidatorConfig {
        /// BLS key pair.
        pub bls: String,

        /// Ed25519 key pair.
        pub ed25519: String,

        /// Banersnatch key pair.
        pub banersnatch: String,
    }

    impl TryFrom<String> for LocalValidatorConfig {
        type Error = anyhow::Error;

        fn try_from(seed: String) -> Result<Self> {
            if let Ok(config) = toml::from_str(&seed) {
                return Ok(config);
            }

            if let Ok(config) = serde_json::from_str(&seed) {
                return Ok(config);
            }

            anyhow::bail!("unsupported format, must be toml or json string")
        }
    }

    impl TryFrom<LocalValidatorConfig> for crate::validator::LocalValidator {
        type Error = anyhow::Error;

        fn try_from(config: LocalValidatorConfig) -> Result<Self> {
            let bls_seed: [u8; 32] = hex::decode(config.bls)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid bls seed length"))?;
            let ed25519_seed: [u8; 32] = hex::decode(config.ed25519)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid ed25519 seed length"))?;
            let banersnatch_seed: [u8; 32] = hex::decode(config.banersnatch)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid banersnatch seed length"))?;

            Ok(Self {
                bls: bls::KeyPair::from(bls_seed),
                ed25519: ed25519::KeyPair::from(ed25519_seed),
                banersnatch: vrf::KeyPair::from(banersnatch_seed),
            })
        }
    }

    // Strings should be supported:
    //
    // - 0x prefixed hex string
    // - toml string
    // - json string
    impl TryFrom<String> for crate::validator::LocalValidator {
        type Error = anyhow::Error;

        fn try_from(seed: String) -> Result<Self> {
            if let Ok(num) = seed.parse::<u8>() {
                let seed: [u8; 32] = [num; 32];
                return Ok(Self::from(seed));
            }

            if let Ok(seed) = hex::decode(seed.trim_start_matches("0x")) {
                let seed: [u8; 32] = seed
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid seed length, must be 32 bytes"))?;
                return Ok(Self::from(seed));
            }

            LocalValidatorConfig::try_from(seed).map(|config| config.try_into())?
        }
    }
}

#[cfg(not(feature = "serde"))]
impl TryFrom<String> for LocalValidator {
    type Error = anyhow::Error;

    fn try_from(seed: String) -> Result<Self> {
        let seed = hex::decode(seed.trim_start_matches("0x"))?;
        let seed: [u8; 32] = seed
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid seed length"))?;
        Ok(Self::from(seed))
    }
}
