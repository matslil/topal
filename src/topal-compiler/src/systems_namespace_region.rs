//! TK-DEC-281: actual namespace-page loan, comparison-only oracle, fatal cleanup.
//! TOPAL-SYSTEMS-NAMESPACE-ACTION-001. Recoverable personality admission precedes
//! this exclusive controller; copied facts/addresses never confer authority.
use super::CompileError;
use super::namespace_oracle as oracle;
use super::root_region::{Code, address, branch, clear_temporaries, erase, zero_readback};
use super::x86_function::{E, R, R::*};
use crate::systems_region_library::{
    NAMESPACE_FRAME_ABORT, NAMESPACE_FRAME_ALLOCATE, NAMESPACE_FRAME_EXPORT,
};
use object::write::{Object, SectionId, Symbol, SymbolSection};
use object::{SectionKind, SymbolFlags, SymbolKind, SymbolScope};

#[path = "systems_namespace_region_inspection.rs"]
mod inspection;
#[path = "systems_namespace_region_abort.rs"]
mod recovery;
pub use inspection::SystemsNamespaceRegionBinding;
pub(crate) use inspection::inspect_linked;

pub(crate) const APPLY: &str = "topal_x86_systems_namespace_apply_v1";
pub(crate) const FINALIZE: &str = "topal_x86_systems_namespace_finalize_v1";
pub(crate) const HALT: &str = "topal_x86_systems_namespace_halt_v1";
pub(crate) const ARENA: &str = "topal_x86_systems_namespace_arena";
pub(crate) const CONTROL: &str = "topal_x86_systems_namespace_control";
pub(crate) const HISTORY: &str = "topal_x86_systems_namespace_history";
pub(crate) const RECOVERY: &str = "topal_x86_systems_namespace_recovery";
pub(super) const ARENA_BYTES: u64 = 65536;
pub(super) const RETIRED_STACK_BYTES: u64 = 4096;
pub(super) const LIVE_FRAME_BYTES: u64 = 8 + 14 * 8 + 64;
pub(super) const CALL_ALIGNMENT_BYTES: u64 = 64;
pub(super) const RECOVERY_BYTES: u64 = 32;
pub(super) const PRIVATE_BSS_MINIMUM: u64 = 0x800000;
pub(super) const METADATA: i32 = super::process_runtime::volatile_files::BASE;
pub(super) const FRAME_DELTA: i32 = 14 * 8 + 64;
pub(super) const STACK_ENTRY_MAXIMUM: i32 = 0x180000 - 8;
const SAVES: [(R, u8); 13] = [
    (B, 3),
    (R12, 12),
    (R13, 13),
    (R14, 14),
    (R15, 15),
    (Di, 5),
    (Si, 4),
    (D, 1),
    (C, 2),
    (R8, 8),
    (R9, 9),
    (R10, 10),
    (R11, 11),
];

fn call(e: &mut E<'_>, references: &mut Vec<(u64, &'static str, bool)>, target: &'static str) {
    e.arith(4, _Sp, -16);
    e.add(_Sp, -16);
    branch(e, references, target, true);
    e.mov(_Sp, R12);
}

