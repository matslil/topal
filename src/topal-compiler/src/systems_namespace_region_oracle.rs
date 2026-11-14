//! TK281 comparison-only namespace snapshot and completion cleanup.
//! TOPAL-SYSTEMS-NAMESPACE-ACTION-001. These helpers do not issue ownership,
//! invoke source, retire grants or implement the owning controller's fatal path.
use super::CompileError;
use super::root_region::{Code, address};
use super::x86_function::{E, R, R::*};

pub(crate) const PREPARE: &str = "topal_x86_systems_namespace_prepare_v1";
pub(crate) const CHECK: &str = "topal_x86_systems_namespace_check_v1";
pub(crate) const EXPECTED: &str = "topal_x86_systems_namespace_expected";
pub(crate) const EXPECTED_BYTES: u64 = 4096;
// Leaf helpers have no pushes, local stack, calls, SIMD or implicit red zone.
pub(crate) const HELPER_STACK_BYTES: u64 = 8;

fn shift(e: &mut E<'_>, register: R, count: u8) {
    let n = register as u8;
    e.0.bytes(&[0x48 | (n >> 3), 0xc1, 0xe0 | (n & 7), count]);
}
fn pointer(e: &mut E<'_>, out: R, base: R, field: i32, scale: u8, bias: i32) {
    e.load(out, R8, field);
    shift(e, out, scale);
    e.rr(0x01, out, base);
    e.add(out, bias);
}
fn equal(e: &mut E<'_>, left: R, a: i32, right: R, b: i32) {
    e.load(A, left, a);
    e.mem(0x3b, A, right, b);
    e.0.jump_if(0x85, "refuse");
}
fn zero(e: &mut E<'_>, offset: i32, bytes: i32) -> Result<(), CompileError> {
    let label = format!("zero-{offset}");
    e.mov(Si, R8);
    e.add(Si, offset);
    e.imm(C, (bytes / 8) as u64);
    e.0.bind(&label)?;
    e.load(A, Si, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "refuse");
    e.add(Si, 8);
    e.add(C, -1);
    e.0.jump_if(0x85, &label);
    Ok(())
}
/// D is length; SI points at a zero-padded 256-byte raw component.
fn component(e: &mut E<'_>, tag: &str) -> Result<(), CompileError> {
    e.cmp(D, 1);
    e.0.jump_if(0x82, "refuse");
    e.cmp(D, 255);
    e.0.jump_if(0x87, "refuse");
    e.cmp(D, 2);
    e.0.jump_if(0x87, &format!("component-loop-{tag}"));
    e.load(A, Si, 0);
    e.arith(4, A, 255);
    e.cmp(A, 46);
    e.0.jump_if(0x85, &format!("component-loop-{tag}"));
    e.cmp(D, 1);
    e.0.jump_if(0x84, "refuse");
    e.load(A, Si, 1);
    e.arith(4, A, 255);
    e.cmp(A, 46);
    e.0.jump_if(0x84, "refuse");
    e.0.bind(&format!("component-loop-{tag}"))?;
    e.imm(C, 0);
    e.0.bind(&format!("component-byte-{tag}"))?;
    e.load(A, Si, 0); // Last read remains inside the admitted metadata page.
    e.arith(4, A, 255);
    e.rr(0x39, C, D);
    e.0.jump_if(0x83, &format!("component-padding-{tag}"));
    e.cmp(A, 0);
    e.0.jump_if(0x84, "refuse");
    e.cmp(A, 47);
    e.0.jump_if(0x84, "refuse");
    e.0.jump(&format!("component-next-{tag}"));
    e.0.bind(&format!("component-padding-{tag}"))?;
    e.cmp(A, 0);
    e.0.jump_if(0x85, "refuse");
    e.0.bind(&format!("component-next-{tag}"))?;
    e.add(Si, 1);
    e.add(C, 1);
    e.cmp(C, 256);
    e.0.jump_if(0x85, &format!("component-byte-{tag}"));
    Ok(())
}

