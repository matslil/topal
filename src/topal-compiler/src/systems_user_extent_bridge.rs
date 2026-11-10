//! Private C-ABI bridge for the unpublished full-root experiment.
use super::root_region::{Code, append_code, branch};
use super::x86_function::{E, R::*};
use super::*;

pub(crate) fn object() -> Result<Vec<u8>, CompileError> {
    let mut code = Code::default();
    let mut e = E(&mut code.encoder);
    e.mov(C, D);
    e.mov(D, Si);
    e.mov(Si, Di);
    e.imm(Di, 8);
    e.imm(R8, 0);
    e.0.bytes(&[0x6a, 0]);
    code.unwind
        .push((code.encoder.bytes.len() as u32, vec![0x0e, 16]));
    branch(
        &mut E(&mut code.encoder),
        &mut code.references,
        "topal_x86_frame_gateway_v1",
        true,
    );
    let point = code.encoder.bytes.len() as u64;
    let mut e = E(&mut code.encoder);
    e.imm(C, 0);
    e.store(_Sp, -8, C);
    e.store(_Sp, 0, C);
    e.add(_Sp, 8);
    code.unwind
        .push((code.encoder.bytes.len() as u32, vec![0x0e, 8]));
    code.encoder.bytes(&[0xc3]);
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let section = object.add_section(
        Vec::new(),
        b".text.topal.user.extent.bridge".to_vec(),
        SectionKind::Text,
    );
    let frame = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    object.add_section(Vec::new(), b".note.GNU-stack".to_vec(), SectionKind::Other);
    append_code(
        &mut object,
        section,
        frame,
        "topal_x86_user_extent_call_v1",
        code,
    )?;
    let start = object
        .symbol(object.symbol_id(b"topal_x86_user_extent_call_v1").unwrap())
        .value;
    object.add_symbol(Symbol {
        name: b"topal_x86_user_extent_return_v1".to_vec(),
        value: start + point,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
