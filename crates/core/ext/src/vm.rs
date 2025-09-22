//! VM extensions

use crate::AccountsExt;
use score::{
    Accounts,
    vm::{AccumulateState, IndexSalt},
};
use score::{ServiceId, TimeSlot};

/// (I) Generate a new index from provided environment
#[cfg(feature = "blake2")]
pub fn index<R: Accounts>(
    state: &mut AccumulateState<R>,
    service: ServiceId,
    timeslot: TimeSlot,
) -> ServiceId {
    let encoded = codec::encode(&IndexSalt {
        service,
        entropy: state.entropy[0],
        timeslot,
    })
    .expect("failed to encode");
    let hash = crypto::blake2b(&encoded);
    let base = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
    let index = (base % score::CHECK_SALT) + (1 << 8);
    state.accounts.check(index)
}
