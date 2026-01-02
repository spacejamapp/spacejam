//! Header verifier

use crate::{State, block::Header, extrinsic::TicketBody, safrole::ValidatorIter};

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

            if epoch_mark.entropy != self.entropy[0] {
                anyhow::bail!("epoch mark entropy mismatch");
            }

            if epoch_mark.tickets_entropy != self.entropy[1] {
                anyhow::bail!("epoch mark tickets entropy mismatch");
            }
        } else if new_epoch {
            anyhow::bail!("epoch mark is required");
        }

        // Validate tickets mark per GP eq 262-265:
        // H_winnersmark ≡ Z(accumulator) when e' = e ∧ m < Y ≤ m' ∧ |accumulator| = E
        //               ≡ None otherwise
        let curr_epoch = header.slot / crate::EPOCH_LENGTH;
        let prev_epoch = self.timeslot / crate::EPOCH_LENGTH;
        let curr_slot_phase = header.slot % crate::EPOCH_LENGTH;
        let prev_slot_phase = self.timeslot % crate::EPOCH_LENGTH;
        let accumulator_full = self.safrole.accumulator.len() == crate::EPOCH_LENGTH as usize;

        // Condition: same epoch, prior slot before tail start, current slot at/after tail start, accumulator full
        let should_have_tickets_mark = curr_epoch == prev_epoch
            && prev_slot_phase < crate::TICKET_SUBMISSION_PERIOD
            && curr_slot_phase >= crate::TICKET_SUBMISSION_PERIOD
            && accumulator_full;

        if let Some(tickets_mark) = header.tickets_mark {
            if !should_have_tickets_mark {
                anyhow::bail!(
                    "tickets mark present but not expected: curr_epoch={}, prev_epoch={}, \
                     curr_phase={}, prev_phase={}, accumulator_len={}",
                    curr_epoch,
                    prev_epoch,
                    curr_slot_phase,
                    prev_slot_phase,
                    self.safrole.accumulator.len()
                );
            }

            // Validate content: tickets_mark == Z(accumulator)
            let expected = TicketBody::sequence(&self.safrole.accumulator);
            if tickets_mark != expected {
                anyhow::bail!("tickets mark content mismatch");
            }

            // Validate ticket attempts
            for ticket in tickets_mark {
                if ticket.attempt > crate::TICKET_ENTRIES_PER_VALIDATOR as u8 {
                    anyhow::bail!("invalid ticket attempt {}", ticket.attempt);
                }
            }
        } else if should_have_tickets_mark {
            anyhow::bail!(
                "tickets mark required but not present: curr_phase={}, prev_phase={}, accumulator_len={}",
                curr_slot_phase,
                prev_slot_phase,
                self.safrole.accumulator.len()
            );
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
