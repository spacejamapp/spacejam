//! Network handle for Spacejam.

use crate::Context;
use crate::Event;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Network handle for Spacejam.
pub struct Handle<C: Context + Send + Sync + 'static> {
    /// The context of the network
    pub context: Arc<C>,

    /// Action receiver
    pub rx: mpsc::UnboundedReceiver<Event>,
}

impl<C: Context + Send + Sync + 'static> Handle<C> {
    /// Spawn the network
    pub async fn spawn(mut self) {
        while let Some(e) = self.rx.recv().await {
            e.handle_unchecked(self.context.clone()).await;
        }
    }
}
