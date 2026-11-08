//! Kernel root adapter. Source never constructs any address or context here.
use super::*;
#[path = "systems_tail_kernel_tests.rs"]
#[cfg(test)]
mod tests;
pub(crate) const APPLY: &str = "topal_x86_systems_region_library_apply_v1";
pub(crate) const FINALIZE: &str = "topal_x86_systems_region_library_finalize_v1";
pub(crate) const PRIVATE: &str = "topal_x86_tail_kernel_private_v1";
pub(crate) const STACK: &str = "topal_x86_tail_kernel_stack_v1";
pub(crate) const HISTORY: &str = "topal_x86_tail_kernel_history_v1";
pub(crate) const OUTPUT: &str = "topal_x86_tail_kernel_output_v1";
pub(crate) const RECOVERY: &str = "topal_x86_tail_kernel_recovery_v1";
const OWNER: &str = "topal_tail_owner_invoke_v1";
const FRAME_BYTES: u64 = 80;
const SPILL_BYTES: i32 = 48;

fn bss(object: &mut Object<'_>, name: &str, bytes: u64, alignment: u64) {
    let section = object.add_section(
        Vec::new(),
        format!(".bss.{name}").into_bytes(),
        SectionKind::UninitializedData,
    );
    let value = object.append_section_bss(section, bytes, alignment);
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value,
        size: bytes,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
}
fn zero(code: &mut Code, symbol: &'static str, bytes: u64, fail: &str) {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, symbol);
    e.mov(Di, R11);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, fail);
}
fn wipe(code: &mut Code, symbol: &'static str, bytes: u64, fail: &str) {
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, symbol);
    erase(&mut e, R11, bytes, fail);
}
fn halt(code: &mut Code, label: &str) -> Result<(), CompileError> {
    code.encoder.bind(label)?;
    code.encoder.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    Ok(())
}
fn erase_caller(code: &mut Code, bytes: u64, fail: &str) {
    let mut e = E(&mut code.encoder);
    e.mov(R11, R12);
    e.add(R11, -(bytes as i32));
    erase(&mut e, R11, bytes, fail);
}
fn spill_and_recovery(code: &mut Code, failure: &str) {
    let mut e = E(&mut code.encoder);
    e.imm(A, 0);
    for offset in (-SPILL_BYTES..0).step_by(8) {
        e.store(_Sp, offset, A);
    }
    for offset in (-SPILL_BYTES..0).step_by(8) {
        e.load(A, _Sp, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, failure);
    }
    address(&mut e, &mut code.references, A, RECOVERY);
    for offset in (0..32).step_by(8) {
        e.mem(0xc7, R::A, R::A, offset);
        e.0.bytes(&[0; 4]);
    }
    for offset in (0..32).step_by(8) {
        e.mem(0x81, Di, R::A, offset);
        e.0.bytes(&[0; 4]);
        e.0.jump_if(0x85, failure);
    }
    clear_volatile(&mut e);
}
fn late_failure(code: &mut Code) -> Result<(), CompileError> {
    code.encoder.bind("late_failure")?;
    // All source/controller domains have already been erased and all original
    // caller registers restored. Never reload erased saved-register slots.
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, A, HISTORY);
    e.mem(0xc7, R::A, R::A, 24);
    e.0.bytes(&[0; 4]);
    e.mem(0xc7, R::A, R::A, 32);
    e.0.bytes(&1_i32.to_le_bytes());
    spill_and_recovery(code, "halt");
    halt(code, "halt")
}

