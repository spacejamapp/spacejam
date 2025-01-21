//! Block sync validation

use score::{validator::ValidatorData, OpaqueHash};

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod statistic;
pub mod ticket;

/// Updates the entropy accumulator.
///
/// graypaper reference: 6.4
pub fn eta(new_epoch: bool, eta: &[OpaqueHash; 4], entropy: OpaqueHash) -> [OpaqueHash; 4] {
    let mut next = *eta;

    // graypaper reference: 6.23
    //
    // eta'_e = H(eta_e || eta'_(e-1))
    if new_epoch {
        let historical_eta = eta;
        next[1..].copy_from_slice(&historical_eta[..3]);
    }

    // graypaper reference: 6.22
    //
    // eta'_0 = H(eta_0 || Y(H_v))
    let eta_0 = crypto::blake2b(&[eta[0], entropy].concat());
    next[0] = eta_0;
    next
}

/// Updates the previous epoch's validators.
pub fn lambda(
    new_epoch: bool,
    lambda: &[ValidatorData],
    kappa: &[ValidatorData],
) -> Vec<ValidatorData> {
    if new_epoch {
        kappa.to_vec()
    } else {
        lambda.to_vec()
    }
}
