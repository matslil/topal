fn translation_backing_selector() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xf6]); // test boot_params, boot_params
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]);
    code.bytes(&[0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]);

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]); // candidate = base
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]); // length
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]); // type
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]); // end = base
    code.bytes(&[0x4d, 0x01, 0xcb]); // end += length
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0xb8, 0x00, 0x00, 0x00, 0x01]);
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x89, 0xc2]); // end of 5 pages = candidate
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x50, 0x00, 0x00]);
    code.jump_if(0x82, "next");
    code.bytes(&[0x44, 0x8b, 0x8e, 0x18, 0x02, 0x00, 0x00]); // resource base
    code.bytes(&[0x44, 0x8b, 0x9e, 0x1c, 0x02, 0x00, 0x00]); // resource size
    code.bytes(&[0x4d, 0x01, 0xcb]); // resource end
    code.bytes(&[0x4d, 0x39, 0xca]); // extent end <= resource base
    code.jump_if(0x86, "resource-disjoint");
    code.bytes(&[0x4c, 0x39, 0xd8]); // candidate >= resource end
    code.jump_if(0x83, "resource-disjoint");
    code.jump("next");
    code.bind("resource-disjoint")?;
    code.bytes(&[0x4d, 0x39, 0xda]); // candidate + 5 pages <= RAM end
    code.jump_if(0x87, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0xb0, 0xff, 0x3f]); // all backing below 1 GiB
    code.jump_if(0x87, "next");
    code.bytes(&[0xc3]);

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");
    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_space_builder() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x49, 0x89, 0xf8]); // r8 = root
    code.bytes(&[0x31, 0xc0]); // zero value
    code.bytes(&[0xb9, 0x00, 0x0a, 0x00, 0x00]); // 2560 qwords
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // cld; rep stosq
    code.bytes(&[0x4d, 0x8d, 0x88, 0x00, 0x10, 0x00, 0x00]); // pdpt
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x07]);
    code.bytes(&[0x49, 0x89, 0x00]); // pml4[0]
    code.bytes(&[0x4d, 0x8d, 0x88, 0x00, 0x20, 0x00, 0x00]); // pd
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x07]);
    code.bytes(&[0x49, 0x89, 0x80, 0x00, 0x10, 0x00, 0x00]); // pdpt[0]
    code.bytes(&[0x4d, 0x8d, 0x90, 0x00, 0x30, 0x00, 0x00]); // local APIC PD
    code.bytes(&[0x4c, 0x89, 0xd0, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x49, 0x89, 0x80, 0x18, 0x10, 0x00, 0x00]); // pdpt[3]
    code.bytes(&[0x49, 0x8d, 0xb8, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc9]);
    code.bind("leaves")?;
    code.bytes(&[0x48, 0x89, 0xc8, 0x48, 0xc1, 0xe0, 0x15]);
    code.bytes(&[0x48, 0x0d, 0x83, 0x00, 0x00, 0x00]); // present/write/2MiB
    code.bytes(&[0x48, 0x89, 0x04, 0xcf]);
    code.bytes(&[0xff, 0xc1, 0x81, 0xf9, 0x00, 0x02, 0x00, 0x00]);
    code.jump_if(0x85, "leaves");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fec0_009b_u64.to_le_bytes()); // HPET 2 MiB MMIO leaf
    code.bytes(&[0x49, 0x89, 0x82, 0xb0, 0x0f, 0x00, 0x00]); // PDE[502]
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fee0_009b_u64.to_le_bytes()); // APIC 2 MiB MMIO leaf
    code.bytes(&[0x49, 0x89, 0x82, 0xb8, 0x0f, 0x00, 0x00]); // PDE[503]
    code.bytes(&[0x0f, 0xae, 0xf0]); // mfence publishes the inactive hierarchy
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.finish()
}

