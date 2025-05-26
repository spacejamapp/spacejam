//! Validator from local.

use anyhow::Result;
use crypto::{bls, ed25519, vrf};
use rand::Rng;
use runtime::Validator;
use score::{
    BandersnatchPublic, BandersnatchRingVrfSignature, BandersnatchVrfSignature, BlsPublic,
    Ed25519Public, ValidatorMetadata,
};
use serde::{Deserialize, Serialize};

/// Validator from local.
pub struct LocalValidator {
    /// BLS key pair.
    pub bls: bls::KeyPair,

    /// Ed25519 key pair.
    pub ed25519: ed25519::KeyPair,

    /// Banersnatch key pair.
    pub bandersnatch: vrf::KeyPair,

    /// Banersnatch public key.
    bandersnatch_public: BandersnatchPublic,
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

    /// Create a random seed.
    pub fn random_seed() -> [u8; 32] {
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();
        seed
    }
}

impl Validator for LocalValidator {
    fn dev() -> Self {
        Self::from([0; 32])
    }

    fn random() -> Self {
        Self::random()
    }

    fn bls_public_key(&self) -> BlsPublic {
        self.bls.public()
    }

    fn ed25519_public_key(&self) -> Ed25519Public {
        *self.ed25519.verifying.as_bytes()
    }

    fn bandersnatch_public_key(&self) -> BandersnatchPublic {
        self.bandersnatch_public
    }

    fn bandersnatch_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> anyhow::Result<BandersnatchVrfSignature> {
        self.bandersnatch.ietf_sign(keys.to_vec(), message, context)
    }

    fn bandersnatch_ring_sign(
        &self,
        keys: &[[u8; 32]],
        context: &[u8],
        message: &[u8],
    ) -> anyhow::Result<BandersnatchRingVrfSignature> {
        self.bandersnatch.ring_sign(keys.to_vec(), message, context)
    }

    fn ietf_vrf_output(&self, message: &[u8]) -> Result<[u8; 32]> {
        self.bandersnatch.output_hash(message)
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
        let bandersnatch = vrf::KeyPair::from(seed);
        let bandersnatch_public = bandersnatch
            .public()
            .expect("invalid bandersnatch public key");

        Self {
            bls: bls::KeyPair::from(seed),
            ed25519: ed25519::KeyPair::from(seed),
            bandersnatch,
            bandersnatch_public,
        }
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
            let seed: [u8; 32] = [
                num, 0, 0, 0, num, 0, 0, 0, num, 0, 0, 0, num, 0, 0, 0, num, 0, 0, 0, num, 0, 0, 0,
                num, 0, 0, 0, num, 0, 0, 0,
            ];
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

#[derive(Debug, Serialize, Deserialize)]
struct LocalValidatorConfig {
    /// BLS key pair.
    pub bls: String,

    /// Ed25519 key pair.
    pub ed25519: String,

    /// Banersnatch key pair.
    pub bandersnatch: String,
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
        let bandersnatch_seed: [u8; 32] = hex::decode(config.bandersnatch)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid banersnatch seed length"))?;

        let bandersnatch = vrf::KeyPair::from(bandersnatch_seed);
        let bandersnatch_public = bandersnatch
            .public()
            .expect("invalid bandersnatch public key");

        Ok(Self {
            bls: bls::KeyPair::from(bls_seed),
            ed25519: ed25519::KeyPair::from(ed25519_seed),
            bandersnatch,
            bandersnatch_public,
        })
    }
}
