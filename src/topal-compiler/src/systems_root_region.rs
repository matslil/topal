//! Private root-owned region invocation. No process or GS context is borrowed.
//! TOPAL-SYSTEMS-REGION-ROOT-{BINDING,CALL,RECLAIM}-001.
use super::x86_function::{E, R, R::*};
use super::{CompileError, X86FunctionEncoder};
use object::write::{Object, Relocation, SectionId, Symbol, SymbolSection};
use object::{
    RelocationEncoding, RelocationFlags, RelocationKind, SectionKind, SymbolFlags, SymbolKind,
    SymbolScope,
};

pub(crate) const APPLY: &str = "topal_x86_systems_region_library_apply_v1";
pub(crate) const FINALIZE: &str = "topal_x86_systems_region_library_finalize_v1";
pub(crate) const CONTROL: &str = "topal_x86_systems_region_control";
pub(crate) const ARENA: &str = "topal_x86_systems_region_arena";
pub(crate) const HISTORY: &str = "topal_x86_systems_region_history";
pub(crate) const ARENA_BYTES: u64 = 65536;
pub(crate) const RETIRED_STACK_BYTES: u64 = 4096;
pub(crate) const CALL_ALIGNMENT_BYTES: u64 = 32;

#[path = "systems_root_region_inspection.rs"]
mod inspection;
#[path = "systems_root_region_unwind.rs"]
mod unwind;
pub(crate) use inspection::inspect_linked;
pub use inspection::{SystemsRootRegionBinding, SystemsRootRegionCode, SystemsRootRegionStorage};

