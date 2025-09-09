//! Macro blocks

use crate::{Exit, Translator};
use cranelift::prelude::{types, Block, InstBuilder, IntCC};
use cranelift_frontend::FunctionBuilder;

const HALT_TARGET: u64 = (u32::MAX - u16::MAX as u32) as u64;

/// Macro blocks
pub struct MacroBlocks {
    /// The trap block
    pub trap: Block,

    /// The djump processor
    pub djump: Block,
}

impl MacroBlocks {
    /// Create a new macro blocks
    pub fn new(builder: &mut FunctionBuilder) -> Self {
        let djump = builder.create_block();
        builder.append_block_param(djump, types::I64);

        Self {
            trap: builder.create_block(),
            djump,
        }
    }
}

impl Translator<'_> {
    /// Seal the macro blocks
    pub fn build_macros(&mut self) {
        self.build_trap();
        self.build_djump();
    }

    /// Seal the trap block
    fn build_trap(&mut self) {
        let trap = self.masm.trap;
        self.builder.switch_to_block(trap);
        self.return_(Exit::InvalidJumpTarget);
    }

    /// Seal the djump block
    fn build_djump(&mut self) {
        self.context.builder.switch_to_block(self.masm.djump);
        let target = self.builder.block_params(self.masm.djump)[0];
        let halt_block = self.builder.create_block();
        let check_valid = self.builder.create_block();
        let halt = self.builder.ins().iconst(types::I64, HALT_TARGET as i64);
        let is_halt = self.builder.ins().icmp(IntCC::Equal, target, halt);
        self.builder
            .ins()
            .brif(is_halt, halt_block, &[], check_valid, &[]);

        // Halt block: return halt result
        self.builder.switch_to_block(halt_block);
        self.return_(Exit::Halt);

        // Jump target validation:
        // 1. address == 0 (null address)
        // 2. address > table.len() * JUMP_ALIGNMENT_FACTOR (beyond table bounds)
        // 3. address % 2 != 0 (not aligned to 2-byte boundary)
        self.builder.switch_to_block(check_valid);
        let valid = self.builder.create_block();
        let two = self.builder.ins().iconst(types::I64, 2);
        {
            // Check 1: address == 0
            let zero = self.builder.ins().iconst(types::I64, 0);
            let is_zero = self.builder.ins().icmp(IntCC::Equal, target, zero);

            // Check 2: address > table.len() * JUMP_ALIGNMENT_FACTOR
            let table_len = self.jump.len() as u32;
            let max_address = table_len * pvm::JUMP_ALIGNMENT_FACTOR;
            let max_addr_val = self.builder.ins().iconst(types::I64, max_address as i64);
            let exceeds_bounds =
                self.builder
                    .ins()
                    .icmp(IntCC::UnsignedGreaterThan, target, max_addr_val);

            // Check 3: address % 2 != 0 (misaligned)
            let remainder = self.builder.ins().urem(target, two);
            let is_misaligned = self.builder.ins().icmp(IntCC::NotEqual, remainder, zero);

            // Combine all invalid conditions with OR
            let invalid = self.builder.ins().bor(is_zero, exceeds_bounds);
            let invalid_jump = self.builder.ins().bor(invalid, is_misaligned);
            self.context
                .builder
                .ins()
                .brif(invalid_jump, self.masm.trap, &[], valid, &[]);
        }

        // Valid jump block: calculate index and dispatch
        self.builder.switch_to_block(valid);
        {
            // Calculate jump table index: (address / 2) - 1
            let addr_div_2 = self.builder.ins().udiv(target, two);
            let one = self.builder.ins().iconst(types::I64, 1);
            let jump_index = self.builder.ins().isub(addr_div_2, one);
            let jump_index = self.builder.ins().ireduce(types::I32, jump_index);
            self.context
                .builder
                .ins()
                .br_table(jump_index, self.rt_jump_table);
        }

        // Seal all created blocks
        self.builder.seal_block(halt_block);
        self.builder.seal_block(check_valid);
        self.builder.seal_block(valid);
    }
}
