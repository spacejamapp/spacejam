//! BLS12-381 keypair

use anyhow::Result;
pub use {public::PublicKey, secret::SecretKey, signature::Signature};

mod public;
mod secret;
mod signature;

/// A BLS12-381 key pair.
pub struct KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl KeyPair {
    /// Generate a new key pair.
    pub fn random() -> Self {
        let secret_key = SecretKey::random();
        let public_key = secret_key.public();

        Self {
            secret_key,
            public_key,
        }
    }

    /// Sign a message.
    pub fn sign(&self, message: &[u8]) -> Result<[u8; 96]> {
        self.secret_key.sign(message).try_into()
    }

    /// Verify a signature.
    pub fn verify(&self, message: &[u8], signature: &[u8; 96]) -> bool {
        self.public_key.verify(message, signature)
    }
}

#[test]
fn test_bls() {
    use ark_std::rand;

    // Generate a random keypair
    let KeyPair {
        secret_key,
        public_key,
    } = KeyPair::random();

    // Test signing and verification with a random message
    let message = b"Hello, World!";
    let signature = secret_key
        .sign(message)
        .try_into()
        .expect("failed to sign message");
    assert!(public_key.verify(message, &signature));

    // Test that verification fails with wrong message
    let wrong_message = b"Wrong message";
    assert!(!public_key.verify(wrong_message, &signature));

    // Test that verification fails with wrong signature
    let mut wrong_signature = signature;
    wrong_signature[0] = wrong_signature[0].wrapping_add(1);
    assert!(!public_key.verify(message, &wrong_signature));

    // Test with random messages
    for _ in 0..10 {
        let random_msg: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let sig = secret_key
            .sign(&random_msg)
            .try_into()
            .expect("failed to sign message");
        assert!(public_key.verify(&random_msg, &sig));
    }
}
