//! Math operations with safe division and remainder

use crate::Translator;
use cranelift::prelude::*;

impl Translator<'_> {
    /// Check if a value is zero, returning the comparison result
    fn is_zero(&mut self, value: Value, ty: types::Type) -> Value {
        let zero = self.builder.ins().iconst(ty, 0);
        self.builder.ins().icmp(IntCC::Equal, value, zero)
    }

    /// Create a safe divisor (replace zero with one to avoid division faults)
    fn safe_divisor(&mut self, divisor: Value, is_zero: Value, ty: types::Type) -> Value {
        let one = self.builder.ins().iconst(ty, 1);
        self.builder.ins().select(is_zero, one, divisor)
    }

    /// Check for signed division overflow (MIN_VALUE / -1)
    fn is_signed_div_overflow(
        &mut self,
        dividend: Value,
        divisor: Value,
        ty: types::Type,
    ) -> Value {
        let min_val = match ty {
            types::I32 => self.builder.ins().iconst(ty, i32::MIN as i64),
            types::I64 => self.builder.ins().iconst(ty, i64::MIN),
            _ => panic!("Unsupported type for signed overflow check"),
        };
        let neg_one = self.builder.ins().iconst(ty, -1);
        let is_min = self.builder.ins().icmp(IntCC::Equal, dividend, min_val);
        let is_neg_one = self.builder.ins().icmp(IntCC::Equal, divisor, neg_one);
        self.builder.ins().band(is_min, is_neg_one)
    }

    /// Create constants for error return values
    fn div_error_constants(&mut self) -> (Value, Value, Value) {
        let max_val = self.builder.ins().iconst(types::I64, u64::MAX as i64);
        let min_val = self.builder.ins().iconst(types::I64, i32::MIN as i64);
        let zero_val = self.builder.ins().iconst(types::I64, 0);
        (max_val, min_val, zero_val)
    }

    /// Safe signed 32-bit division with PVM semantics
    /// Returns u64::MAX for div by zero, i32::MIN for overflow, otherwise quotient
    pub fn safe_div_s32(&mut self, dividend: Value, divisor: Value) -> Value {
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);
        let is_zero = self.is_zero(divisor_32, types::I32);
        let is_overflow = self.is_signed_div_overflow(dividend_32, divisor_32, types::I32);

        // Safe division
        let safe_divisor = self.safe_divisor(divisor_32, is_zero, types::I32);
        let safe_divisor = self.safe_divisor(safe_divisor, is_overflow, types::I32);
        let result_32 = self.builder.ins().sdiv(dividend_32, safe_divisor);
        let result_ext = self.builder.ins().sextend(types::I64, result_32);

        // Error handling
        let (max_val, min_val, _) = self.div_error_constants();
        let result_or_overflow = self.builder.ins().select(is_overflow, min_val, result_ext);
        self.builder
            .ins()
            .select(is_zero, max_val, result_or_overflow)
    }

    /// Safe signed 64-bit division with PVM semantics
    pub fn safe_div_s64(&mut self, dividend: Value, divisor: Value) -> Value {
        let is_zero = self.is_zero(divisor, types::I64);
        let is_overflow = self.is_signed_div_overflow(dividend, divisor, types::I64);

        // Safe division
        let safe_divisor = self.safe_divisor(divisor, is_zero, types::I64);
        let safe_divisor = self.safe_divisor(safe_divisor, is_overflow, types::I64);
        let result = self.builder.ins().sdiv(dividend, safe_divisor);

        // Error handling - for 64-bit, overflow returns original dividend
        let (max_val, _, _) = self.div_error_constants();
        let result_or_overflow = self.builder.ins().select(is_overflow, dividend, result);
        self.builder
            .ins()
            .select(is_zero, max_val, result_or_overflow)
    }

    /// Safe unsigned division (common logic for 32/64-bit)
    fn safe_udiv_common(&mut self, dividend: Value, divisor: Value, is_32bit: bool) -> Value {
        let (div_val, divisor_val, result_ty) = if is_32bit {
            let div_32 = self.builder.ins().ireduce(types::I32, dividend);
            let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);
            (div_32, divisor_32, types::I32)
        } else {
            (dividend, divisor, types::I64)
        };

        let is_zero = self.is_zero(divisor_val, result_ty);
        let safe_divisor = self.safe_divisor(divisor_val, is_zero, result_ty);
        let result = self.builder.ins().udiv(div_val, safe_divisor);
        let final_result = if is_32bit {
            self.builder.ins().sextend(types::I64, result)
        } else {
            result
        };

        // Return u64::MAX for division by zero
        let (max_val, _, _) = self.div_error_constants();
        self.builder.ins().select(is_zero, max_val, final_result)
    }

    /// Safe unsigned 32-bit division
    pub fn safe_div_u32(&mut self, dividend: Value, divisor: Value) -> Value {
        self.safe_udiv_common(dividend, divisor, true)
    }

    /// Safe unsigned 64-bit division
    pub fn safe_div_u64(&mut self, dividend: Value, divisor: Value) -> Value {
        self.safe_udiv_common(dividend, divisor, false)
    }

    /// Safe signed 32-bit remainder
    pub fn safe_rem_s32(&mut self, dividend: Value, divisor: Value) -> Value {
        let dividend_32 = self.builder.ins().ireduce(types::I32, dividend);
        let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);
        let is_zero = self.is_zero(divisor_32, types::I32);
        let is_overflow = self.is_signed_div_overflow(dividend_32, divisor_32, types::I32);

        // Safe remainder
        let safe_divisor = self.safe_divisor(divisor_32, is_zero, types::I32);
        let safe_divisor = self.safe_divisor(safe_divisor, is_overflow, types::I32);
        let result_32 = self.builder.ins().srem(dividend_32, safe_divisor);
        let result_ext = self.builder.ins().sextend(types::I64, result_32);

        // Error handling: div by zero returns dividend, overflow returns 0
        let dividend_ext = self.builder.ins().sextend(types::I64, dividend_32);
        let (_, _, zero_val) = self.div_error_constants();
        let result_or_overflow = self.builder.ins().select(is_overflow, zero_val, result_ext);
        self.builder
            .ins()
            .select(is_zero, dividend_ext, result_or_overflow)
    }

    /// Safe signed 64-bit remainder
    pub fn safe_rem_s64(&mut self, dividend: Value, divisor: Value) -> Value {
        let is_zero = self.is_zero(divisor, types::I64);
        let is_overflow = self.is_signed_div_overflow(dividend, divisor, types::I64);

        // Safe remainder
        let safe_divisor = self.safe_divisor(divisor, is_zero, types::I64);
        let safe_divisor = self.safe_divisor(safe_divisor, is_overflow, types::I64);
        let result = self.builder.ins().srem(dividend, safe_divisor);

        // Error handling: div by zero returns dividend, overflow returns 0
        let (_, _, zero_val) = self.div_error_constants();
        let result_or_overflow = self.builder.ins().select(is_overflow, zero_val, result);
        self.builder
            .ins()
            .select(is_zero, dividend, result_or_overflow)
    }

    /// Safe unsigned remainder (common logic)
    fn safe_urem_common(&mut self, dividend: Value, divisor: Value, is_32bit: bool) -> Value {
        let (div_val, divisor_val, result_ty) = if is_32bit {
            let div_32 = self.builder.ins().ireduce(types::I32, dividend);
            let divisor_32 = self.builder.ins().ireduce(types::I32, divisor);
            (div_32, divisor_32, types::I32)
        } else {
            (dividend, divisor, types::I64)
        };

        let is_zero = self.is_zero(divisor_val, result_ty);
        let safe_divisor = self.safe_divisor(divisor_val, is_zero, result_ty);
        let result = self.builder.ins().urem(div_val, safe_divisor);
        let (final_result, dividend_for_error) = if is_32bit {
            let result_ext = self.builder.ins().sextend(types::I64, result);
            let div_ext = self.builder.ins().sextend(types::I64, div_val);
            (result_ext, div_ext)
        } else {
            (result, div_val)
        };

        // Return original dividend for division by zero
        self.builder
            .ins()
            .select(is_zero, dividend_for_error, final_result)
    }

    /// Safe unsigned 32-bit remainder
    pub fn safe_rem_u32(&mut self, dividend: Value, divisor: Value) -> Value {
        self.safe_urem_common(dividend, divisor, true)
    }

    /// Safe unsigned 64-bit remainder
    pub fn safe_rem_u64(&mut self, dividend: Value, divisor: Value) -> Value {
        self.safe_urem_common(dividend, divisor, false)
    }
}
