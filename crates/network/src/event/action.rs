//! Internal actions for the network.

use crate::Event;

/// Internal actions for the network.
///
/// This is a special event in the network which is only for internal use.
pub enum Action {
    /// Subscribe a block.
    SubscribeBlock(Vec<u8>),
}

impl From<Action> for Event {
    fn from(action: Action) -> Self {
        Event::Action(action)
    }
}
