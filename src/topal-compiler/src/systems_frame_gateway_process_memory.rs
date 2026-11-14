//! TK-DEC-284 / TOPAL-SYSTEMS-PROCESS-MM-001 private compiled caller admission.
use super::*;

pub(super) fn entry(code: &mut Code) -> Result<(), CompileError> {
    let mut e = E(&mut code.encoder);
    e.0.bind("mm_process_entry")?;
    for r in [D, C, R8] {
        e.cmp(r, 0);
        e.0.jump_if(0x85, "unsafe");
    }
    e.load(A, _Sp, 0);
    for (op, site, numbers) in [
        (9, "topal_x86_process_mm_prepare_return_v1", &[56, 57][..]),
        (10, "topal_x86_process_mm_commit_return_v1", &[56, 57][..]),
        (
            11,
            "topal_x86_process_mm_switch_return_v1",
            &[0, 1, 19, 20, 24, 61][..],
        ),
        (
            12,
            "topal_x86_process_mm_child_exit_return_v1",
            &[60, 231][..],
        ),
        (13, "topal_x86_process_mm_cancel_return_v1", &[60, 231][..]),
    ] {
        let next = format!("mm_process_next_{op}");
        let admitted = format!("mm_process_number_{op}");
        e.cmp(Di, op);
        e.0.jump_if(0x85, &next);
        address(&mut e, &mut code.references, R10, site);
        e.rr(0x39, A, R10);
        e.0.jump_if(0x85, "unsafe");
        if op == 10 {
            e.cmp(Si, 2);
            e.0.jump_if(0x82, "unsafe");
            e.cmp(Si, i32::MAX);
            e.0.jump_if(0x87, "unsafe");
        } else {
            e.cmp(Si, 0);
            e.0.jump_if(0x85, "unsafe");
        }
        address(&mut e, &mut code.references, R10, EPOCH);
        e.load(R9, R10, 8);
        for number in numbers {
            e.cmp(R9, *number);
            e.0.jump_if(0x84, &admitted);
        }
        e.0.jump("unsafe");
        e.0.bind(&admitted)?;
        e.0.jump("mm_syscall_context");
        e.0.bind(&next)?;
    }
    e.0.jump("unsafe");
    Ok(())
}