fn translation_space_activator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0xb0, 0xff, 0x3f]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0xb9, 0x80, 0x00, 0x00, 0xc0]); // IA32_EFER
    code.bytes(&[0x0f, 0x32]); // rdmsr
    code.bytes(&[0x0d, 0x00, 0x08, 0x00, 0x00]); // enable NXE
    code.bytes(&[0x0f, 0x30]); // wrmsr
    code.bytes(&[0x0f, 0x22, 0xdf]); // mov cr3, rdi
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_begin() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x01]); // unmap edit kind
    code.jump_if(0x84, "unmap");
    code.bytes(&[0x85, 0xd2]); // map edit kind is zero
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xff]); // payload frame
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xc7, 0x00, 0x10, 0x00, 0x00]); // metadata floor
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // boot params
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]);
    code.bytes(&[0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]);

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]);
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]);
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]);
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]);
    code.bytes(&[0x4d, 0x01, 0xcb]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x39, 0xf8]);
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0x48, 0x89, 0xf8]);
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x89, 0xc2]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x20, 0x00, 0x00]); // two pages
    code.jump_if(0x82, "next");
    code.bytes(&[0x4d, 0x39, 0xda]);
    code.jump_if(0x87, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0xe0, 0xff, 0x3f]);
    code.jump_if(0x87, "next");
    code.bytes(&[0xc3]);

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");
    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");
    code.jump("fail");

    code.bind("unmap")?;
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_map() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc6, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xf8]); // retain PD
    code.bytes(&[0x4c, 0x8d, 0x8f, 0x00, 0x10, 0x00, 0x00]); // PT
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x04, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // clear two pages
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x49, 0x89, 0x00]); // PD[0] -> PT
    code.bytes(&[0x48, 0x89, 0xf0]);
    code.bytes(&[0x48, 0xba, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);
    code.bytes(&[0x48, 0x09, 0xd0]);
    code.bytes(&[0x49, 0x89, 0x01]); // PT[0] -> payload, writable and NX
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_unmap() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0x08, 0x10, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_commit() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "unmap");
    code.bytes(&[0x85, 0xd2]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc6, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x89, 0x87, 0x08, 0x10, 0x00, 0x00]); // publish PDPT[1]
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0xb8, 0x00, 0x00, 0x00, 0x40, 0xc3]); // opaque mapping VA

    code.bind("unmap")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x85, "fail");
    code.bytes(&[
        0x48, 0xc7, 0x87, 0x08, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x0f, 0x01, 0x3e]); // invlpg [rsi]
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

struct LinkableX86Function {
    bytes: Vec<u8>,
    calls: Vec<(u64, &'static str)>,
}

#[derive(Default)]
struct X86FunctionEncoder {
    bytes: Vec<u8>,
    labels: BTreeMap<String, usize>,
    fixups: Vec<(usize, String)>,
    external_calls: Vec<(u64, &'static str)>,
}

impl X86FunctionEncoder {
    fn bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn bind(&mut self, label: &str) -> Result<(), CompileError> {
        if self
            .labels
            .insert(label.to_owned(), self.bytes.len())
            .is_some()
        {
            return Err(CompileError::Tool(format!(
                "duplicate generated x86 label `{label}`"
            )));
        }
        Ok(())
    }

    fn jump_if(&mut self, condition: u8, label: &str) {
        self.bytes.extend_from_slice(&[0x0f, condition]);
        self.fixup(label);
    }

    fn jump(&mut self, label: &str) {
        self.bytes.push(0xe9);
        self.fixup(label);
    }

    fn call(&mut self, label: &str) {
        self.bytes.push(0xe8);
        self.fixup(label);
    }

    fn rip_lea(&mut self, opcode: &[u8], label: &str) {
        self.bytes.extend_from_slice(opcode);
        self.fixup(label);
    }