/// DI actual controller, SI actual metadata page, D admitted operation,
/// C admitted realtime seconds. Preserve all GPRs except A, and all SIMD.
/// The source alone performs persistent namespace stores; CHECK erases staging.
pub(super) fn apply() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.cmp(_Sp, 0x108000);
    e.0.jump_if(0x82, "fail");
    e.cmp(_Sp, STACK_ENTRY_MAXIMUM);
    e.0.jump_if(0x87, "fail");
    e.mov(A, _Sp);
    e.arith(4, A, 7);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.0.bytes(&[0x9c]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0e, 16]));
    e.0.bytes(&[0x58]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0e, 8]));
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.0.bytes(&[0x55]);
    code.unwind
        .push((e.0.bytes.len() as u32, vec![0x0e, 16, 0x86, 2]));
    e.0.bytes(&[0x48, 0x89, 0xe5]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0d, 6]));
    for (index, (register, dwarf)) in SAVES.iter().enumerate() {
        let n = *register as u8;
        if n >= 8 {
            e.0.bytes(&[0x41]);
        }
        e.0.bytes(&[0x50 | (n & 7)]);
        code.unwind
            .push((e.0.bytes.len() as u32, vec![0x80 | dwarf, index as u8 + 3]));
    }
    e.add(_Sp, -64);
    e.mov(R12, _Sp);
    e.mov(R13, Di);
    e.mov(R14, Si);
    e.imm(A, 0);
    for offset in (0..64).step_by(8) {
        e.store(R12, offset, A);
    }
    e.store(R12, 0, D);
    e.store(R12, 8, C);
    address(&mut e, &mut code.references, R11, RECOVERY);
    zero_readback(&mut e, R11, RECOVERY_BYTES);
    // Trusted kernel frame facts are outside every source view. An aborting
    // callee need not restore R12 or any other callee-saved register first.
    e.store(R11, 0, R12);
    e.load(A, R12, 0);
    e.store(R11, 16, A);
    e.load(A, R12, 8);
    e.store(R11, 24, A);
    e.gs(
        0x8b,
        R11,
        super::process_runtime::file_descriptions::TABLE_CONTEXT,
    );
    e.rr(0x39, R13, R11);
    e.0.jump_if(0x85, "fail");
    address(
        &mut e,
        &mut code.references,
        R11,
        super::X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL,
    );
    e.rr(0x39, R13, R11);
    e.0.jump_if(0x85, "fail");
    e.load(A, R13, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    e.mov(A, R13);
    e.add(A, METADATA);
    e.rr(0x39, R14, A);
    e.0.jump_if(0x85, "fail");
    for control in [
        super::file_region::CONTROL,
        super::ram_page_region::CONTROL,
        super::inode_region::CONTROL,
    ] {
        address(&mut e, &mut code.references, R11, control);
        zero_readback(&mut e, R11, 64);
    }
    address(&mut e, &mut code.references, B, CONTROL);
    zero_readback(&mut e, B, 64);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(R15, R11, 0);
    e.add(R15, 1);
    e.0.jump_if(0x82, "fail");
    e.mov(Di, R14);
    e.load(Si, R12, 0);
    e.load(D, R12, 8);
    call(&mut e, &mut code.references, oracle::PREPARE);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(A, R11, 0);
    e.store(B, 16, A);
    // Consume the epoch before entering source, so even a runtime abort cannot
    // rewind a namespace grant. Only final terminal retirement clears history.
    e.store(R11, 0, R15);
    address(&mut e, &mut code.references, R11, RECOVERY);
    e.store(R11, 8, R15);
    address(&mut e, &mut code.references, A, NAMESPACE_FRAME_EXPORT);
    e.store(B, 0, A);
    e.store(B, 8, R13);
    e.store(B, 32, R14);
    e.imm(A, 4096);
    e.store(B, 40, A);
    e.imm(A, 1);
    e.store(B, 48, A);
    address(&mut e, &mut code.references, R11, ARENA);
    e.store(R11, 0, B);
    e.imm(A, 32);
    e.store(R11, 8, A);
    e.store(R11, 16, R15);
    e.mov(Di, B);
    e.mov(Si, R13);
    e.mov(D, R15);
    e.mov(C, R12);
    e.imm(R8, 2);
    call(&mut e, &mut code.references, NAMESPACE_FRAME_EXPORT);
    e.store(R12, 16, A);
    for offset in [0, 8, 24, 32, 40, 48, 56] {
        e.load(A, B, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
    }
    e.load(A, B, 16);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(A, R11, 0);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R11, RECOVERY);
    for (offset, expected) in [(0, R12), (8, R15)] {
        e.load(A, R11, offset);
        e.rr(0x39, A, expected);
        e.0.jump_if(0x85, "fail");
    }
    for offset in [0, 8] {
        e.load(A, R11, 16 + offset);
        e.mem(0x3b, A, R12, offset);
        e.0.jump_if(0x85, "fail");
    }
    for offset in (24..64).step_by(8) {
        e.load(A, R12, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
    }
    address(&mut e, &mut code.references, R11, ARENA);
    e.load(A, R11, 0);
    e.rr(0x39, A, B);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 16);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    e.mov(Di, R14);
    e.load(Si, R12, 16);
    call(&mut e, &mut code.references, oracle::CHECK);
    e.mov(R15, A);
    address(&mut e, &mut code.references, R11, ARENA);
    erase(&mut e, R11, ARENA_BYTES);
    erase(&mut e, B, 64);
    // Erase AFTER the final helper call, including its return address. There
    // are no calls or pushes between this readback and owner resumption.
    e.mov(R11, R12);
    e.add(R11, -(RETIRED_STACK_BYTES as i32));
    erase(&mut e, R11, RETIRED_STACK_BYTES + 64);
    e.cmp(R15, 1);
    e.0.jump_if(0x85, "fail");
    clear_temporaries(&mut e);
    e.imm(A, 1);
    e.add(_Sp, 64);
    for (register, dwarf) in SAVES.iter().rev() {
        let n = *register as u8;
        if n >= 8 {
            e.0.bytes(&[0x41]);
        }
        e.0.bytes(&[0x58 | (n & 7)]);
        code.unwind
            .push((e.0.bytes.len() as u32, vec![0x08, *dwarf]));
    }
    e.0.bytes(&[0x5d]);
    code.unwind
        .push((e.0.bytes.len() as u32, vec![0x0c, 7, 8, 0x08, 6]));
    // The restored registers hold the caller's values. Erase their popped
    // spill slots as well as the scalar frame, using only the result register.
    // The still-live return address remains the caller's ABI-owned word.
    e.imm(A, 0);
    for offset in (-FRAME_DELTA..0).step_by(8) {
        e.store(_Sp, offset, A);
    }
    for offset in (-FRAME_DELTA..0).step_by(8) {
        e.load(A, _Sp, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
    }
    address(&mut e, &mut code.references, A, RECOVERY);
    for offset in (0..RECOVERY_BYTES as i32).step_by(8) {
        e.mem(0xc7, R::A, R::A, offset); // MOV qword [RAX+offset], imm32(0)
        e.0.bytes(&[0; 4]);
    }
    for offset in (0..RECOVERY_BYTES as i32).step_by(8) {
        e.mem(0x81, Di, R::A, offset); // CMP qword [RAX+offset], imm32(0), /7
        e.0.bytes(&[0; 4]);
        e.0.jump_if(0x85, "fail");
    }
    e.imm(A, 1);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    code.unwind.push((e.0.bytes.len() as u32, vec![0x07, 16]));
    branch(&mut e, &mut code.references, NAMESPACE_FRAME_ABORT, false);
    Ok(code)
}

pub(super) fn allocator() -> Result<Code, CompileError> {
    super::root_region::allocator_for(
        CONTROL,
        ARENA,
        NAMESPACE_FRAME_EXPORT,
        NAMESPACE_FRAME_ABORT,
        true,
    )
}

pub(super) fn finalize() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    for (symbol, count) in [
        (ARENA, ARENA_BYTES),
        (CONTROL, 64),
        (oracle::EXPECTED, oracle::EXPECTED_BYTES),
        (RECOVERY, RECOVERY_BYTES),
    ] {
        address(&mut e, &mut code.references, R11, symbol);
        zero_readback(&mut e, R11, count);
    }
    address(&mut e, &mut code.references, R11, HISTORY);
    erase(&mut e, R11, 8);
    clear_temporaries(&mut e);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    branch(&mut e, &mut code.references, NAMESPACE_FRAME_ABORT, false);
    Ok(code)
}