fn objects(e: &mut E<'_>) -> Result<(), CompileError> {
    for slot in 0..4 {
        let base = 256 + slot * 128;
        e.load(A, R8, base);
        e.cmp(A, 0);
        e.0.jump_if(0x85, &format!("object-live-{slot}"));
        zero(e, base, 128)?;
        e.0.jump(&format!("object-next-{slot}"));
        e.0.bind(&format!("object-live-{slot}"))?;
        e.cmp(A, 1);
        e.0.jump_if(0x85, "refuse");
        e.load(D, R8, base + 8);
        e.cmp(D, 7);
        e.0.jump_if(0x82, "refuse");
        e.mem(0x3b, D, R8, 0);
        e.0.jump_if(0x83, "refuse");
        for previous in 0..slot {
            e.mem(0x3b, D, R8, 256 + previous * 128 + 8);
            e.0.jump_if(0x84, "refuse");
        }
        e.load(A, R8, base + 16);
        e.imm(C, i64::MAX as u64);
        e.rr(0x39, A, C);
        e.0.jump_if(0x87, "refuse");
        e.load(A, R8, base + 24);
        e.arith(4, A, !0o7777);
        e.cmp(A, 0o100000);
        e.0.jump_if(0x85, "refuse");
        e.load(A, R8, base + 64);
        e.cmp(A, 1);
        e.0.jump_if(0x87, "refuse");
        e.imm(C, 0);
        for entry in 0..4 {
            e.mem(0x3b, D, R8, 64 + entry * 16);
            e.0.jump_if(0x85, &format!("object-count-{slot}-{entry}"));
            e.add(C, 1);
            e.0.bind(&format!("object-count-{slot}-{entry}"))?;
        }
        e.mem(0x3b, C, R8, base + 56);
        e.0.jump_if(0x85, "refuse");
        e.cmp(C, 0);
        e.0.jump_if(0x84, &format!("object-intent-{slot}"));
        e.cmp(A, 0);
        e.0.jump_if(0x85, "refuse");
        e.0.bind(&format!("object-intent-{slot}"))?;
        zero(e, base + 72, 56)?;
        e.0.bind(&format!("object-next-{slot}"))?;
    }
    Ok(())
}
fn identity_exists(e: &mut E<'_>, register: R, label: &str) -> Result<(), CompileError> {
    for slot in 0..4 {
        e.mem(0x3b, register, R8, 264 + slot * 128);
        e.0.jump_if(0x84, label);
    }
    e.0.jump("refuse");
    e.0.bind(label)?;
    Ok(())
}
fn names(e: &mut E<'_>) -> Result<(), CompileError> {
    for slot in 0..4 {
        let name = 1536 + slot * 256;
        e.load(D, R8, 64 + slot * 16);
        e.cmp(D, 0);
        e.0.jump_if(0x85, &format!("entry-live-{slot}"));
        zero(e, 64 + slot * 16, 16)?;
        zero(e, name, 256)?;
        e.0.jump(&format!("entry-next-{slot}"));
        e.0.bind(&format!("entry-live-{slot}"))?;
        identity_exists(e, D, &format!("entry-owner-{slot}"))?;
        e.load(D, R8, 72 + slot * 16);
        e.mov(Si, R8);
        e.add(Si, name);
        component(e, &format!("entry-{slot}"))?;
        for previous in 0..slot {
            e.mov(Si, R8);
            e.add(Si, name);
            e.mov(Di, R8);
            e.add(Di, 1536 + previous * 256);
            e.imm(C, 32);
            e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa7]);
            e.0.jump_if(0x84, "refuse");
        }
        e.0.bind(&format!("entry-next-{slot}"))?;
    }
    Ok(())
}
fn pages(e: &mut E<'_>) -> Result<(), CompileError> {
    for slot in 0..8 {
        let base = 1024 + slot * 32;
        e.load(D, R8, base);
        e.cmp(D, 0);
        e.0.jump_if(0x85, &format!("page-live-{slot}"));
        zero(e, base, 32)?;
        e.mov(Si, R8);
        e.add(Si, 24 + slot);
        e.load(A, Si, 0);
        e.arith(4, A, 255);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "refuse");
        e.0.jump(&format!("page-next-{slot}"));
        e.0.bind(&format!("page-live-{slot}"))?;
        identity_exists(e, D, &format!("page-owner-{slot}"))?;
        e.load(A, R8, base + 8);
        e.imm(C, (i64::MAX as u64) / 4096);
        e.rr(0x39, A, C);
        e.0.jump_if(0x87, "refuse");
        for previous in 0..slot {
            let next = format!("page-distinct-{slot}-{previous}");
            e.mem(0x3b, D, R8, 1024 + previous * 32);
            e.0.jump_if(0x85, &next);
            e.mem(0x3b, A, R8, 1032 + previous * 32);
            e.0.jump_if(0x84, "refuse");
            e.0.bind(&next)?;
        }
        zero(e, base + 16, 16)?;
        e.load(A, R8, 24 + slot);
        e.arith(4, A, 255);
        e.cmp(A, 1);
        e.0.jump_if(0x85, "refuse");
        e.0.bind(&format!("page-next-{slot}"))?;
    }
    Ok(())
}

