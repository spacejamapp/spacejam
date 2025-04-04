//! This vrf implementation is based on the example in the Bandersnatch VRFs specification.
//!
//! The specification can be found at <https://github.com/davxy/bandersnatch-vrfs-spec>
//!
//! commit hash: 8c82722
#![cfg(feature = "vrf")]

use crate::ring::RING_CTX;
use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_vrf::suites::bandersnatch;
pub use bandersnatch::{IetfProof, Input, Output, Public, RingProof, Secret};

/// Get the VRF output hash.
pub fn ietf_output(sig: [u8; 96]) -> Result<[u8; 32]> {
    let signature = IetfVrfSignature::deserialize_compressed(sig.as_ref())
        .map_err(|e| anyhow::anyhow!("Failed to deserialize bandersnatch signature: {e}"))?;
    let output = signature.output;
    let output_hash = output.hash();
    let mut buf = [0; 32];
    buf.copy_from_slice(&output_hash[..32]);
    Ok(buf)
}

/// Banersnatch key pair.
pub struct KeyPair {
    /// Secret key.
    pub secret: Secret,

    /// Public key.
    pub public: Public,
}

impl KeyPair {
    /// Get the public key as a 32-byte array.
    pub fn public(&self) -> Result<[u8; 32]> {
        let mut buf = [0; 32];
        self.public.0.serialize_compressed(&mut buf[..])?;
        Ok(buf)
    }

    /// Get the VRF output.
    pub fn output(&self, message: &[u8]) -> Result<[u8; 96]> {
        let output = self.secret.output(
            Input::new(message).ok_or(anyhow::anyhow!("Invalid bandersnatch input {message:?}"))?,
        );
        let mut buf = [0; 96];
        output.serialize_compressed(&mut buf[..])?;
        Ok(buf)
    }

    /// Get the VRF output.
    pub fn output_hash(&self, message: &[u8]) -> Result<[u8; 32]> {
        let output = self.secret.output(
            Input::new(message).ok_or(anyhow::anyhow!("Invalid bandersnatch input {message:?}"))?,
        );
        let src = output.hash();
        let mut hash = [0; 32];
        hash.copy_from_slice(&src[..32]);
        Ok(hash)
    }

    /// Sign a message using the ring VRF.
    pub fn sign(
        &self,
        pks: Vec<[u8; 32]>,
        context: &[u8],
        message: &[u8],
        ring: bool,
        buffer: &mut [u8],
    ) -> Result<()> {
        let public = self.public()?;
        let this = pks
            .iter()
            .position(|pk| pk == &public)
            .ok_or(anyhow::anyhow!(
                "Public key not found in the list of validators"
            ))?;

        let prover = Prover::new(
            pks.into_iter()
                .map(|pk| {
                    Public::deserialize_compressed(&mut pk.as_slice())
                        .map_err(|e| anyhow::anyhow!(e))
                })
                .collect::<Result<Vec<_>>>()?,
            this,
            self.secret.clone(),
        );

        let signature = if ring {
            prover.ring_vrf_sign(message, context)?
        } else {
            prover.ietf_vrf_sign(message, context)?
        };

        buffer.copy_from_slice(&signature);
        Ok(())
    }

    /// Sign a message using bandersnatch.
    pub fn ietf_sign(
        &self,
        pks: Vec<[u8; 32]>,
        message: &[u8],
        context: &[u8],
    ) -> Result<[u8; 96]> {
        let mut buffer = [0; 96];
        self.sign(pks, context, message, false, &mut buffer)?;
        Ok(buffer)
    }

    /// Sign a message using ring.
    pub fn ring_sign(
        &self,
        pks: Vec<[u8; 32]>,
        message: &[u8],
        context: &[u8],
    ) -> Result<[u8; 784]> {
        let mut buffer = [0; 784];
        self.sign(pks, context, message, true, &mut buffer)?;
        Ok(buffer)
    }
}

impl From<[u8; 32]> for KeyPair {
    fn from(seed: [u8; 32]) -> Self {
        let secret = Secret::from_seed(&seed);
        let public = secret.public();
        Self { secret, public }
    }
}

// This is the IETF `Prove` procedure output as described in section 2.2
// of the Bandersnatch VRFs specification
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct IetfVrfSignature {
    pub output: Output,
    proof: IetfProof,
}

/// Ring commitment type
pub type RingCommitment = ark_vrf::ring::RingCommitment<bandersnatch::BandersnatchSha512Ell2>;

// This is the IETF `Prove` procedure output as described in section 4.2
// of the Bandersnatch VRFs specification
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct RingVrfSignature {
    pub output: Output,
    // This contains both the Pedersen proof and actual ring proof.
    pub proof: RingProof,
}

// Prover actor.
pub struct Prover {
    pub prover_idx: usize,
    pub secret: Secret,
    pub ring: Vec<Public>,
}

impl Prover {
    /// Creates a new prover.
    pub fn new(ring: Vec<Public>, prover_idx: usize, secret: Secret) -> Self {
        Self {
            prover_idx,
            secret,
            ring,
        }
    }

