//! Block sync validation

use anyhow::Result;
use score::{
    validator::{Validator, ValidatorData},
    Block, OpaqueHash, State,
};

pub mod assurance;
pub mod dispute;
pub mod guarantee;
pub mod preimage;
pub mod statistic;
pub mod ticket;

/// Transit state with new block
pub fn transit(block: &Block, mut state: State, validator: impl Validator) -> Result<State> {
    let epoch = block.header.slot / score::EPOCH_LENGTH;
    let new_epoch: bool = epoch > (state.timeslot / score::EPOCH_LENGTH);

    // The first round computation
    {
        // (η') Update entropy
        let entropy = validator.entropy(state.entropy[0], &block.header.entropy_source)?;
        state.entropy = crate::eta(new_epoch, &state.entropy, entropy);

        // (λ') Update validator state
        state.validators.previous = crate::lambda(
            new_epoch,
            &state.validators.previous,
            &state.validators.current,
        );

        // (ψ') Update disputes and get marks
        let (disputes, marks) = crate::dispute::disputes(
            state.timeslot,
            &state.validators.current,
            &state.validators.previous,
            &state.disputes,
            &block.extrinsic.disputes,
        )?;
        state.disputes = disputes;

        // (ρ†) Update availability assignments based on verdicts (V)
        state.reports = crate::dispute::reports(&marks, &state.reports);

        // (ρ‡) Update availability assignments based on assurances
        let (reports, _availiable) = crate::assurance::transit(
            &state.reports,
            &state.validators.current,
            block.header.slot,
            block.header.parent,
            &block.extrinsic.assurances,
        )?;
        state.reports = reports;

        // (ρ') Update availability assignments based on guarantees
    }

    Ok(state)
}

/// (η') Updates the entropy accumulator.
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

/// (λ') Updates the previous epoch's validators.
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