fn inputs(e: &mut E<'_>) -> Result<(), CompileError> {
    for (offset, maximum) in [(3072, 3), (3080, 4), (3096, 3)] {
        e.load(A, R8, offset);
        e.cmp(A, maximum);
        e.0.jump_if(0x87, "refuse");
    }
    pointer(e, D, R8, 3072, 7, 256);
    equal(e, D, 8, R8, 3112);
    equal(e, D, 56, R8, 3128);
    e.load(A, D, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "refuse");
    e.load(A, D, 56);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "input-linked");
    e.cmp(R10, 0);
    e.0.jump_if(0x85, "refuse");
    e.load(A, D, 64);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "refuse");
    e.load(A, R8, 3080);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "refuse");
    e.0.bind("input-linked")?;
    e.load(A, R8, 3080);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "input-entry");
    e.cmp(R10, 0);
    e.0.jump_if(0x85, "refuse");
    e.0.jump("input-entry-ready");
    e.0.bind("input-entry")?;
    pointer(e, D, R8, 3080, 4, 64);
    equal(e, D, 0, R8, 3112);
    e.0.bind("input-entry-ready")?;
    e.load(D, R8, 3104);
    e.mov(Si, R8);
    e.add(Si, 3200);
    component(e, "input")?;
    zero(e, 3136, 64)?;
    zero(e, 3456, 128)?;
    e.cmp(R10, 2);
    e.0.jump_if(0x83, "input-target-live");
    e.load(A, R8, 3088);
    e.cmp(A, 4);
    e.0.jump_if(0x85, "refuse");
    e.load(A, R8, 3120);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "refuse");
    for slot in 0..4 {
        e.mov(Si, R8);
        e.add(Si, 3200);
        e.mov(Di, R8);
        e.add(Di, 1536 + slot * 256);
        e.imm(C, 32);
        e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa7]);
        e.0.jump_if(0x84, "refuse");
    }
    e.cmp(R10, 1);
    e.0.jump_if(0x84, "input-move");
    pointer(e, D, R8, 3096, 4, 64);
    e.load(A, D, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "refuse");
    e.0.jump("input-ready");
    e.0.bind("input-move")?;
    equal(e, R8, 3080, R8, 3096);
    e.0.jump("input-ready");
    e.0.bind("input-target-live")?;
    e.load(A, R8, 3088);
    e.cmp(A, 3);
    e.0.jump_if(0x87, "refuse");
    e.mem(0x3b, A, R8, 3072);
    e.0.jump_if(0x84, "refuse");
    pointer(e, D, R8, 3088, 7, 256);
    e.load(A, D, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "refuse");
    equal(e, D, 8, R8, 3120);
    pointer(e, D, R8, 3096, 4, 64);
    equal(e, D, 0, R8, 3120);
    equal(e, D, 8, R8, 3104);
    pointer(e, Di, R8, 3096, 8, 1536);
    e.mov(Si, R8);
    e.add(Si, 3200);
    e.imm(C, 32);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa7]);
    e.0.jump_if(0x85, "refuse");
    e.0.bind("input-ready")?;
    Ok(())
}