fn apply(storage_bytes: u64, retired: u64) -> Result<(Code, u64), CompileError> {
    let mut code = Code::default();
    {
        let mut e = E(&mut code.encoder);
        e.cmp(_Sp, 0x109000);
        e.0.jump_if(0x82, "unsafe");
        e.cmp(_Sp, 0x17fff8);
        e.0.jump_if(0x87, "unsafe");
        e.mov(A, _Sp);
        e.arith(4, A, 7);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "unsafe");
        e.imm(A, 0);
        e.0.bytes(&[0x8c, 0xc8]);
        e.arith(4, A, 3);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "unsafe");
        e.0.bytes(&[0x9c]);
        row(&mut code, vec![0x0e, 16]);
    }
    {
        let e = E(&mut code.encoder);
        e.0.bytes(&[0x58]);
    }
    row(&mut code, vec![0x0e, 8]);
    {
        let mut e = E(&mut code.encoder);
        e.arith(4, A, 0x200);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "unsafe");
    }
    push_caller(&mut code);
    E(&mut code.encoder).mov(R12, _Sp);
    saved_rows(&mut code, 12);
    {
        let mut e = E(&mut code.encoder);
        e.mov(R13, Di);
        e.mov(R14, D);
        e.cmp(Si, 64);
        e.0.jump_if(0x85, "refuse");
        e.mov(A, R13);
        e.arith(4, A, 7);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "refuse");
        address(&mut e, &mut code.references, R11, "topal_bootstrap_storage");
        e.rr(0x39, R13, R11);
        e.0.jump_if(0x82, "refuse");
        e.mov(A, R13);
        e.rr(0x29, A, R11);
        e.cmp(A, 4096);
        e.0.jump_if(0x82, "refuse");
        e.cmp(A, (storage_bytes - 64) as i32);
        e.0.jump_if(0x87, "refuse");
        e.add(A, 1);
        e.rr(0x39, A, R14);
        e.0.jump_if(0x85, "refuse");
    }
    zero(&mut code, RECOVERY, 32, "refuse");
    zero(&mut code, ACTIVE, 8, "refuse");
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, B, RECOVERY);
        e.store(B, 0, R12);
        e.imm(A, 1);
        e.store(B, 8, A);
        address(&mut e, &mut code.references, R11, HISTORY);
        e.load(A, R11, 16);
        e.store(B, 16, A);
        e.store(B, 24, R14);
        e.mov(_Sp, R12);
        e.add(_Sp, -32);
        e.imm(A, 64);
        e.store(_Sp, 0, A);
        e.imm(A, 0);
        e.store(_Sp, 8, A);
        e.imm(A, 1);
        e.store(_Sp, 16, A);
        e.imm(A, 0);
        e.store(_Sp, 24, A);
        e.mov(D, _Sp);
        e.mov(Di, R13);
        e.imm(Si, 64);
        address(&mut e, &mut code.references, C, HISTORY);
        address(&mut e, &mut code.references, R8, PRIVATE);
        address(&mut e, &mut code.references, R9, STACK);
        e.mov(_Sp, R12);
        e.add(_Sp, -64);
        e.arith(4, _Sp, -16);
        e.add(_Sp, -16);
        address(&mut e, &mut code.references, R11, OUTPUT);
        e.store(_Sp, 0, R11);
        branch(&mut e, &mut code.references, OWNER, true);
        e.0.bind("owner_returned")?;
        e.mov(_Sp, R12);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
        address(&mut e, &mut code.references, R11, OUTPUT);
        e.load(A, R11, 0);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "fail");
        e.load(A, R11, 48);
        e.cmp(A, 3);
        e.0.jump_if(0x85, "fail");
        address(&mut e, &mut code.references, R11, HISTORY);
        e.load(A, R11, 24);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
        e.load(A, R11, 32);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
    }
    for (symbol, bytes) in [(PRIVATE, PRIVATE_BYTES), (STACK, STACK_BYTES), (ACTIVE, 8)] {
        zero(&mut code, symbol, bytes, "fail");
    }
    wipe(&mut code, OUTPUT, 56, "fail");
    erase_caller(&mut code, retired, "fail");
    saved_rows(&mut code, 7);
    pop_caller(&mut code);
    spill_and_recovery(&mut code, "late_failure");
    let normal = code.encoder.bytes.len() as u64;
    code.encoder.bytes(&[0xc3]);
    late_failure(&mut code)?;
    code.encoder.bind("fail")?;
    saved_rows(&mut code, 12);
    branch(
        &mut E(&mut code.encoder),
        &mut code.references,
        FATAL,
        false,
    );
    code.encoder.bind("refuse")?;
    saved_rows(&mut code, 12);
    E(&mut code.encoder).mov(_Sp, R12);
    saved_rows(&mut code, 7);
    pop_caller(&mut code);
    // Invalid entry provides no invocation-cleanup authority and cannot resume.
    row(&mut code, vec![0x07, 16]);
    halt(&mut code, "unsafe")?;
    Ok((code, normal))
}

