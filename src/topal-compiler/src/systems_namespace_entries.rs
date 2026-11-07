//! V6 entry associations; identity and copied names never create authority.
//! TOPAL-SYSTEMS-NAMESPACE-OWNERSHIP-001 and -ACTION-001.
use super::super::{CompileError, E, R, R::*, X86FunctionEncoder};
use super::{BASE, NAMES, shift};

pub(in super::super) const ENTRIES: i32 = BASE + 64;
pub(in super::super) const COUNT: u64 = 4;
pub(in super::super) const CHECK_STACK_BYTES: u64 = 88;
pub(in super::super) const LOOKUP_STACK_BYTES: u64 = CHECK_STACK_BYTES + 8;

pub(in super::super) fn name_pointer(e: &mut E<'_>, destination: R, slot: R) {
    e.mov(destination, slot);
    shift(e, destination, true, 8);
    e.rr(0x01, destination, R12);
    e.add(destination, NAMES);
}

/// Actual R12 controller already belongs to the captured entry. Check complete
/// canonical bytes before consuming a name/link association. This is a private
/// label inside the exact bound process entry, not another namespace callback.
pub(in super::super) fn emit(code: &mut X86FunctionEncoder) -> Result<(), CompileError> {
    let mut e = E(code);
    e.0.bind("file-entry-table-check")?;
    e.0.bytes(&[0x57]);
    e.mov(Di, R12);
    e.add(Di, BASE);
    e.0.call("file-entry-metadata-validate");
    e.0.bytes(&[0x5f]);
    // Retire our popped DI and the completed inner call's return word before
    // checking the Boolean; no register save survives a refusal either.
    for offset in [-16, -8] {
        e.mem(0xc7, A, R::_Sp, offset);
        e.0.bytes(&[0; 4]);
    }
    for offset in [-16, -8] {
        e.mem(0x81, Di, R::_Sp, offset);
        e.0.bytes(&[0; 4]);
        e.0.jump_if(0x85, "fatal");
    }
    e.cmp(A, 1);
    e.0.jump_if(0x85, "fatal");
    e.0.bytes(&[0xc3]);
    e.0.bind("file-entry-metadata-validate")?;
    let validator = super::super::super::namespace_oracle::metadata_validator()?;
    if !validator.references.is_empty() {
        return Err(CompileError::Tool(
            "E-SYSTEMS-NAMESPACE: metadata validator must remain a closed leaf".into(),
        ));
    }
    e.0.bytes(&validator.encoder.finish()?);

    // SI raw component, D length. Return CF0, R10 entry, R11 slot, R15 id;
    // CF1 means no entry. The checked table guarantees a live unique inode.
    e.0.bind("file-entry-by-name")?;
    e.0.call("file-entry-table-check");
    e.mov(R10, R12);
    e.add(R10, ENTRIES);
    e.imm(R11, 0);
    e.0.bind("file-entry-name-next")?;
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "file-entry-name-advance");
    e.mem(0x3b, D, R10, 8);
    e.0.jump_if(0x85, "file-entry-name-advance");
    name_pointer(&mut e, R8, R11);
    e.mov(Di, Si);
    e.mov(C, D);
    e.0.bind("file-entry-name-byte")?;
    super::super::namespace::byte(&mut e, A, R8, 0);
    super::super::namespace::byte(&mut e, R9, Di, 0);
    e.rr(0x39, A, R9);
    e.0.jump_if(0x85, "file-entry-name-advance");
    e.add(R8, 1);
    e.add(Di, 1);
    e.add(C, -1);
    e.cmp(C, 0);
    e.0.jump_if(0x85, "file-entry-name-byte");
    e.load(R15, R10, 0);
    e.0.bytes(&[0xf8, 0xc3]);
    e.0.bind("file-entry-name-advance")?;
    e.add(R10, 16);
    e.add(R11, 1);
    e.cmp(R11, COUNT as i32);
    e.0.jump_if(0x82, "file-entry-name-next");
    e.imm(R10, 0);
    e.imm(R15, 0);
    e.0.bytes(&[0xf9, 0xc3]);

    // Return a clean entry/name slot without publishing or reserving it.
    e.0.bind("file-entry-free")?;
    e.0.call("file-entry-table-check");
    e.mov(R10, R12);
    e.add(R10, ENTRIES);
    e.imm(R11, 0);
    e.0.bind("file-entry-free-next")?;
    e.load(A, R10, 0);
    e.cmp(A, 0);
    e.0.jump_if(0x84, "file-entry-free-found");
    e.add(R10, 16);
    e.add(R11, 1);
    e.cmp(R11, COUNT as i32);
    e.0.jump_if(0x82, "file-entry-free-next");
    e.0.bytes(&[0xf9, 0xc3]);
    e.0.bind("file-entry-free-found")?;
    e.0.bytes(&[0xf8, 0xc3]);
    Ok(())
}