fn metadata(e: &mut E<'_>) -> Result<(), CompileError> {
    e.load(A, R8, 0);
    e.cmp(A, 7);
    e.0.jump_if(0x82, "refuse");
    e.load(A, R8, 32);
    e.arith(4, A, !0o7777);
    e.cmp(A, 0o40000);
    e.0.jump_if(0x85, "refuse");
    for (start, bytes) in [
        (8, 16),
        (128, 128),
        (768, 256),
        (1280, 256),
        (2560, 512),
        (3584, 512),
    ] {
        zero(e, start, bytes)?;
    }
    objects(e)?;
    names(e)?;
    pages(e)?;
    Ok(())
}

/// Read-only byte consistency, not an owner/loan/credential grant. The directory
/// staging is private to the current exclusive operation and may contain its
/// pre-admission facts. The action's separate input checks still apply at entry.
/// DI metadata, A Boolean; preserve all other GPRs and SIMD. Leaf stack: 72.
pub(super) fn metadata_validator() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    let saved = [
        (C, 2),
        (D, 1),
        (Si, 4),
        (Di, 5),
        (R8, 8),
        (R9, 9),
        (R10, 10),
        (R11, 11),
    ];
    for (index, (register, dwarf)) in saved.iter().enumerate() {
        let n = *register as u8;
        if n >= 8 {
            e.0.bytes(&[0x41]);
        }
        e.0.bytes(&[0x50 | (n & 7)]);
        code.unwind.push((
            e.0.bytes.len() as u32,
            vec![0x0e, 16 + index as u8 * 8, 0x80 | dwarf, index as u8 + 2],
        ));
    }
    e.mov(R8, Di);
    metadata(&mut e)?;
    e.imm(A, 1);
    e.0.jump("metadata-return");
    e.0.bind("refuse")?;
    e.imm(A, 0);
    e.0.bind("metadata-return")?;
    for (index, (register, dwarf)) in saved.iter().rev().enumerate() {
        let n = *register as u8;
        if n >= 8 {
            e.0.bytes(&[0x41]);
        }
        e.0.bytes(&[0x58 | (n & 7)]);
        code.unwind.push((
            e.0.bytes.len() as u32,
            vec![0x0e, 64 - index as u8 * 8, 0x08, *dwarf],
        ));
    }
    // Preserve the Boolean and restored caller registers while retiring every
    // popped spill. The caller still owns the live return-address word.
    for offset in (-64..0).step_by(8) {
        e.mem(0xc7, A, R::_Sp, offset);
        e.0.bytes(&[0; 4]);
    }
    for offset in (-64..0).step_by(8) {
        e.mem(0x81, Di, R::_Sp, offset);
        e.0.bytes(&[0; 4]);
        e.0.jump_if(0x85, "metadata-erasure-failed");
    }
    e.0.bytes(&[0xc3]);
    e.0.bind("metadata-erasure-failed")?;
    e.imm(A, 0);
    e.0.bytes(&[0xc3]);
    Ok(code)
}