fn fatal(retired: u64) -> Result<(Code, u64), CompileError> {
    let mut code = Code::default();
    row(&mut code, vec![0x07, 16]);
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R11, RECOVERY);
        e.load(A, R11, 8);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "unsafe");
        e.load(R12, R11, 0);
        e.cmp(R12, 0x108fd0); // Lowest admitted entry SP minus six saved words.
        e.0.jump_if(0x82, "unsafe");
        e.cmp(R12, 0x17ffc8);
        e.0.jump_if(0x87, "unsafe");
        e.mov(A, R12);
        e.arith(4, A, 7);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "unsafe");
        e.mov(_Sp, R12);
    }
    saved_rows(&mut code, 12);
    row(&mut code, vec![0x90, 1]); // Recovery now exposes the actual outer return word.
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R11, HISTORY);
        e.load(A, R11, 16);
        address(&mut e, &mut code.references, R10, RECOVERY);
        e.load(C, R10, 16);
        e.rr(0x39, A, C);
        e.0.jump_if(0x84, "history_ready");
        e.imm(A, 0);
        e.store(R11, 24, A);
        e.imm(A, 1);
        e.store(R11, 32, A);
        e.0.bind("history_ready")?;
    }
    for (index, (symbol, bytes)) in [
        (OUTPUT, 56),
        (STACK, STACK_BYTES),
        (PRIVATE, PRIVATE_BYTES),
        (ACTIVE, 8),
    ]
    .into_iter()
    .enumerate()
    {
        let next = format!("retired_{index}");
        wipe(&mut code, symbol, bytes, &next);
        code.encoder.bind(&next)?;
    }
    erase_caller(&mut code, retired, "caller_retired");
    code.encoder.bind("caller_retired")?;
    saved_rows(&mut code, 7);
    pop_caller(&mut code);
    spill_and_recovery(&mut code, "late_failure");
    let marker = code.encoder.bytes.len() as u64;
    halt(&mut code, "unsafe")?;
    late_failure(&mut code)?;
    Ok((code, marker))
}
fn finalize() -> Result<Code, CompileError> {
    let mut code = Code::default();
    for (symbol, bytes) in [
        (PRIVATE, PRIVATE_BYTES),
        (STACK, STACK_BYTES),
        (OUTPUT, 56),
        (ACTIVE, 8),
        (RECOVERY, 32),
    ] {
        zero(&mut code, symbol, bytes, "unsafe");
    }
    {
        let mut e = E(&mut code.encoder);
        address(&mut e, &mut code.references, R11, HISTORY);
        e.load(A, R11, 16);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "unsafe");
        for offset in [24, 32] {
            e.load(A, R11, offset);
            e.cmp(A, 0);
            e.0.jump_if(0x85, "unsafe");
        }
    }
    wipe(&mut code, HISTORY, 40, "unsafe");
    clear_volatile(&mut E(&mut code.encoder));
    code.encoder.bytes(&[0xc3]);
    halt(&mut code, "unsafe")?;
    Ok(code)
}

pub(crate) fn component_object(
    storage_bytes: u64,
    owner_stack: u64,
    combined: u64,
) -> Result<Vec<u8>, CompileError> {
    if !(4160..=i32::MAX as u64).contains(&storage_bytes)
        || owner_stack < 8
        || owner_stack > combined
        || owner_stack % 8 != 0
        || combined % 8 != 0
        || combined > 4096 - 8 - 48 - FRAME_BYTES
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-TAIL-KERNEL: incomplete caller/storage bounds".into(),
        ));
    }
    let retired = (FRAME_BYTES + owner_stack).div_ceil(8) * 8;
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.topal.tail.kernel".to_vec(),
        SectionKind::Text,
    );
    let frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    object.add_section(Vec::new(), b".note.GNU-stack".to_vec(), SectionKind::Other);
    for (name, bytes, alignment) in [
        (PRIVATE, PRIVATE_BYTES, 16),
        (STACK, STACK_BYTES, 4096),
        (HISTORY, 40, 8),
        (OUTPUT, 56, 8),
        (RECOVERY, 32, 8),
    ] {
        bss(&mut object, name, bytes, alignment);
    }
    let (code, normal) = apply(storage_bytes, retired)?;
    let points = [
        (
            "topal_x86_tail_kernel_owner_returned_v1",
            code.encoder.labels["owner_returned"] as u64,
        ),
        (
            "topal_x86_tail_kernel_refused_v1",
            code.encoder.labels["unsafe"] as u64 + 1,
        ),
    ];
    append_code(&mut object, text, frame, APPLY, code)?;
    let start = object
        .symbol(object.symbol_id(APPLY.as_bytes()).unwrap())
        .value;
    marker(
        &mut object,
        text,
        "topal_x86_tail_kernel_return_reclaimed_v1",
        start + normal,
    );
    for (name, offset) in points {
        marker(&mut object, text, name, start + offset);
    }
    let (code, point) = fatal(retired)?;
    append_code(&mut object, text, frame, FATAL, code)?;
    let start = object
        .symbol(object.symbol_id(FATAL.as_bytes()).unwrap())
        .value;
    marker(
        &mut object,
        text,
        "topal_x86_tail_kernel_fatal_reclaimed_v1",
        start + point,
    );
    append_code(&mut object, text, frame, FINALIZE, finalize()?)?;
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