    /// Anonymous VRF signature.
    ///
    /// Used for tickets submission.
    pub fn ring_vrf_sign(&self, vrf_input_data: &[u8], aux_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        use ark_vrf::ring::Prover as _;

        let input = Input::new(vrf_input_data).ok_or(anyhow::anyhow!("Invalid input"))?;
        let output = self.secret.output(input);

        // Backend currently requires the wrapped type (plain affine points)
        let pts: Vec<_> = self.ring.iter().map(|pk| pk.0).collect();

        // Proof construction
        let prover_key = RING_CTX.prover_key(&pts);
        let prover = RING_CTX.prover(prover_key, self.prover_idx);
        let proof = self.secret.prove(input, output, aux_data, &prover);

        // Output and Ring Proof bundled together (as per section 2.2)
        let signature = RingVrfSignature { output, proof };
        let mut buf = Vec::new();
        signature.serialize_compressed(&mut buf)?;
        Ok(buf)
    }

    /// Non-Anonymous VRF signature.
    ///
    /// Used for ticket claiming during block production.
    /// Not used with Safrole test vectors.
    pub fn ietf_vrf_sign(&self, vrf_input_data: &[u8], aux_data: &[u8]) -> anyhow::Result<Vec<u8>> {
        use ark_vrf::ietf::Prover as _;

        let input = Input::new(vrf_input_data).ok_or(anyhow::anyhow!("Invalid input"))?;
        let output = self.secret.output(input);
        let proof = self.secret.prove(input, output, aux_data);

        // Output and IETF Proof bundled together (as per section 2.2)
        let signature = IetfVrfSignature { output, proof };
        let mut buf = Vec::new();
        signature.serialize_compressed(&mut buf)?;
        Ok(buf)
    }
}

// Verifier actor.
//
// TODO: use life time to avoid cloning the ring.
pub struct Verifier {
    pub commitment: RingCommitment,
    pub ring: Vec<Public>,
}

impl Verifier {
    pub fn new(ring: Vec<Public>) -> Self {
        // Backend currently requires the wrapped type (plain affine points)
        let pts: Vec<_> = ring.iter().map(|pk| pk.0).collect();
        let verifier_key = RING_CTX.verifier_key(&pts);
        let commitment = verifier_key.commitment();
        Self { ring, commitment }
    }

    /// Anonymous VRF signature verification.
    ///
    /// Used for tickets verification.
    ///
    /// On success returns the VRF output hash.
    pub fn ring_vrf_verify(
        &self,
        vrf_input_data: &[u8],
        aux_data: &[u8],
        signature: &[u8],
    ) -> anyhow::Result<[u8; 32]> {
        use ark_vrf::ring::Verifier as _;

        let signature = RingVrfSignature::deserialize_compressed(signature)?;
        let input = Input::new(vrf_input_data).ok_or(anyhow::anyhow!("Invalid input"))?;
        let output = signature.output;

        // The verifier key is reconstructed from the commitment and the constant
        // verifier key component of the SRS in order to verify some proof.
        // As an alternative we can construct the verifier key using the
        // RingContext::verifier_key() method, but is more expensive.
        // In other words, we prefer computing the commitment once, when the keyset changes.
        let verifier_key = RING_CTX.verifier_key_from_commitment(self.commitment.clone());
        let verifier = RING_CTX.verifier(verifier_key);
        Public::verify(input, output, aux_data, &signature.proof, &verifier)
            .map_err(|e| anyhow::anyhow!("Ring signature verification failure: {:?}", e))?;

        // This truncated hash is the actual value used as ticket-id/score in JAM
        let vrf_output_hash: [u8; 32] = output.hash()[..32].try_into()?;
        Ok(vrf_output_hash)
    }

    /// Non-Anonymous VRF signature verification.
    ///
    /// Used for ticket claim verification during block import.
    /// Not used with Safrole test vectors.
    ///
    /// On success returns the VRF output hash.
    pub fn ietf_vrf_verify(
        &self,
        vrf_input_data: &[u8],
        aux_data: &[u8],
        signature: &[u8],
        signer_key_index: usize,
    ) -> anyhow::Result<[u8; 32]> {
        use ark_vrf::ietf::Verifier as _;

        let signature = IetfVrfSignature::deserialize_compressed(signature)?;
        let input = Input::new(vrf_input_data).ok_or(anyhow::anyhow!("Invalid input"))?;
        let output = signature.output;

        let public = &self.ring[signer_key_index];
        public
            .verify(input, output, aux_data, &signature.proof)
            .map_err(|_| anyhow::anyhow!("Ietf signature verification failure"))?;

        // This is the actual value used as ticket-id/score
        // NOTE: as far as vrf_input_data is the same, this matches the one produced
        // using the ring-vrf (regardless of aux_data).
        let vrf_output_hash: [u8; 32] = output.hash()[..32].try_into()?;
        Ok(vrf_output_hash)
    }
}