/// DI readable admitted metadata page, SI operation, D clock. Returns Boolean;
/// prepares only the private EXPECTED page, never writes to owner storage.
pub(super) fn prepare() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.mov(R8, Di);
    e.mov(R10, Si);
    e.mov(R11, D);
    address(&mut e, &mut code.references, R9, EXPECTED);
    e.mov(Di, R9);
    e.imm(A, 0);
    e.imm(C, 512);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "refuse"); // A nested preparation cannot erase a live oracle.
    e.cmp(R10, 3);
    e.0.jump_if(0x87, "refuse");
    metadata(&mut e)?;
    inputs(&mut e)?;
    e.mov(Si, R8);
    e.mov(Di, R9);
    e.imm(C, 512);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa5]);
    e.cmp(R10, 2);
    e.0.jump_if(0x83, "effect-live");
    pointer(&mut e, Di, R9, 3096, 8, 1536);
    e.mov(Si, R8);
    e.add(Si, 3200);
    e.imm(C, 32);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa5]);
    pointer(&mut e, Di, R9, 3096, 4, 64);
    e.load(A, R8, 3104);
    e.store(Di, 8, A);
    e.cmp(R10, 1);
    e.0.jump_if(0x84, "effect-times");
    e.load(A, R8, 3112);
    e.store(Di, 0, A);
    pointer(&mut e, D, R9, 3072, 7, 256);
    e.load(A, R8, 3128);
    e.add(A, 1);
    e.store(D, 56, A);
    e.imm(A, 0);
    e.store(D, 64, A);
    e.0.jump("effect-times");
    e.0.bind("effect-live")?;
    pointer(&mut e, Di, R9, 3096, 4, 64);
    e.load(A, R8, 3112);
    e.store(Di, 0, A);
    pointer(&mut e, Di, R9, 3080, 4, 64);
    e.cmp(R10, 3);
    e.0.jump_if(0x84, "effect-exchange");
    e.imm(A, 0);
    e.store(Di, 0, A);
    e.store(Di, 8, A);
    pointer(&mut e, Di, R9, 3080, 8, 1536);
    e.imm(C, 32);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    pointer(&mut e, D, R9, 3088, 7, 256);
    e.load(A, D, 56);
    e.add(A, -1);
    e.store(D, 56, A);
    e.0.jump("effect-victim-time");
    e.0.bind("effect-exchange")?;
    e.load(A, R8, 3120);
    e.store(Di, 0, A);
    pointer(&mut e, D, R9, 3088, 7, 256);
    e.0.bind("effect-victim-time")?;
    e.store(D, 48, R11);
    e.0.bind("effect-times")?;
    pointer(&mut e, D, R9, 3072, 7, 256);
    e.store(D, 48, R11);
    e.store(R9, 48, R11);
    e.store(R9, 56, R11);
    e.imm(A, 1);
    e.0.bytes(&[0xc3]);
    e.0.bind("refuse")?;
    e.imm(A, 0);
    e.0.bytes(&[0xc3]);
    Ok(code)
}

/// DI actual loaned page, SI exact source result. Comparison consumes the private
/// oracle and erases actual staging on BOTH success and failure. The owning
/// controller must additionally retire the grant, erase all source temporary
/// storage, and halt on false; calling this helper alone proves no authority.
pub(super) fn check() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.mov(R8, Di);
    e.mov(R10, Si);
    address(&mut e, &mut code.references, R9, EXPECTED);
    e.cmp(R10, 1);
    e.0.jump_if(0x85, "invalid");
    e.load(A, R9, 0);
    e.cmp(A, 7);
    e.0.jump_if(0x82, "invalid");
    e.mov(Si, R9);
    e.imm(C, 512);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xa7]);
    e.0.jump_if(0x84, "cleanup");
    e.0.bind("invalid")?;
    e.imm(R10, 0);
    e.0.bind("cleanup")?;
    for (base, offset, words) in [(R8, 3072, 64), (R9, 0, 512)] {
        e.mov(Di, base);
        e.add(Di, offset);
        e.imm(A, 0);
        e.imm(C, words);
        e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    }
    for (base, offset, words) in [(R8, 3072, 64), (R9, 0, 512)] {
        e.mov(Di, base);
        e.add(Di, offset);
        e.imm(C, words);
        e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
        e.0.jump_if(0x85, "fail");
    }
    e.mov(A, R10);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    e.imm(A, 0);
    e.0.bytes(&[0xc3]);
    Ok(code)
}

#[cfg(test)]
pub(crate) fn component_object() -> Result<Vec<u8>, CompileError> {
    use object::write::{Object, Symbol, SymbolSection};
    use object::{
        Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
    };
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.namespace.oracle".to_vec(),
        SectionKind::Text,
    );
    let bss = object.add_section(
        Vec::new(),
        b".bss.namespace.oracle".to_vec(),
        SectionKind::UninitializedData,
    );
    // Component-only unnamed guard pages surround the real 4096-byte symbol.
    // The C caller protects both pages; the future owner binding must separately
    // qualify its own production BSS geometry and cannot reuse this fixture.
    let offset = object.append_section_bss(bss, EXPECTED_BYTES * 3, 4096) + EXPECTED_BYTES;
    object.add_symbol(Symbol {
        name: EXPECTED.as_bytes().to_vec(),
        value: offset,
        size: EXPECTED_BYTES,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(bss),
        flags: SymbolFlags::None,
    });
    let debug = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    for (name, code) in [(PREPARE, prepare()?), (CHECK, check()?)] {
        super::root_region::append_code(&mut object, text, debug, name, code)?;
    }
    object
        .write()
        .map_err(|error| CompileError::Tool(error.to_string()))
}
