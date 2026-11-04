//! Actual field loan under the qualified serialized file-controller domain.
//! TOPAL-SYSTEMS-REGION-ACTION-{OWNER,RETURN,POSITION}-001.
use super::root_region::{Code, address, branch, clear_temporaries, erase, zero_readback};
use super::x86_function::{E, R, R::*};
use super::{CompileError, X86FunctionEncoder};
use crate::systems_region_library::{
    ACTION_ABORT, ACTION_ALLOCATE, ACTION_EXPORT, FILE_FRAME_ABORT, FILE_FRAME_ALLOCATE,
    FILE_FRAME_EXPORT, FRAME_ABORT, FRAME_ALLOCATE, FRAME_EXPORT,
};
use object::write::{Object, SectionId, Symbol, SymbolSection};
use object::{SectionKind, SymbolFlags, SymbolKind, SymbolScope};

#[path = "systems_file_region_inspection.rs"]
mod inspection;
pub use inspection::SystemsFilePositionRegionBinding;
pub(crate) use inspection::inspect_linked;
pub(super) use inspection::inspect_owned;

/// Caller-owned symbols and exact templates for the shared region verifier.
/// Source instances still have separate code, arenas, controls and histories.
pub(super) struct InvocationTemplate {
    pub(super) framed: bool,
    pub(super) schema: &'static str,
    pub(super) interface: &'static str,
    pub(super) export: &'static str,
    pub(super) allocate: &'static str,
    pub(super) abort_hook: &'static str,
    pub(super) arena: &'static str,
    pub(super) control: &'static str,
    pub(super) history: &'static str,
    pub(super) live_frame_bytes: u64,
    pub(super) call_alignment_bytes: u64,
    pub(super) code: Vec<(&'static str, Code)>,
}
pub(super) fn fatal_template() -> Code {
    Code {
        encoder: X86FunctionEncoder {
            bytes: vec![0xfa, 0xf4, 0xeb, 0xfd],
            ..X86FunctionEncoder::default()
        },
        ..Code::default()
    }
}

pub(crate) const APPLY: &str = "topal_x86_systems_file_position_apply_v1";
pub(crate) const FINALIZE: &str = "topal_x86_systems_file_position_finalize_v1";
pub(crate) const ARENA: &str = "topal_x86_systems_file_position_arena";
pub(crate) const CONTROL: &str = "topal_x86_systems_file_position_control";
pub(crate) const HISTORY: &str = "topal_x86_systems_file_position_history";
pub(super) const ARENA_BYTES: u64 = 65536;
pub(super) const RETIRED_STACK_BYTES: u64 = 4096;
pub(super) const LIVE_FRAME_BYTES: u64 = 88;
// Includes the immutable-read caller's eight-byte count/alignment save and
// its 24-byte inner alignment adjustment. The other two bound call sites
// need only the 16-byte inner adjustment. This is a caller-specific allowance,
// not a bound for arbitrary native entry stacks.
pub(super) const CALL_ALIGNMENT_BYTES: u64 = 32;

/// DI controller, SI description, D effective transfer base, C actual prefix.
/// Only the checked handler supplies this grant; words never manufacture it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FileRegionProfile {
    Action,
    Frame,
    Instance,
}
impl FileRegionProfile {
    pub(super) fn from_interface(
        interface: topal_language::compiler::CompilerSystemsSourceLibraryInterface,
    ) -> Result<Self, CompileError> {
        use topal_language::compiler::CompilerSystemsSourceLibraryInterface as Interface;
        match interface {
            Interface::BorrowedByteRegionActionV1 => Ok(Self::Action),
            Interface::BorrowedByteRegionFramedActionV1 => Ok(Self::Frame),
            Interface::BorrowedByteRegionInstanceActionV1 => Ok(Self::Instance),
            _ => Err(CompileError::Tool(
                "E-SYSTEMS-REGION-ACTION: incompatible file source context".into(),
            )),
        }
    }
    fn framed(self) -> bool {
        self != Self::Action
    }
}
fn native_names(profile: FileRegionProfile) -> (&'static str, &'static str, &'static str) {
    match profile {
        FileRegionProfile::Action => (ACTION_EXPORT, ACTION_ALLOCATE, ACTION_ABORT),
        FileRegionProfile::Frame => (FRAME_EXPORT, FRAME_ALLOCATE, FRAME_ABORT),
        FileRegionProfile::Instance => (FILE_FRAME_EXPORT, FILE_FRAME_ALLOCATE, FILE_FRAME_ABORT),
    }
}

