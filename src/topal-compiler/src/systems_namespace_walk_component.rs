//! Host adapters for the actual copied-byte walker, without kernel authority.
use super::*;

pub(super) fn object(namespace_entries: bool) -> Result<Vec<u8>, CompileError> {
    use object::write::{Object, Symbol, SymbolSection};
    use object::{
        Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
    };
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.namespace.walk.component".to_vec(),
        SectionKind::Text,
    );
    let mut code = X86FunctionEncoder::default();
    let mut e = E(&mut code);
    // SysV DI owned host buffer, SI dirfd, D walker mode, C flags. Establish
    // the SAME private walk entry's R12/B/BP registers; supply copied bytes in
    // owner+PATH. Neither this host buffer nor its numeric result is authority.
    e.0.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    e.mov(R12, Di);
    e.mov(B, Si);
    e.mov(Bp, C);
    e.store(R12, MODE, D);
    e.0.call("namespace-walk");
    e.mov(A, R15);
    e.0.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    let wrapper_bytes = e.0.bytes.len();
    super::emit_walk(&mut e, true, namespace_entries)?;
    if namespace_entries {
        files::entries::emit(e.0)?;
    }
    // CWD and absolute paths only: unexpected real-descriptor access must
    // trap, rather than treating a mock as a qualified descriptor service.
    e.0.bind("fd-lookup")?;
    e.0.jump("fatal");
    for (label, errno) in [
        ("namespace-efault", 14),
        ("namespace-enoent", 2),
        ("namespace-enametoolong", 36),
        ("namespace-ebadf", 9),
        ("namespace-enotdir", 20),
        ("namespace-eisdir", 21),
        ("namespace-erofs", 30),
    ] {
        e.0.bind(label)?;
        e.imm(R15, (-errno as i64) as u64);
        e.0.bytes(&[0xc3]);
    }
    e.0.bind("fatal")?;
    e.0.bytes(&[0x0f, 0x0b]);
    let bytes = code.finish()?;
    let offset = object.append_section_data(text, &bytes, 16);
    object.add_symbol(Symbol {
        name: b"topal_namespace_walk_component_v1".to_vec(),
        value: offset,
        size: wrapper_bytes as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
    object
        .write()
        .map_err(|e| CompileError::Tool(e.to_string()))
}
