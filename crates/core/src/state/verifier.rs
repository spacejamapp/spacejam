//! Header verifier

use crate::{State, block::Header, safrole::ValidatorIter};

impl State {
    /// Verify the header
    ///
    /// NOTE: this is for doing the header validation, currently
    /// inside the runtime.
    pub fn check(&mut self, header: &Header, new_epoch: bool) -> anyhow::Result<()> {
        if header.slot <= self.timeslot {
            anyhow::bail!("block slot is less than or equal to current height");
        }

        if header.author_index >= crate::VALIDATORS_COUNT {
            anyhow::bail!("invalid author index");
        }

        // validate the epoch mark
        if let Some(epoch_mark) = &header.epoch_mark {
            let expected = self
                .safrole
                .next(&self.validators.drawn, &header.offenders_mark)
                .evals();
            if epoch_mark.validators != expected.as_slice() {
                anyhow::bail!("epoch mark validators mismatch");
            }

            // GP (217): epoch mark = (η_0, η_1, validators)
            if epoch_mark.entropy != self.entropy[0] {
                anyhow::bail!("epoch mark entropy mismatch");
            }

            if epoch_mark.tickets_entropy != self.entropy[1] {
                anyhow::bail!("epoch mark tickets entropy mismatch");
            }
        } else if new_epoch {
            anyhow::bail!("epoch mark is required");
        }

        // handle marks in the block
        let slot_phase = header.slot % crate::EPOCH_LENGTH;
        if let Some(tickets_mark) = header.tickets_mark {
            if slot_phase < crate::TICKET_SUBMISSION_PERIOD {
                anyhow::bail!("invalid tickets mark");
            }

            for ticket in tickets_mark {
                if ticket.attempt > crate::TICKET_ENTRIES_PER_VALIDATOR as u8 {
                    anyhow::bail!("invalid ticket attempt {}", ticket.attempt);
                }
            }
        } else if slot_phase == crate::TICKET_SUBMISSION_PERIOD
            && self.safrole.accumulator.len() == crate::EPOCH_LENGTH as usize
        {
            anyhow::bail!("invalid tickets mark");
        }

        // validate the block parent and complete the state root
        if let Some(parent) = self
            .recent_blocks
            .complete_state_root(header.parent_state_root)?
            && parent != header.parent
        {
            anyhow::bail!(
                "Parent mismatch, expected: 0x{}, got: 0x{}",
                hex::encode(header.parent),
                hex::encode(parent),
            );
        }

        Ok(())
    }
}
