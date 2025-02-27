//! Network handle for Spacejam.

use crate::Context;
use crate::{event, Event};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Network handle for Spacejam.
pub struct Handle<C: Context + Send + Sync + 'static> {
    /// The context of the network
    pub context: Arc<C>,

    /// Action receiver
    pub arx: mpsc::UnboundedReceiver<event::action::Event>,

    /// Event receiver
    pub prx: mpsc::UnboundedReceiver<event::peer::Event>,
}

impl<C: Context + Send + Sync + 'static> Handle<C> {
    /// Spawn the network
    pub async fn spawn(self) {
        // Spawn the event handling loop
        let mut arx = self.arx;
        let mut prx = self.prx;

        loop {
            let ctx = self.context.clone();
            let e = tokio::select! {
                Some(act) = arx.recv() => Event::Action(act),
                Some(ev) = prx.recv() => Event::Peer(ev),
                else => {
                    tracing::error!("all channels closed, terminating event loop");
                    break;
                }
            };

            e.handle_unchecked(ctx).await;
        }
    }
}
