//! Ticket types

use crate::misc::*;
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a unique identifier for a ticket.
pub type TicketId = OpaqueHash; // Corresponds to OpaqueHash

/// Represents an attempt to use a ticket.
pub type TicketAttempt = u8; // Corresponds to U8

/// Represents a ticket envelope containing an attempt and a signature.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone)]
pub struct TicketEnvelope {
    /// Ticket attempt
    pub attempt: TicketAttempt,
    /// Ticket ring signature
    #[json(hex)]
    #[serde(with = "codec")]
    pub signature: BandersnatchRingVrfSignature,
}

impl Default for TicketEnvelope {
    fn default() -> Self {
        Self {
            attempt: 0,
            signature: [0u8; 784],
        }
    }
}

/// Represents the body of a ticket, containing an ID and an attempt.
#[derive(Debug, Serialize, Deserialize, Json, Copy, Clone, Default, PartialEq, Eq)]
pub struct TicketBody {
    #[json(hex)]
    pub id: TicketId,
    pub attempt: TicketAttempt,
}

/// Represents an accumulator of tickets.
pub type TicketsAccumulator = Vec<TicketBody>;

/// Represents either tickets or keys.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TicketsOrKeys {
    Tickets(Vec<TicketBody>),
    Keys(Vec<BandersnatchPublic>),
}

impl Default for TicketsOrKeys {
    fn default() -> Self {
        Self::Tickets(Default::default())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TicketsOrKeysJson {
    tickets: Option<Vec<TicketBodyJson>>,
    keys: Option<Vec<String>>,
}

impl Json<TicketsOrKeysJson> for TicketsOrKeys {
    fn from_json(json: TicketsOrKeysJson) -> anyhow::Result<Self> {
        Ok(if let Some(tickets) = json.tickets {
            Self::Tickets(
                tickets
                    .into_iter()
                    .map(<TicketBody as Json<TicketBodyJson>>::from_json)
                    .collect::<anyhow::Result<Vec<_>>>()?,
            )
        } else if let Some(keys) = json.keys {
            Self::Keys(
                keys.into_iter()
                    .map(|k| {
                        let mut r = [0u8; 32];
                        hex::decode(k.trim_start_matches("0x")).map(|d| r.copy_from_slice(&d))?;
                        Ok(r)
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?,
            )
        } else {
            Self::default()
        })
    }

    fn to_json(self) -> TicketsOrKeysJson {
        match self {
            Self::Tickets(tickets) => TicketsOrKeysJson {
                tickets: Some(tickets.into_iter().map(|t| t.to_json()).collect()),
                keys: None,
            },
            Self::Keys(keys) => TicketsOrKeysJson {
                tickets: None,
                keys: Some(keys.into_iter().map(hex::encode).collect()),
            },
        }
    }
}

/// Represents the extrinsic data for tickets.
pub type TicketsExtrinsic = Vec<TicketEnvelope>;