/// R10 live inode; canonical table has already been checked. Counted references
/// must be zero before releasing pages/object; never erase a name by object slot.
pub(in super::super) fn assert_unlinked(e: &mut E<'_>) {
    e.load(A, R10, 56);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
    for slot in 0..COUNT as i32 {
        e.load(A, R12, ENTRIES + slot * 16);
        e.mem(0x3b, A, R10, 8);
        e.0.jump_if(0x84, "fatal");
    }
    // Object names are absent from the new inode record.
    e.load(A, R10, 72);
    e.cmp(A, 0);
    e.0.jump_if(0x85, "fatal");
}

/// Terminal caller has proved no description retains R10. Remove every name
/// for that inode, preserving unrelated slots regardless of their object index.
pub(in super::super) fn retire_inode_names(e: &mut E<'_>) -> Result<(), CompileError> {
    e.imm(R8, 0);
    e.0.bind("file-terminal-entry-next")?;
    e.mov(R11, R8);
    shift(e, R11, true, 4);
    e.rr(0x01, R11, R12);
    e.add(R11, ENTRIES);
    e.load(A, R11, 0);
    e.mem(0x3b, A, R10, 8);
    e.0.jump_if(0x85, "file-terminal-entry-advance");
    super::erase(e, R11, 16);
    name_pointer(e, R11, R8);
    super::erase(e, R11, 256);
    e.0.bind("file-terminal-entry-advance")?;
    e.add(R8, 1);
    e.cmp(R8, COUNT as i32);
    e.0.jump_if(0x82, "file-terminal-entry-next");
    e.imm(A, 0);
    e.store(R10, 56, A);
    Ok(())
}

#[cfg(test)]
pub(in super::super::super) fn component_object() -> Result<Vec<u8>, CompileError> {
    use object::write::{Object, Symbol, SymbolSection};
    use object::{
        Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
    };
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = object.add_section(
        Vec::new(),
        b".text.namespace.entries.component".to_vec(),
        SectionKind::Text,
    );
    let debug = object.add_section(Vec::new(), b".debug_frame".to_vec(), SectionKind::Debug);
    super::super::super::root_region::append_code(
        &mut object,
        text,
        debug,
        "topal_namespace_metadata_check_v1",
        super::super::super::namespace_oracle::metadata_validator()?,
    )?;
    let mut code = X86FunctionEncoder::default();
    let mut e = E(&mut code);
    // Host-only SysV adapters to the SAME private label emitters. They establish
    // no kernel owner/loan authority and do not model Linux error dispositions.
    e.0.bytes(&[0x41, 0x54, 0x41, 0x57, 0x55]);
    e.mov(R12, Di);
    e.mov(R::_Bp, C);
    e.0.call("file-entry-by-name");
    e.store(R::_Bp, 0, R11);
    e.mov(A, R15);
    e.0.bytes(&[0x5d, 0x41, 0x5f, 0x41, 0x5c, 0xc3]);
    let lookup_bytes = e.0.bytes.len();
    let free_offset = lookup_bytes;
    e.0.bytes(&[0x41, 0x54]);
    e.mov(R12, Di);
    e.0.call("file-entry-free");
    e.mov(A, R11);
    e.0.bytes(&[0x41, 0x5c, 0xc3]);
    let free_bytes = e.0.bytes.len() - free_offset;
    let retire_offset = e.0.bytes.len();
    e.0.bytes(&[0x41, 0x54]);
    e.mov(R12, Di);
    e.mov(R10, Si);
    shift(&mut e, R10, true, 7);
    e.rr(0x01, R10, R12);
    e.add(R10, super::OBJECTS);
    e.0.call("component-retire-entries");
    e.0.bytes(&[0x41, 0x5c, 0xc3]);
    let retire_bytes = e.0.bytes.len() - retire_offset;
    let erase_offset = e.0.bytes.len();
    super::erase(&mut e, Di, 512);
    e.0.bytes(&[0xc3]);
    let erase_bytes = e.0.bytes.len() - erase_offset;
    emit(e.0)?;
    e.0.bind("component-retire-entries")?;
    retire_inode_names(&mut e)?;
    e.0.bytes(&[0xc3]);
    e.0.bind("fatal")?;
    e.0.bytes(&[0x0f, 0x0b]); // Host-only invariant trap; no ring-0 halt claim.
    let bytes = code.finish()?;
    let offset = object.append_section_data(text, &bytes, 16);
    for (name, start, length) in [
        ("topal_namespace_entry_lookup_v1", 0, lookup_bytes),
        ("topal_namespace_entry_free_v1", free_offset, free_bytes),
        (
            "topal_namespace_entry_retire_v1",
            retire_offset,
            retire_bytes,
        ),
        ("topal_namespace_erase_di_v1", erase_offset, erase_bytes),
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
