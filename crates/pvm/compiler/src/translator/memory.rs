//! Memory operation translation

use crate::Translator;
use cranelift::prelude::*;

// Context structure offsets
const MEMORY_PTR_OFFSET: i64 = 112; // 13*8 + 8 = registers + pc
const MAX_MEMORY: i64 = 0x100000; // 1MB linear memory size
const OPS_COUNT_OFFSET: i64 = 1664; // memory_ops_count offset
const OPS_OFFSET: i64 = 128; // memory_ops array offset
const OP_SIZE: i64 = 24; // Size of each MemoryOp struct
const MAX_OPS: i32 = 64; // Maximum recorded memory operations

/// Memory operation sizes
#[derive(Debug, Copy, Clone)]
pub enum MemorySize {
    Byte,  // 8 bits
    Word,  // 16 bits
    DWord, // 32 bits
    QWord, // 64 bits
}

impl Translator<'_, '_> {
    /// Generate direct linear memory read
    pub fn emit_memory_read(&mut self, address: Value, size: MemorySize) -> Value {
        // Get the context pointer - use current block if it has params, otherwise entry block
        let context_param = if let Some(current_block) = self.builder.current_block() {
            let params = self.builder.block_params(current_block);
            if !params.is_empty() {
                params[0]
            } else {
                // Fall back to entry block
                let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
                self.builder.block_params(entry_block)[0]
            }
        } else {
            // Fall back to entry block
            let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
            self.builder.block_params(entry_block)[0]
        };

        // Convert address to 32-bit for PVM memory addressing
        let addr_32 = self.builder.ins().ireduce(types::I32, address);
        let addr_64 = self.builder.ins().uextend(types::I64, addr_32);

        // Load linear memory pointer from context
        let mem_ptr_addr = self
            .builder
            .ins()
            .iadd_imm(context_param, MEMORY_PTR_OFFSET);
        let memory_ptr = self
            .builder
            .ins()
            .load(types::I64, MemFlags::new(), mem_ptr_addr, 0);

        // Bounds check: address < MAX_MEMORY
        let in_bounds = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThan, addr_64, MAX_MEMORY);

        // Calculate read address
        let read_addr = self.builder.ins().iadd(memory_ptr, addr_64);

        // Load from linear memory with proper type
        let loaded_value = match size {
            MemorySize::Byte => self
                .builder
                .ins()
                .load(types::I8, MemFlags::new(), read_addr, 0),
            MemorySize::Word => self
                .builder
                .ins()
                .load(types::I16, MemFlags::new(), read_addr, 0),
            MemorySize::DWord => self
                .builder
                .ins()
                .load(types::I32, MemFlags::new(), read_addr, 0),
            MemorySize::QWord => self
                .builder
                .ins()
                .load(types::I64, MemFlags::new(), read_addr, 0),
        };

        // Return 0 if out of bounds, otherwise return the loaded value
        let zero_value = match size {
            MemorySize::Byte => self.builder.ins().iconst(types::I8, 0),
            MemorySize::Word => self.builder.ins().iconst(types::I16, 0),
            MemorySize::DWord => self.builder.ins().iconst(types::I32, 0),
            MemorySize::QWord => self.builder.ins().iconst(types::I64, 0),
        };

