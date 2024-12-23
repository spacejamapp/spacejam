//! BLS12-381 signature.

use anyhow::Result;
use ark_bls12_381::{g2::Config, G2Affine};
use ark_ec::{short_weierstrass::Projective, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

/// A BLS12-381 signature.
#[derive(Clone, Copy)]
pub struct Signature(pub(crate) Projective<Config>);

impl TryFrom<[u8; 96]> for Signature {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 96]) -> Result<Self> {
        let mut output = [0u8; 96];
        output.copy_from_slice(&value);
        let signature = G2Affine::deserialize_compressed(&output[..])?;
        Ok(Self(signature.into()))
    }
}

impl TryFrom<[u8; 192]> for Signature {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 192]) -> Result<Self> {
        let mut output = [0u8; 192];
        output.copy_from_slice(&value);
        let signature = G2Affine::deserialize_uncompressed(&output[..])?;
        Ok(Self(signature.into()))
    }
}

impl TryFrom<&Signature> for [u8; 96] {
    type Error = anyhow::Error;

    fn try_from(value: &Signature) -> Result<[u8; 96]> {
        let mut output = [0u8; 96];
        value
            .0
            .into_affine()
            .serialize_compressed(&mut output[..])?;
        Ok(output)
    }
}

impl TryFrom<Signature> for [u8; 96] {
    type Error = anyhow::Error;

    fn try_from(value: Signature) -> Result<[u8; 96]> {
        (&value).try_into()
    }
}

impl TryFrom<&Signature> for [u8; 192] {
    type Error = anyhow::Error;

    fn try_from(value: &Signature) -> Result<[u8; 192]> {
        let mut output = [0u8; 192];
        value
            .0
            .into_affine()
            .serialize_uncompressed(&mut output[..])?;
        Ok(output)
    }
}

impl TryFrom<Signature> for [u8; 192] {
    type Error = anyhow::Error;

    fn try_from(value: Signature) -> Result<[u8; 192]> {
        (&value).try_into()
    }
}