pub(super) fn templates() -> Result<Vec<(&'static str, Code)>, CompileError> {
    Ok(vec![
        (APPLY, apply()?),
        (NAMESPACE_FRAME_ALLOCATE, allocator()?),
        (NAMESPACE_FRAME_ABORT, recovery::abort()?),
        (FINALIZE, finalize()?),
        (HALT, super::file_region::fatal_template()),
        (oracle::PREPARE, oracle::prepare()?),
        (oracle::CHECK, oracle::check()?),
    ])
}

pub(super) fn append(object: &mut Object<'_>, text: SectionId) -> Result<(), CompileError> {
    for (name, bytes, alignment) in [
        (ARENA, ARENA_BYTES, 4096),
        (CONTROL, 64, 8),
        (HISTORY, 8, 8),
        (oracle::EXPECTED, oracle::EXPECTED_BYTES, 4096),
        (RECOVERY, RECOVERY_BYTES, 8),
    ] {
        let section = object.add_section(
            Vec::new(),
            format!(".bss.{name}").into_bytes(),
            SectionKind::UninitializedData,
        );
        let offset = object.append_section_bss(section, bytes, alignment);
        object.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: offset,
            size: bytes,
            kind: SymbolKind::Data,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(section),
            flags: SymbolFlags::None,
        });
    }
    let debug = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    for (name, code) in templates()? {
        super::root_region::append_code(object, text, debug, name, code)?;
    }
    Ok(())
}