        self.builder
            .ins()
            .select(in_bounds, loaded_value, zero_value)
    }

    /// Generate signed memory read
    pub fn emit_memory_read_signed(&mut self, address: Value, size: MemorySize) -> Value {
        // Read the unsigned value first
        let unsigned_value = self.emit_memory_read(address, size);

        // Sign-extend to 64-bit based on the size
        match size {
            MemorySize::Byte => self.builder.ins().sextend(types::I64, unsigned_value),
            MemorySize::Word => self.builder.ins().sextend(types::I64, unsigned_value),
            MemorySize::DWord => self.builder.ins().sextend(types::I64, unsigned_value),
            MemorySize::QWord => unsigned_value, // Already 64-bit
        }
    }

    /// Generate direct memory write using inline pointer dereferencing
    pub fn emit_memory_write(&mut self, address: Value, value: Value, size: MemorySize) {
        // Get the context pointer - use current block if it has params, otherwise entry block
        let context_param = if let Some(current_block) = self.builder.current_block() {
            let params = self.builder.block_params(current_block);
            if !params.is_empty() {
                params[0]
            } else {
                // Fall back to entry block
                let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
                self.builder.block_params(entry_block)[0]
            }
        } else {
            // Fall back to entry block
            let entry_block = self.builder.func.layout.blocks().nth(0).unwrap();
            self.builder.block_params(entry_block)[0]
        };

        // Convert address to 32-bit for PVM memory addressing
        let addr_32 = self.builder.ins().ireduce(types::I32, address);

        // Prepare value in correct type
        let value_type = self.builder.func.dfg.value_type(value);
        let write_value = match size {
            MemorySize::Byte => {
                if value_type.bits() > 8 {
                    self.builder.ins().ireduce(types::I8, value)
                } else {
                    value
                }
            }
            MemorySize::Word => {
                if value_type.bits() > 16 {
                    self.builder.ins().ireduce(types::I16, value)
                } else {
                    value
                }
            }
            MemorySize::DWord => {
                if value_type.bits() > 32 {
                    self.builder.ins().ireduce(types::I32, value)
                } else {
                    value
                }
            }
            MemorySize::QWord => value,
        };

        // Record the memory write operation in the context for post-execution processing
        // This is a simplified approach that avoids complex JIT memory integration

        // Convert value to 64-bit for storage
        let value_64 = match size {
            MemorySize::Byte | MemorySize::Word | MemorySize::DWord => {
                self.builder.ins().uextend(types::I64, write_value)
            }
            MemorySize::QWord => write_value,
        };

        // Size byte
        let size_byte = match size {
            MemorySize::Byte => 1u8,
            MemorySize::Word => 2u8,
            MemorySize::DWord => 4u8,
            MemorySize::QWord => 8u8,
        };
        let size_val = self.builder.ins().iconst(types::I8, size_byte as i64);
        let is_write_val = self.builder.ins().iconst(types::I8, 1); // True for write

        // Use predefined context offsets
        let ops_count_offset = OPS_COUNT_OFFSET;
        let ops_offset = OPS_OFFSET;
        let op_size = OP_SIZE;

        // Load current memory_ops_count
        let ops_count_addr = self.builder.ins().iadd_imm(context_param, ops_count_offset);
        let current_count = self
            .builder
            .ins()
            .load(types::I32, MemFlags::new(), ops_count_addr, 0);

        // Check if we have space for recording (count < MAX_OPS)
        let max_ops = self.builder.ins().iconst(types::I32, MAX_OPS as i64);
        let has_space = self
            .builder
            .ins()
            .icmp(IntCC::UnsignedLessThan, current_count, max_ops);

        // Record operation for post-execution processing (trap handling in apply_memory_operations)
        let can_write = has_space;

        // Create blocks for conditional recording
        let record_block = self.builder.create_block();
        let done_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(can_write, record_block, &[], done_block, &[]);

        // Record memory operation when we have space
        self.builder.switch_to_block(record_block);

        // Calculate address of current memory operation
        let count_64 = self.builder.ins().uextend(types::I64, current_count);
        let op_size_val = self.builder.ins().iconst(types::I64, op_size);
        let offset_from_ops = self.builder.ins().imul(count_64, op_size_val);
        let ops_base_addr = self.builder.ins().iadd_imm(context_param, ops_offset);
        let current_op_addr = self.builder.ins().iadd(ops_base_addr, offset_from_ops);

        // Store the memory operation fields
        self.builder
            .ins()
            .store(MemFlags::new(), addr_32, current_op_addr, 0); // address
        self.builder
            .ins()
            .store(MemFlags::new(), value_64, current_op_addr, 8); // value
        self.builder
            .ins()
            .store(MemFlags::new(), size_val, current_op_addr, 16); // size
        self.builder
            .ins()
            .store(MemFlags::new(), is_write_val, current_op_addr, 17); // is_write

        // Increment memory_ops_count
        let new_count = self.builder.ins().iadd_imm(current_count, 1);
        self.builder
            .ins()
            .store(MemFlags::new(), new_count, ops_count_addr, 0);

        self.builder.ins().jump(done_block, &[]);

        // Continue execution
        self.builder.switch_to_block(done_block);
        self.builder.seal_block(record_block);
        self.builder.seal_block(done_block);
    }
}