    fn fixup(&mut self, label: &str) {
        let offset = self.bytes.len();
        self.fixups.push((offset, label.to_owned()));
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn external_call(&mut self, symbol: &'static str) {
        self.bytes.push(0xe8);
        self.external_calls.push((self.bytes.len() as u64, symbol));
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn finish_linkable(mut self) -> Result<LinkableX86Function, CompileError> {
        let calls = std::mem::take(&mut self.external_calls);
        Ok(LinkableX86Function {
            bytes: self.finish()?,
            calls,
        })
    }

    fn finish(mut self) -> Result<Vec<u8>, CompileError> {
        if !self.external_calls.is_empty() {
            return Err(CompileError::Tool(
                "external native calls require relocation-aware publication".into(),
            ));
        }
        for (offset, label) in self.fixups {
            let target = *self.labels.get(&label).ok_or_else(|| {
                CompileError::Tool(format!("missing generated x86 label `{label}`"))
            })?;
            let displacement = i64::try_from(target).unwrap_or(i64::MAX)
                - i64::try_from(offset + 4).unwrap_or(i64::MIN);
            let displacement = i32::try_from(displacement).map_err(|_| {
                CompileError::Tool(format!("generated x86 jump to `{label}` is out of range"))
            })?;
            self.bytes[offset..offset + 4].copy_from_slice(&displacement.to_le_bytes());
        }
        Ok(self.bytes)
    }
}

fn provider_note(plan: &X86SystemsProviderPlan) -> String {
    let mut note = format!(
        "revision={X86_SYSTEMS_PROVIDER_OBJECT_REVISION}\nprovider={}\ntarget={}\nboard={}\nprofile={}\nmachine-cpu={}\ncodegen-cpu={}\nplatform-abi={}\ndata-layout={}\nobject-format={}\nrelocation-model={}\ncode-model={}\nstorage-capacity={}\nstorage-alignment={}\nsource-library-interface={}\n",
        X86_SYSTEMS_PROVIDER_REVISION,
        plan.target,
        plan.board,
        plan.profile,
        plan.machine_cpu_model,
        plan.codegen_cpu,
        plan.platform_abi,
        plan.data_layout,
        plan.object_format,
        plan.relocation_model,
        plan.code_model,
        plan.bootstrap_placement.capacity_bytes,
        plan.bootstrap_placement.alignment_bytes,
        plan.source_library_interface.unwrap_or("none"),
    );
    if let Some(interface) = plan.time_library_interface {
        note.push_str(&format!("time-library-interface={interface}\nnumeric-runtime-bytes=65536\nnumeric-stack-reserve=4096\n"));
    }
    if let Some(interface) = plan.file_position_library_interface {
        note.push_str(&format!("file-position-library-interface={interface}\nfile-position-runtime-bytes=65536\nfile-position-control-bytes=64\nfile-position-history-bytes=8\nfile-position-field-bytes=8\nfile-position-retired-stack-bytes=4096\nfile-position-live-frame-bytes={}\nfile-position-call-alignment-bytes={}\nfile-position-domain=serialized-single-processor-masked\n",
            file_region::LIVE_FRAME_BYTES, file_region::CALL_ALIGNMENT_BYTES));
    }
    if let Some(interface) = plan.region_library_interface {
        note.push_str(&format!("region-library-interface={interface}\nregion-runtime-bytes=65536\nregion-control-bytes=64\nregion-history-bytes=8\nregion-retired-stack-bytes=4096\nregion-private-prefix-bytes=4096\n"));
    }
    note
}

fn elf_note(description: &[u8]) -> Result<Vec<u8>, CompileError> {
    const NAME: &[u8] = b"TOPAL\0";
    const PROVIDER_NOTE_TYPE: u32 = 1;

    let description_size = u32::try_from(description.len()).map_err(|_| {
        CompileError::Tool("systems provider ELF note exceeds the ELF32 field limit".into())
    })?;
    let mut note = Vec::with_capacity(
        12 + aligned_note_size(NAME.len()) + aligned_note_size(description.len()),
    );
    note.extend_from_slice(&6_u32.to_le_bytes());
    note.extend_from_slice(&description_size.to_le_bytes());
    note.extend_from_slice(&PROVIDER_NOTE_TYPE.to_le_bytes());
    append_note_field(&mut note, NAME);
    append_note_field(&mut note, description);
    Ok(note)
}

const fn aligned_note_size(size: usize) -> usize {
    (size + 3) & !3
}

fn append_note_field(note: &mut Vec<u8>, field: &[u8]) {
    note.extend_from_slice(field);
    note.resize(
        note.len() + (aligned_note_size(field.len()) - field.len()),
        0,
    );
}
