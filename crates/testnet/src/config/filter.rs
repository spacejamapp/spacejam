//! Filter configuration.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Filter {
    /// The log filters of the filter.
    #[serde(default)]
    pub filters: Vec<String>,
}

impl Filter {
    /// Check if a log message should be filtered.
    pub fn check(&self, msg: &str) -> bool {
        if !self.filters.is_empty() && !self.filters.iter().any(|filter| msg.contains(filter)) {
            return false;
        }

        true
    }
}
