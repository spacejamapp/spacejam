//! Account extensions

use score::{Account, Accounts, ServiceId};

/// Account extensions
pub trait AccountExt: Account {
    /// (Λ) lookup preimage in the recent histories
    fn historical_lookup(&mut self, timeslot: u32, hash: [u8; 32]) -> Option<Vec<u8>> {
        let preimage = self.preimage(hash)?;
        let lookup = self.lookup(hash, preimage.len() as u32)?;
        if (lookup.len() == 1 && timeslot >= lookup[0])
            || (lookup.len() == 2 && timeslot >= lookup[0] && timeslot <= lookup[1])
            || (lookup.len() == 3
                && ((timeslot >= lookup[0] && timeslot < lookup[1]) || timeslot >= lookup[2]))
        {
            Some(preimage)
        } else {
            None
        }
    }

    /// Add a preimage to the account
    #[cfg(feature = "blake2")]
    fn add_preimage(&mut self, preimage: Vec<u8>, timeslot: u32) -> score::OpaqueHash {
        let hash = crypto::blake2b(&preimage);
        self.insert_lookup(hash, preimage.len() as u32, vec![timeslot]);
        self.insert_preimage(hash, preimage);
        hash
    }
}

impl<T: Account> AccountExt for T {}

/// Accounts extensions
pub trait AccountsExt: Accounts {
    /// Check and find a free account index
    fn check(&mut self, mut index: ServiceId) -> ServiceId {
        loop {
            if self.get(index).is_none() {
                return index;
            }

            index = ((index - (1 << 8) + 1) % score::CHECK_SALT) + (1 << 8);
        }
    }
}

impl<T: Accounts> AccountsExt for T {}
