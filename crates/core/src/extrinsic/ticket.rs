//! Ticket types

use crate::{BandersnatchPublic, BandersnatchRingVrfSignature, OpaqueHash};
use serde::{Deserialize, Serialize};
use spacejson::Json;

/// Represents a unique identifier for a ticket.
pub type TicketId = OpaqueHash;

/// Represents an attempt to use a ticket.
pub type TicketAttempt = u8;

/// Represents a ticket envelope containing an attempt and a signature.
#[derive(Debug, Serialize, Deserialize, Json, PartialEq, Eq, Clone, Hash)]
pub struct TicketEnvelope {
    /// Ticket attempt
    pub attempt: TicketAttempt,
    /// Ticket ring signature
    #[json(hex)]
    #[serde(with = "codec::bytes")]
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
    /// Ticket entry index
    pub attempt: TicketAttempt,
}

impl TicketBody {
    /// Returns the message for the ticket
    ///
    /// ring VRF input data (6.29)
    /// - X_T token
    /// - η'_2 (second-oldest entropy)
    /// - r (attempt number)
    pub fn message(attempt: TicketAttempt, entropy: &[u8; 32]) -> Vec<u8> {
        [
            &crate::JAM_TICKET_SEAL,
            entropy.as_slice(),
            [attempt].as_slice(),
        ]
        .concat()
    }

    /// Sequences the tickets with Z function (outside-in)
    pub fn sequence(tickets: &[TicketBody]) -> Vec<TicketBody> {
        let mut ordered_tickets = Vec::with_capacity(tickets.len());
        let mid = tickets.len() / 2;

        for i in 0..mid {
            ordered_tickets.push(tickets[i]);
            if i + mid < tickets.len() {
                ordered_tickets.push(tickets[tickets.len() - 1 - i]);
            }
        }

        ordered_tickets
    }

    /// Get the ticket at the given slot
    ///
    /// Note that this function could be unnecessary, since the tickets are already
    /// sorted by the Z function.
    pub fn entry(tickets: &[TicketBody], slot: u32) -> TicketBody {
        let slot_in_epoch = slot % crate::EPOCH_LENGTH;
        let tickets_count = tickets.len() as u32;

        let entry_index = if slot_in_epoch < tickets_count / 2 {
            slot_in_epoch
        } else {
            tickets_count - 1 - (slot_in_epoch - tickets_count / 2)
        };

        tickets[entry_index as usize]
    }
}

/// Represents an accumulator of tickets.
pub type TicketsAccumulator = Vec<TicketBody>;

/// Represents either tickets or keys.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TicketsOrKeys {
    Keys(Vec<BandersnatchPublic>),
    Tickets(Vec<TicketBody>),
}

impl Default for TicketsOrKeys {
    fn default() -> Self {
        Self::Keys(Default::default())
    }
}

/// Represents the JSON representation of either tickets or keys.
#[derive(Serialize, Deserialize, Debug)]
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
