//! Filter configuration.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Filter {
    /// The log filters of the filter.
    #[serde(default)]
    pub filter: Vec<String>,
}

impl Filter {
    /// Check if a log message should be filtered.
    pub fn skip(&self, msg: &str) -> bool {
        if self.filter.is_empty() {
            return false;
        }

        if !self.filter.iter().any(|filter| msg.contains(filter)) {
            return true;
        }

        false
    }
}