#[derive(Default)]
struct Code {
    encoder: X86FunctionEncoder,
    references: Vec<(u64, &'static str, bool)>,
    unwind: Vec<(u32, Vec<u8>)>,
}
fn address(
    e: &mut E<'_>,
    references: &mut Vec<(u64, &'static str, bool)>,
    register: R,
    symbol: &'static str,
) {
    e.imm(register, 0);
    references.push(((e.0.bytes.len() - 8) as u64, symbol, false));
}
fn branch(
    e: &mut E<'_>,
    references: &mut Vec<(u64, &'static str, bool)>,
    symbol: &'static str,
    call: bool,
) {
    e.0.bytes(&[if call { 0xe8 } else { 0xe9 }]);
    references.push((e.0.bytes.len() as u64, symbol, true));
    e.0.bytes(&[0; 4]);
}
fn zero_readback(e: &mut E<'_>, base: R, bytes: u64) {
    e.mov(Di, base);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xaf]);
    e.0.jump_if(0x85, "fail");
}
fn erase(e: &mut E<'_>, base: R, bytes: u64) {
    e.mov(Di, base);
    e.imm(A, 0);
    e.imm(C, bytes / 8);
    e.0.bytes(&[0xfc, 0xf3, 0x48, 0xab]);
    zero_readback(e, base, bytes);
}
fn clear_temporaries(e: &mut E<'_>) {
    for register in [A, C, D, Si, Di, R8, R9, R10, R11] {
        e.imm(register, 0);
    }
}

/// SysV entry: DI actual live root-owned bytes, SI extent64, D owner offset+1.
/// The checked root callsite supplies ownership; these address/word tests do not
/// create it. The source graph and explicit root stack/IF context are qualified.
fn apply(storage_bytes: u64) -> Result<Code, CompileError> {
    if storage_bytes < 4160 {
        return Err(CompileError::Tool(
            "root storage cannot contain its private prefix and region owner".into(),
        ));
    }
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.cmp(_Sp, 0x108000);
    e.0.jump_if(0x82, "fail");
    e.cmp(_Sp, 0x180000);
    e.0.jump_if(0x87, "fail");
    e.0.bytes(&[0x9c]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0e, 16]));
    e.0.bytes(&[0x58]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0e, 8]));
    e.arith(4, A, 0x200);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.cmp(Si, 64);
    e.0.jump_if(0x85, "fail");
    // Standard preserved frame, independent of the process continuation frame.
    e.0.bytes(&[0x55]);
    code.unwind
        .push((e.0.bytes.len() as u32, vec![0x0e, 16, 0x86, 2]));
    e.0.bytes(&[0x48, 0x89, 0xe5]);
    code.unwind.push((e.0.bytes.len() as u32, vec![0x0d, 6]));
    for (bytes, register, factor) in [
        (&[0x53][..], 3, 3),
        (&[0x41, 0x54], 12, 4),
        (&[0x41, 0x55], 13, 5),
        (&[0x41, 0x56], 14, 6),
        (&[0x41, 0x57], 15, 7),
    ] {
        e.0.bytes(bytes);
        code.unwind
            .push((e.0.bytes.len() as u32, vec![0x80 | register, factor]));
    }
    e.mov(R12, _Sp);
    e.mov(R13, Di);
    e.mov(R14, D);
    e.mov(A, R13);
    e.arith(4, A, 7);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R11, "topal_bootstrap_storage");
    e.rr(0x39, R13, R11);
    e.0.jump_if(0x82, "fail");
    e.mov(A, R13);
    e.rr(0x29, A, R11);
    e.cmp(A, 4096);
    e.0.jump_if(0x82, "fail");
    e.cmp(
        A,
        i32::try_from(
            storage_bytes
                .checked_sub(64)
                .ok_or_else(|| CompileError::Tool("root region store is too small".into()))?,
        )
        .map_err(|_| CompileError::Tool("root region store exceeds native bound".into()))?,
    );
    e.0.jump_if(0x87, "fail");
    e.add(A, 1);
    e.rr(0x39, A, R14);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, B, CONTROL);
    zero_readback(&mut e, B, 64);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(A, R11, 0);
    e.cmp(A, 0); // the initial root admits exactly one invocation
    e.0.jump_if(0x85, "fail");
    e.imm(R15, 1);
    address(
        &mut e,
        &mut code.references,
        A,
        crate::systems_region_library::EXPORT,
    );
    e.store(B, 0, A);
    e.store(B, 8, R14);
    e.imm(A, 0);
    e.store(B, 16, A);
    e.store(B, 24, A);
    e.store(B, 32, R13);
    e.imm(A, 64);
    e.store(B, 40, A);
    e.imm(A, 1);
    e.store(B, 48, A);
    e.imm(A, 0);
    e.store(B, 56, A);
    address(&mut e, &mut code.references, R11, ARENA);
    e.store(R11, 0, B);
    e.imm(A, 32);
    e.store(R11, 8, A);
    e.imm(A, 1);
    e.store(R11, 16, A);
    e.imm(A, 0);
    e.store(R11, 24, A);
    e.mov(Di, B);
    e.mov(Si, R14);
    e.mov(D, R15);
    e.arith(4, _Sp, -16);
    e.add(_Sp, -16);
    branch(
        &mut e,
        &mut code.references,
        crate::systems_region_library::EXPORT,
        true,
    );
    e.mov(_Sp, R12);
    // All live source bindings must already be revoked by the private bridge.
    for offset in [0, 8, 24, 32, 40, 48, 56] {
        e.load(A, B, offset);
        e.cmp(A, 0);
        e.0.jump_if(0x85, "fail");
    }
    e.load(A, B, 16);
    e.rr(0x39, A, R15);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R11, ARENA);
    e.load(A, R11, 0);
    e.rr(0x39, A, B);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 16);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 8);
    e.cmp(A, 32);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    erase(&mut e, R11, ARENA_BYTES);
    erase(&mut e, B, 64);
    e.mov(R11, R12);
    e.add(R11, -(RETIRED_STACK_BYTES as i32));
    erase(&mut e, R11, RETIRED_STACK_BYTES);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.store(R11, 0, R15); // scalar only; no source/owner/address/code binding
    clear_temporaries(&mut e);
    for (bytes, register) in [
        (&[0x41, 0x5f][..], 15),
        (&[0x41, 0x5e], 14),
        (&[0x41, 0x5d], 13),
        (&[0x41, 0x5c], 12),
        (&[0x5b], 3),
    ] {
        e.0.bytes(bytes);
        code.unwind
            .push((e.0.bytes.len() as u32, vec![0x08, register]));
    }
    e.0.bytes(&[0x5d]);
    code.unwind
        .push((e.0.bytes.len() as u32, vec![0x0c, 7, 8, 0x08, 6]));
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    // Terminal failures can arrive with different partial prologues.
    code.unwind.push((e.0.bytes.len() as u32, vec![0x07, 16]));
    branch(
        &mut e,
        &mut code.references,
        crate::systems_region_library::ABORT,
        false,
    );
    Ok(code)
}

