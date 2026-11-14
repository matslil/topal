//! Private boot binding from the same adapter source, not publication authority.
use super::*;

pub(crate) fn inspect(kernel: &[u8], image: &[u8]) -> Result<serde_json::Value, CompileError> {
    let fresh = unpublished_component_fixture(kernel)?;
    if fresh != image {
        return Err(CompileError::Tool(
            "boot adapter differs from fresh kernel reconstruction".into(),
        ));
    }
    let file = object::File::parse(kernel).map_err(|e| CompileError::Tool(e.to_string()))?;
    let loads = collect_load_segments(&file)?;
    let handlers = [
        X86_SYSTEMS_KERNEL_ENTRY,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
        X86_SYSTEMS_DEADLINE_ENTRY,
        X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
    ]
    .into_iter()
    .map(|name| {
        required_executable_symbol(&file, &loads, name)
            .map(|address| serde_json::json!({"symbol":name,"address":address}))
    })
    .collect::<Result<Vec<_>, _>>()?;
    Ok(serde_json::json!({
        "schema":"topal.kernel-image.boot-binding.x86_64/1",
        "scope":"Fresh complete private adapter/kernel reconstruction and retained-role identities; no activation, CPU lifetime or production publication",
        "kernel_sha256":sha256(kernel),"boot_image_sha256":sha256(image),
        "adapter_revision":X86_LINUX_BOOT_ADAPTER_REVISION,"handlers":handlers,
        "roles":[
            {"role":"closed-handoff-code","start":X86_PROTECTED_PAYLOAD_ADDRESS,"end":PAGE_TABLE_PML4_ADDRESS},
            {"role":"retained-entry-tables","start":PAGE_TABLE_PML4_ADDRESS,"end":GDT_ADDRESS},
            {"role":"retained-gdt","start":GDT_ADDRESS,"end":IDT_ADDRESS},
            {"role":"retained-idt","start":IDT_ADDRESS,"end":IDT_DESCRIPTOR_ADDRESS},
            {"role":"retained-idt-pointer","start":IDT_DESCRIPTOR_ADDRESS,"end":LOCAL_APIC_PD_ADDRESS},
            {"role":"retained-mmio-table","start":LOCAL_APIC_PD_ADDRESS,"end":TRANSITION_RESERVED_END},
            {"role":"bootstrap-stack","start":TRANSITION_RESERVED_END,"end":X86_INITIAL_STACK_TOP}
        ],
        "entry_root":PAGE_TABLE_PML4_ADDRESS,"entry_table_pages":[PAGE_TABLE_PML4_ADDRESS,PAGE_TABLE_PDPT_ADDRESS,PAGE_TABLE_PD_ADDRESS,LOCAL_APIC_PD_ADDRESS],
        "direct_window":[0,X86_INITIAL_IDENTITY_LIMIT],"direct_cache_index":0,
        "mmio_windows":[[HPET_MMIO_LEAF_ADDRESS,HPET_MMIO_LEAF_ADDRESS+0x200000],
                         [LOCAL_APIC_PAGE_ADDRESS,LOCAL_APIC_PAGE_ADDRESS+0x200000]],"mmio_cache_index":3,
        "variable_mtrr_bound":16,"observation_words":128,
        "firmware_processors":{"discovery":"legacy-acpi","root_entries":64,
            "madt_bytes":4096,"participants":16,"retained_bytes":8192,
            "inventory_only":true},
        "protected_context":false,"production_qualified":false,
    }))
}

pub(crate) fn llvm() -> Result<String, CompileError> {
    let mut boot = vec![0; (TRANSITION_RESERVED_END - X86_PROTECTED_PAYLOAD_ADDRESS) as usize];
    install_page_tables(&mut boot)?;
    install_gdt(&mut boot)?;
    let pages = [
        PAGE_TABLE_PML4_ADDRESS,
        PAGE_TABLE_PDPT_ADDRESS,
        PAGE_TABLE_PD_ADDRESS,
        LOCAL_APIC_PD_ADDRESS,
    ];
    let words = pages
        .into_iter()
        .flat_map(|base| {
            let at = (base - X86_PROTECTED_PAYLOAD_ADDRESS) as usize;
            boot[at..at + 4096]
                .chunks_exact(8)
                .map(|v| u64::from_le_bytes(v.try_into().unwrap()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let gdt = (GDT_ADDRESS - X86_PROTECTED_PAYLOAD_ADDRESS) as usize;
    let gdt = boot[gdt..gdt + 48]
        .chunks_exact(8)
        .map(|v| u64::from_le_bytes(v.try_into().unwrap()))
        .collect::<Vec<_>>();
    let transition = transition_code(0)?;
    let immediate = transition.len() - 10;
    if transition.get(immediate - 2..immediate) != Some(&[0x48, 0xb8])
        || !transition.ends_with(&[0xff, 0xe0])
    {
        return Err(CompileError::Tool(
            "unsupported boot handoff fixup shape".into(),
        ));
    }
    let qwords = |v: &[u64]| {
        v.iter()
            .map(|n| format!("i64 {n}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let mut header = format!(
        "@topal_x86_image_boot_observation_v1 = global [128 x i64] zeroinitializer, align 8\n\
@k.boot.table_words = private constant [2048 x i64] [{}], align 8\n\
@k.boot.table_addresses = private constant [4 x i64] [{}], align 8\n\
@k.boot.gdt = private constant [6 x i64] [{}], align 8\n\
@k.boot.transition_bytes = private constant [{} x i8] [{}], align 1\n\
@k.boot.fixed_msrs = private constant [11 x i32] [i32 592,i32 600,i32 601,i32 616,i32 617,i32 618,i32 619,i32 620,i32 621,i32 622,i32 623], align 4\n\
@topal_x86_frame_handoff_v1 = external global i8\n",
        qwords(&words),
        qwords(&pages),
        qwords(&gdt),
        transition.len(),
        transition
            .iter()
            .map(|n| format!("i8 {n}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    for symbol in [
        X86_SYSTEMS_KERNEL_ENTRY,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
        X86_SYSTEMS_DEADLINE_ENTRY,
        X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
    ] {
        header.push_str(&format!("declare void @{symbol}()\n"));
    }
    let body = (include_str!("runtime/kernel_image_boot.ll").to_owned()
        + include_str!("runtime/kernel_boot_firmware.ll")
        + include_str!("runtime/kernel_boot_processors.ll"))
    .replace(
        ") {",
        ") nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {",
    )
    .replace("{{TRANSITION_BYTES}}", &transition.len().to_string())
    .replace("{{ENTRY_FIXUP}}", &immediate.to_string())
    .replace("{{ENTRY_FIXUP_END}}", &(immediate + 8).to_string())
    .replace("{{KERNEL_ENTRY}}", X86_SYSTEMS_KERNEL_ENTRY)
    .replace("{{DEBUG_ENTRY}}", X86_SYSTEMS_DEBUG_BREAK_ENTRY)
    .replace("{{LOCAL_ENTRY}}", X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY)
    .replace("{{DEADLINE_ENTRY}}", X86_SYSTEMS_DEADLINE_ENTRY)
    .replace("{{FAULT_ENTRY}}", X86_SYSTEMS_USER_COPY_FAULT_SYMBOL);
    if body.contains("{{") {
        return Err(CompileError::Tool("unbound boot contract".into()));
    }
    Ok(header + &body)
}
