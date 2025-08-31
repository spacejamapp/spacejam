//! Math operations with safe division and remainder

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// Safe signed 32-bit division with PVM semantics - optimized for low register pressure
    /// Returns u64::MAX for div by zero, i32::MIN for overflow, otherwise quotient
    pub fn safe_div_s32(&mut self, dividend: Value, divisor: Value) -> Value {
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);

        // Fused zero check and overflow check - reuse constants
        let zero_32 = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero_32);

        // Check for MIN/-1 overflow in single chain
        let min_val_32 = self.builder.ins().iconst(types::I32, i32::MIN as i64);
        let neg_one_32 = self.builder.ins().iconst(types::I32, -1);
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_32, min_val_32);
        let is_neg_one = self
            .builder
            .ins()
            .icmp(IntCC::Equal, divisor_32, neg_one_32);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Create safe divisor directly without helper calls
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_div = self.builder.ins().select(is_zero, one_32, divisor_32);
        let final_div = self.builder.ins().select(is_overflow, one_32, safe_div);

        // Perform division and extend result
        let result_32 = self.builder.ins().sdiv(dividend_32, final_div);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        // Error constants and final selection - fused
        let max_val_64 = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        let min_val_64 = self.builder.ins().iconst(types::I64, i32::MIN as i64);
        let overflow_result = self
            .builder
            .ins()
            .select(is_overflow, min_val_64, result_64);
        self.builder
            .ins()
            .select(is_zero, max_val_64, overflow_result)
    }

    /// Safe signed 64-bit division with PVM semantics - optimized for low register pressure  
    pub fn safe_div_s64(&mut self, dividend: Value, divisor: Value) -> Value {
        // Fused zero and overflow checks
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor, zero_64);

        let min_val_64 = self.builder.ins().iconst(types::I64, i64::MIN);
        let neg_one_64 = self.builder.ins().iconst(types::I64, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor, neg_one_64);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Create safe divisor in single chain
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_div = self.builder.ins().select(is_zero, one_64, divisor);
        let final_div = self.builder.ins().select(is_overflow, one_64, safe_div);

        let result = self.builder.ins().sdiv(dividend, final_div);

        // Fused error handling - for 64-bit, overflow returns original dividend
        let max_val_64 = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        let overflow_result = self.builder.ins().select(is_overflow, dividend, result);
        self.builder
            .ins()
            .select(is_zero, max_val_64, overflow_result)
    }

    /// Safe unsigned 32-bit division - optimized for low register pressure
    pub fn safe_div_u32(&mut self, dividend: Value, divisor: Value) -> Value {
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);

        // Fused zero check and safe division
        let zero_32 = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero_32);

        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let result_32 = self.builder.ins().udiv(dividend_32, safe_divisor);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        // Return u64::MAX for division by zero
        let max_val_64 = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        self.builder.ins().select(is_zero, max_val_64, result_64)
    }

    /// Safe unsigned 64-bit division - optimized for low register pressure
    pub fn safe_div_u64(&mut self, dividend: Value, divisor: Value) -> Value {
        // Fused zero check and safe division
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor, zero_64);

        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor);
        let result = self.builder.ins().udiv(dividend, safe_divisor);

        // Return u64::MAX for division by zero
        let max_val_64 = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        self.builder.ins().select(is_zero, max_val_64, result)
    }

    /// Safe signed 32-bit remainder - optimized for low register pressure
    pub fn safe_rem_s32(&mut self, dividend: Value, divisor: Value) -> Value {
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);

        // Fused zero and overflow checks
        let zero_32 = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero_32);

        let min_val_32 = self.builder.ins().iconst(types::I32, i32::MIN as i64);
        let neg_one_32 = self.builder.ins().iconst(types::I32, -1);
        let is_min = self
            .builder
            .ins()
            .icmp(IntCC::Equal, dividend_32, min_val_32);
        let is_neg_one = self
            .builder
            .ins()
            .icmp(IntCC::Equal, divisor_32, neg_one_32);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Safe remainder calculation
        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_div = self.builder.ins().select(is_zero, one_32, divisor_32);
        let final_div = self.builder.ins().select(is_overflow, one_32, safe_div);
        let result_32 = self.builder.ins().srem(dividend_32, final_div);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        // Error handling: div by zero returns dividend, overflow returns 0
        let dividend_64 = self.builder.ins().sextend(types::I64, dividend_32);
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let overflow_result = self.builder.ins().select(is_overflow, zero_64, result_64);
        self.builder
            .ins()
            .select(is_zero, dividend_64, overflow_result)
    }

    /// Safe signed 64-bit remainder - optimized for low register pressure
    pub fn safe_rem_s64(&mut self, dividend: Value, divisor: Value) -> Value {
        // Fused zero and overflow checks
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor, zero_64);

        let min_val_64 = self.builder.ins().iconst(types::I64, i64::MIN);
        let neg_one_64 = self.builder.ins().iconst(types::I64, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend, min_val_64);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor, neg_one_64);
        let is_overflow = self.builder.ins().band(is_min, is_neg_one);

        // Safe remainder calculation
        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_div = self.builder.ins().select(is_zero, one_64, divisor);
        let final_div = self.builder.ins().select(is_overflow, one_64, safe_div);
        let result = self.builder.ins().srem(dividend, final_div);

        // Error handling: div by zero returns dividend, overflow returns 0
        let overflow_result = self.builder.ins().select(is_overflow, zero_64, result);
        self.builder
            .ins()
            .select(is_zero, dividend, overflow_result)
    }

    /// Safe unsigned 32-bit remainder - optimized for low register pressure
    pub fn safe_rem_u32(&mut self, dividend: Value, divisor: Value) -> Value {
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);

        // Fused zero check and safe remainder
        let zero_32 = self.builder.ins().iconst(types::I32, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor_32, zero_32);

        let one_32 = self.builder.ins().iconst(types::I32, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_32, divisor_32);
        let result_32 = self.builder.ins().urem(dividend_32, safe_divisor);
        let result_64 = self.builder.ins().sextend(types::I64, result_32);

        // Return original dividend for division by zero
        let dividend_64 = self.builder.ins().sextend(types::I64, dividend_32);
        self.builder.ins().select(is_zero, dividend_64, result_64)
    }

    /// Safe unsigned 64-bit remainder - optimized for low register pressure
    pub fn safe_rem_u64(&mut self, dividend: Value, divisor: Value) -> Value {
        // Fused zero check and safe remainder
        let zero_64 = self.builder.ins().iconst(types::I64, 0);
        let is_zero = self.builder.ins().icmp(IntCC::Equal, divisor, zero_64);

        let one_64 = self.builder.ins().iconst(types::I64, 1);
        let safe_divisor = self.builder.ins().select(is_zero, one_64, divisor);
        let result = self.builder.ins().urem(dividend, safe_divisor);

        // Return original dividend for division by zero
        self.builder.ins().select(is_zero, dividend, result)
    }
}
