//! Same final-name helpers under a host ABI; no user-copy or owner grant.
use super::*;

pub(super) fn object() -> Result<Vec<u8>, CompileError> {
    use object::write::{Object, Symbol, SymbolSection};
    use object::{
        Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
    };
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.namespace.final.component".to_vec(),
        SectionKind::Text,
    );
    let mut code = X86FunctionEncoder::default();
    let mut e = E(&mut code);
    // DI raw name, SI length, D FINAL. Return0 normal/1 dot/dotdot/root.
    e.mov(A, D);
    e.mov(D, Si);
    e.mov(Si, Di);
    e.0.call("namespace-transaction-final-kind");
    e.0.bytes(&[0xc3]);
    let kind_bytes = e.0.bytes.len();
    let lookup_start = kind_bytes;
    // DI owner, SI parent, D raw name, C length, R8 output entry slot/error.
    e.0.bytes(&[0x41, 0x54, 0x41, 0x57, 0x55]);
    e.mov(R12, Di);
    e.mov(R15, Si);
    e.mov(Bp, R8);
    e.mov(Si, D);
    e.mov(D, C);
    e.0.call("namespace-transaction-final-lookup");
    e.store(Bp, 0, R11);
    e.mov(A, R15);
    e.0.bytes(&[0x5d, 0x41, 0x5f, 0x41, 0x5c, 0xc3]);
    let lookup_bytes = e.0.bytes.len() - lookup_start;
    publication::emit_final_helpers(&mut e)?;
    files::entries::emit(e.0)?;
    e.0.bind("namespace-enametoolong")?;
    e.imm(R15, 0);
    e.imm(R11, (-36_i64) as u64);
    e.0.bytes(&[0xc3]);
    e.0.bind("fatal")?;
    e.0.bytes(&[0x0f, 0x0b]);
    let bytes = code.finish()?;
    let offset = object.append_section_data(text, &bytes, 16);
    for (name, start, length) in [
        ("topal_namespace_final_kind_v1", 0, kind_bytes),
        (
            "topal_namespace_final_lookup_v1",
            lookup_start,
            lookup_bytes,
        ),
    ] {
        object.add_symbol(Symbol {
            name: name.as_bytes().to_vec(),
            value: offset + start as u64,
            size: length as u64,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(text),
            flags: SymbolFlags::None,
        });
    }
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