/// Leaf temporary hook: exactly the caller's eight-byte return address, with
/// no stack use, callee-saved changes, process state, host service or callback.
fn allocator() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, CONTROL);
    e.load(A, R11, 24);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    e.load(A, R11, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.load(A, R11, 16);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    address(
        &mut e,
        &mut code.references,
        R8,
        crate::systems_region_library::EXPORT,
    );
    e.load(A, R11, 0);
    e.rr(0x39, A, R8);
    e.0.jump_if(0x85, "fail");
    address(&mut e, &mut code.references, R8, ARENA);
    e.load(A, R8, 0);
    e.rr(0x39, A, R11);
    e.0.jump_if(0x85, "fail");
    e.load(A, R8, 16);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    e.mov(A, Di);
    e.add(A, 7);
    e.0.jump_if(0x82, "fail");
    e.arith(4, A, -8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.load(R9, R8, 8);
    e.cmp(R9, 32);
    e.0.jump_if(0x82, "fail");
    e.cmp(R9, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    e.mov(R10, R9);
    e.rr(0x01, R10, A);
    e.0.jump_if(0x82, "fail");
    e.cmp(R10, ARENA_BYTES as i32);
    e.0.jump_if(0x87, "fail");
    e.load(C, R8, 24);
    e.add(C, 1);
    e.0.jump_if(0x82, "fail");
    e.cmp(C, 8192);
    e.0.jump_if(0x87, "fail");
    // No state changes until every allocation precondition is admitted.
    e.store(R8, 8, R10);
    e.store(R8, 24, C);
    e.mov(A, R8);
    e.rr(0x01, A, R9);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    branch(
        &mut e,
        &mut code.references,
        crate::systems_region_library::ABORT,
        false,
    );
    Ok(code)
}

fn finalize() -> Result<Code, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, CONTROL);
    zero_readback(&mut e, R11, 64);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(A, R11, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    erase(&mut e, R11, 8);
    clear_temporaries(&mut e);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    branch(
        &mut e,
        &mut code.references,
        crate::systems_region_library::ABORT,
        false,
    );
    Ok(code)
}

pub(super) fn append(
    object: &mut Object<'_>,
    text: SectionId,
    storage_bytes: u64,
) -> Result<(), CompileError> {
    for (name, bytes, alignment) in [
        (ARENA, ARENA_BYTES, 4096),
        (CONTROL, 64, 8),
        (HISTORY, 8, 8),
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
    // Fatal hook is a qualified terminal leaf; it never resumes an owner.
    super::append_encoded_function(
        object,
        text,
        crate::systems_region_library::ABORT,
        &[0xfa, 0xf4, 0xeb, 0xfd],
    );
    let debug_frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    for (name, code) in [
        (APPLY, apply(storage_bytes)?),
        (crate::systems_region_library::ALLOCATE, allocator()?),
        (FINALIZE, finalize()?),
    ] {
        let Code {
            encoder,
            references,
            unwind: rows,
        } = code;
        let bytes = encoder.finish()?;
        let start = object.append_section_data(text, &bytes, 16);
        let function = object.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: start,
            size: bytes.len() as u64,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(text),
            flags: SymbolFlags::None,
        });
        unwind::append(object, debug_frame, function, bytes.len() as u64, &rows)?;
        for (offset, target, relative) in references {
            let symbol = object.symbol_id(target.as_bytes()).unwrap_or_else(|| {
                object.add_symbol(Symbol {
                    name: target.as_bytes().to_vec(),
                    value: 0,
                    size: 0,
                    kind: SymbolKind::Text,
                    scope: SymbolScope::Unknown,
                    weak: false,
                    section: SymbolSection::Undefined,
                    flags: SymbolFlags::None,
                })
            });
            object
                .add_relocation(
                    text,
                    Relocation {
                        offset: start + offset,
                        symbol,
                        addend: if relative { -4 } else { 0 },
                        flags: RelocationFlags::Generic {
                            kind: if relative {
                                RelocationKind::PltRelative
                            } else {
                                RelocationKind::Absolute
                            },
                            encoding: if relative {
                                RelocationEncoding::X86Branch
                            } else {
                                RelocationEncoding::Generic
                            },
                            size: if relative { 32 } else { 64 },
                        },
                    },
                )
                .map_err(|error| CompileError::Tool(format!("root region relocation: {error}")))?;
        }
    }
    Ok(())
}