pub(super) fn apply(profile: FileRegionProfile) -> Result<Code, CompileError> {
    let (export, _, abort) = native_names(profile);
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
    e.add(_Sp, -32); // immutable input/result scalars, outside the erased reserve
    e.mov(R12, _Sp);
    e.mov(R13, Di);
    e.mov(R14, Si);
    e.store(R12, 0, D);
    e.store(R12, 8, C);
    e.imm(A, 0);
    e.store(R12, 16, A);
    e.store(R12, 24, A);
    e.cmp(C, 1);
    e.0.jump_if(0x82, "fail");
    e.cmp(C, 256);
    e.0.jump_if(0x87, "fail");
    e.gs(
        0x8b,
        R11,
        super::process_runtime::file_descriptions::TABLE_CONTEXT,
    );
    e.rr(0x39, R13, R11);
    e.0.jump_if(0x85, "fail");
    e.cmp(R13, 0x200000);
    e.0.jump_if(0x82, "fail");
    e.mov(A, R13);
    e.arith(4, A, 4095);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.load(A, R13, 0);
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fail");
    e.mov(A, R14);
    e.rr(0x29, A, R13);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 320);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 320 + 16 * 64);
    e.0.jump_if(0x82, "primary-bank");
    e.cmp(A, 0x3d000);
    e.0.jump_if(0x82, "fail");
    e.cmp(A, 0x3d000 + 16 * 64);
    e.0.jump_if(0x83, "fail");
    e.add(A, -0x3d000);
    e.0.jump("bank-stride");
    e.0.bind("primary-bank")?;
    e.add(A, -320);
    e.0.bind("bank-stride")?;
    e.arith(4, A, 63);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fail");
    e.load(A, R14, 8);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "fail");
    e.load(A, R14, 0);
    e.cmp(A, 3);
    e.0.jump_if(0x84, "regular-kind");
    e.cmp(A, 7);
    e.0.jump_if(0x85, "fail");
    e.0.bind("regular-kind")?;
    address(&mut e, &mut code.references, B, CONTROL);
    zero_readback(&mut e, B, 64);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, HISTORY);
    e.load(A, R11, 0);
    e.mov(R15, A);
    e.add(R15, 1);
    e.0.jump_if(0x82, "fail");
    e.store(B, 16, A);
    address(&mut e, &mut code.references, A, export);
    e.store(B, 0, A);
    e.store(B, 8, R13);
    e.mov(A, R14);
    e.add(A, 24);
    e.store(B, 32, A);
    e.imm(A, 8);
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
    if profile.framed() {
        // The first two words of this owned 32-byte live frame are immutable
        // throughout source execution. It is above the retired stack reserve
        // and outside the numeric arena; all 32 bytes are erased on return.
        e.mov(C, R12);
        e.imm(R8, 2);
    } else {
        e.load(C, R12, 0);
        e.load(R8, R12, 8);
    }
    e.arith(4, _Sp, -16);
    e.add(_Sp, -16);
    branch(&mut e, &mut code.references, export, true);
    e.mov(_Sp, R12);
    e.store(R12, 16, A); // actual normalized source Boolean, not a policy fallback
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
    e.rr(0x39, A, R15);
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
    e.store(R11, 0, R15);
    e.load(R15, R12, 16);
    erase(&mut e, R12, 32); // retire input/result scalar staging as well
    clear_temporaries(&mut e);
    e.cmp(R15, 1);
    e.0.jump_if(0x85, "fail"); // admitted action false never resumes the owner
    e.mov(A, R15);
    e.add(_Sp, 32);
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
    code.unwind.push((e.0.bytes.len() as u32, vec![0x07, 16]));
    branch(&mut e, &mut code.references, abort, false);
    Ok(code)
}

pub(super) fn allocator(profile: FileRegionProfile) -> Result<Code, CompileError> {
    let (export, _, abort) = native_names(profile);
    super::root_region::allocator_for(CONTROL, ARENA, export, abort, true)
}

pub(super) fn finalize(profile: FileRegionProfile) -> Result<Code, CompileError> {
    let (_, _, abort) = native_names(profile);
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    address(&mut e, &mut code.references, R11, ARENA);
    zero_readback(&mut e, R11, ARENA_BYTES);
    address(&mut e, &mut code.references, R11, CONTROL);
    zero_readback(&mut e, R11, 64);
    address(&mut e, &mut code.references, R11, HISTORY);
    erase(&mut e, R11, 8); // zero invocations is also a valid terminal history
    clear_temporaries(&mut e);
    e.0.bytes(&[0xc3]);
    e.0.bind("fail")?;
    branch(&mut e, &mut code.references, abort, false);
    Ok(code)
}

pub(super) fn append(
    object: &mut Object<'_>,
    text: SectionId,
    profile: FileRegionProfile,
) -> Result<(), CompileError> {
    let (_, allocate, abort) = native_names(profile);
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
    super::append_encoded_function(object, text, abort, &[0xfa, 0xf4, 0xeb, 0xfd]);
    let debug = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    for (name, code) in [
        (APPLY, apply(profile)?),
        (allocate, allocator(profile)?),
        (FINALIZE, finalize(profile)?),
    ] {
        super::root_region::append_code(object, text, debug, name, code)?;
    }
    Ok(())
}

/// Called only after the handler has completed an admitted nonempty transfer.
/// The live controller/description and scalars are already in retained registers.
pub(super) fn emit_position_update(code: &mut X86FunctionEncoder, count: R) {
    let mut e = E(code);
    e.mov(Di, R12);
    e.mov(Si, R14);
    e.mov(D, R15);
    e.mov(C, count);
    e.0.external_call(APPLY);
    e.load(R15, R14, 24); // the actual source-written end, after complete cleanup
}
