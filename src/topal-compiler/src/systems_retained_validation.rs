//! Native checks generated from the shared ordinary retained value layout.
use super::super::R::_Sp as Sp;
use super::super::{CompileError, E, R::*};
use crate::systems_retained_layout::RetainedValueLayout;

pub(super) fn emit(e: &mut E<'_>, layout: &RetainedValueLayout) -> Result<(), CompileError> {
    // R13 is the selected bank. The enclosing invocation already saved every
    // native callee register; only the fixed caller-owned result slots are read.
    for (index, number) in layout.numbers.iter().enumerate() {
        e.mov(R14, R13);
        e.add(R14, number.node_offset as i32);
        e.load(A, R13, number.pointer_offset as i32);
        e.rr(0x39, A, R14);
        e.0.jump_if(0x85, "fatal");
        e.load(A, R14, 0);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fatal");
        e.load(R8, R14, 8);
        e.load(R9, R14, 16);
        let single = format!("retained-graph-{index}-single");
        let normal = format!("retained-graph-{index}-normal");
        e.cmp(R8, 1);
        e.0.jump_if(0x84, &single);
        e.cmp(R8, 2);
        e.0.jump_if(0x85, "fatal");
        e.imm(A, u32::MAX as u64);
        e.rr(0x39, R9, A);
        e.0.jump_if(0x86, "fatal");
        e.0.jump(&normal);
        e.0.bind(&single)?;
        e.imm(A, u32::MAX as u64);
        e.rr(0x39, R9, A);
        e.0.jump_if(0x87, "fatal");
        e.0.bind(&normal)?;
        e.imm(A, number.minimum);
        e.rr(0x39, R9, A);
        e.0.jump_if(0x82, "fatal");
        if number.maximum != u64::MAX {
            e.imm(A, number.maximum);
            e.rr(0x39, R9, A);
            e.0.jump_if(0x87, "fatal");
        }
        if number.path == ["next"] {
            e.load(D, Sp, 8);
            e.rr(0x39, R9, D);
            e.0.jump_if(0x85, "fatal");
        }
    }
    for (index, order) in layout.orders.iter().enumerate() {
        match order.offsets.as_slice() {
            [offset] if offset % 8 == 0 => {
                // Complete zero word includes the order's upper padding.
                e.load(A, R13, *offset as i32);
                e.cmp(A, 0);
                e.0.jump_if(0x85, "fatal");
            }
            [first, second] if *second == *first + 4 => {
                e.load(A, R13, *first as i32);
                let valid = format!("retained-order-{index}-valid");
                e.imm(R8, UINT32_PAIR_01);
                e.rr(0x39, A, R8);
                e.0.jump_if(0x84, &valid);
                e.cmp(A, 1);
                e.0.jump_if(0x85, "fatal");
                e.0.bind(&valid)?;
            }
            _ => {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-RETAINED-LAYOUT: unqualified native Record order extent".into(),
                ));
            }
        }
    }
    for mask in layout.separate_zero_masks() {
        e.load(A, R13, mask.offset as i32);
        e.imm(R8, mask.mask);
        e.rr(0x21, A, R8);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fatal");
    }
    Ok(())
}
const UINT32_PAIR_01: u64 = 1u64 << 32;
