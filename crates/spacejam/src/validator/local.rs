//! Validator from local.

use crypto::{bls, ed25519, vrf};
use rand::Rng;

/// Validator from local.
pub struct LocalValidator {
    /// BLS key pair.
    pub bls: bls::KeyPair,

    /// Ed25519 key pair.
    pub ed25519: ed25519::KeyPair,

    /// Banersnatch key pair.
    pub banersnatch: vrf::KeyPair,
}

impl LocalValidator {
    /// Create a new local validator.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();
        seed.into()
    }
}

impl From<[u8; 32]> for LocalValidator {
    fn from(seed: [u8; 32]) -> Self {
        let bls_sk = bls::SecretKey::from_seed(&seed);
        let bls_pk = bls_sk.into_public();
        let bls = bls::KeyPair {
            secret: bls_sk,
            public: bls_pk,
        };

        Self {
            bls,
            ed25519: ed25519::KeyPair::from(seed),
            banersnatch: vrf::KeyPair::from(seed),
        }
    }
}
