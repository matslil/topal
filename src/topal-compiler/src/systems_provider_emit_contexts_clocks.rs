fn initial_utsname_bytes() -> [u8; 390] {
    let mut bytes = [0_u8; 390];
    for (index, value) in [
        "Linux",
        "topal",
        "7.2.9-topal",
        "#1 Topal",
        "x86_64",
        "(none)",
    ]
    .into_iter()
    .enumerate()
    {
        let start = index * 65;
        bytes[start..start + value.len()].copy_from_slice(value.as_bytes());
    }
    bytes
}

fn user_copy_page_fault_entry() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "fault-copy-to");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x84, "fault-copy-from");
    code.jump("fault-fatal");
    code.bind("fault-copy-to")?;
    code.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x02]); // non-present supervisor write
    code.jump_if(0x85, "fault-fatal");
    code.jump("fault-site");
    code.bind("fault-copy-from")?;
    code.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x00]); // non-present supervisor read
    code.jump_if(0x85, "fault-fatal");
    code.bind("fault-site")?;
    code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x08]);
    code.bytes(&[0x65, 0x48, 0x3b, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fault-fatal");
    code.bytes(&[0x0f, 0x20, 0xd0]); // CR2
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0060_2000_u32.to_le_bytes());
    code.jump_if(0x85, "fault-fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x89, 0x44, 0x24, 0x08]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x08, 0x48, 0xcf]); // discard error; iretq
    code.bind("fault-fatal")?;
    code.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    code.finish()
}

fn context_create() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // provider-private context state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // stack base
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc6, 0x0f]); // 16-byte aligned stack
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x40, 0x00, 0x00]); // 16 KiB
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xc9]); // typed entry address
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x00]); // fresh state
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0x77, 0x10]); // retain stack base
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x01, 0xd0]); // stack end
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x83, 0xe8, 0x40]); // initial saved RSP
    code.bytes(&[0x45, 0x31, 0xc0]); // zero callee-saved register images
    for displacement in [0_u8, 8, 16, 24, 32, 40] {
        code.bytes(&[0x4c, 0x89, 0x40, displacement]);
    }
    code.bytes(&[0x48, 0x89, 0x48, 0x30]); // synthetic return enters worker
    code.bytes(&[0x48, 0x89, 0x47, 0x08]); // suspended worker RSP
    code.bytes(&[0xc6, 0x47, 0x18, 0x01]); // suspended
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_transfer() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x01]); // suspended
    code.jump_if(0x84, "select-suspended");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active cooperative caller
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x01]); // worker becomes suspended
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x67, 0x08]); // suspend worker RSP
    code.bytes(&[0x48, 0x8b, 0x27]); // select dispatcher RSP
    code.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    code.bind("select-suspended")?;
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x02]); // selected worker becomes active
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x27]); // suspend dispatcher RSP
    code.bytes(&[0x48, 0x8b, 0x67, 0x08]); // select worker RSP
    code.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_await_deadline_preemption() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active worker
    code.jump_if(0x85, "fail");
    code.bytes(&[0xfb, 0xf4, 0xfa]); // sti; hlt; restore sealed IF=0 state
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // restored only by redispatch
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0xfa, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_preempt_current() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active worker
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x01]); // interrupted worker becomes suspended
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x67, 0x08]); // retain interrupt-handler RSP
    code.bytes(&[0x48, 0x8b, 0x27]); // restore exact dispatcher RSP
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]); // complete transfer call
    code.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_retire() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x03]); // completed
    code.bytes(&[0x48, 0x8b, 0x27]); // resume exact caller RSP
    code.bytes(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_reclaim() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x03]); // completed
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xc7, 0x07, 0, 0, 0, 0]); // consume caller RSP
    code.bytes(&[0x48, 0xc7, 0x47, 0x08, 0, 0, 0, 0]); // consume worker RSP
    code.bytes(&[0xc6, 0x47, 0x18, 0x04]); // reclaimed
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn append_critical_functions(object: &mut Object<'_>, text: object::write::SectionId) {
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
        &critical_enter(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
        &critical_restore(),
    );
}

fn append_atomic_functions(object: &mut Object<'_>, text: object::write::SectionId) {
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
        &atomic_word_create(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
        &atomic_word_compare_exchange(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
        &atomic_word_load_acquire(),
    );
}

fn append_local_notification_functions(object: &mut Object<'_>, text: object::write::SectionId) {
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
        &local_notification_send(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
        &local_notification_wait(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
        &local_notification_complete(),
    );
}

fn local_notification_send() -> Vec<u8> {
    let mut code = Vec::new();
    code.extend_from_slice(&[0x9c, 0x58]); // pushfq; pop rax
    code.extend_from_slice(&[0xa9, 0x00, 0x02, 0x00, 0x00]); // test eax, IF
    code.extend_from_slice(&[0x75, 0x41]); // initial profile requires IF clear
    code.extend_from_slice(&[0xc6, 0x07, 0x00]); // completion flag = 0
    code.extend_from_slice(&[0xb9, 0x1b, 0x00, 0x00, 0x00]); // IA32_APIC_BASE
    code.extend_from_slice(&[0x0f, 0x32]); // rdmsr
    code.extend_from_slice(&[0xa9, 0x00, 0x04, 0x00, 0x00]); // reject x2APIC mode
    code.extend_from_slice(&[0x75, 0x30]);
    code.extend_from_slice(&[0x0d, 0x00, 0x08, 0x00, 0x00]); // global APIC enable
    code.extend_from_slice(&[0x0f, 0x30]); // wrmsr
    code.extend_from_slice(&[0x48, 0xba]); // mov rdx, local APIC SVR
    code.extend_from_slice(&0xfee0_00f0_u64.to_le_bytes());
    code.extend_from_slice(&[0x8b, 0x02]); // mov eax, [rdx]
    code.extend_from_slice(&[0x0d, 0x00, 0x01, 0x00, 0x00]); // software enable
    code.extend_from_slice(&[0x89, 0x02]); // mov [rdx], eax
    code.extend_from_slice(&[0x48, 0xba]); // mov rdx, local APIC ICR low
    code.extend_from_slice(&0xfee0_0300_u64.to_le_bytes());
    code.extend_from_slice(&[0xc7, 0x02]); // fixed, assert, edge, self shorthand
    code.extend_from_slice(&0x0004_40f1_u32.to_le_bytes());
    code.extend_from_slice(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.extend_from_slice(&[0x31, 0xc0, 0xc3]); // fail
    code
}

fn local_notification_wait() -> Vec<u8> {
    vec![
        0xfb, // sti; the interrupt shadow covers the following hlt
        0xf4, // hlt until an admitted external interrupt
        0xfa, // restore the sealed prior disabled state
        0x80, 0x3f, 0x01, // cmp byte ptr [rdi], 1
        0x75, 0xf8, // wait again after an unrelated wake
        0xc6, 0x07, 0x00, // consume completion flag
        0xb8, 0x01, 0x00, 0x00, 0x00, // success
        0xc3,
    ]
}

fn local_notification_complete() -> Vec<u8> {
    let mut code = Vec::new();
    code.extend_from_slice(&[0x48, 0xb8]); // mov rax, local APIC EOI
    code.extend_from_slice(&0xfee0_00b0_u64.to_le_bytes());
    code.extend_from_slice(&[0xc7, 0x00, 0x00, 0x00, 0x00, 0x00]); // EOI = 0
    code.extend_from_slice(&[0xc6, 0x07, 0x01, 0xc3]); // completion flag = 1; ret
    code
}

fn monotonic_clock_now() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // test provider-private state address
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc7, 0x07]); // test dil, 7
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xb8]); // r8 = HPET MMIO base
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes());
    code.bytes(&[0x49, 0x8b, 0x00]); // general capabilities
    code.bytes(&[0xa9, 0x00, 0x20, 0x00, 0x00]); // require a 64-bit main counter
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0x81, 0xfa, 0x80, 0x96, 0x98, 0x00]); // Q35 HPET: 10 ns period
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x40, 0x10]); // general configuration
    code.bytes(&[0x48, 0x83, 0xc8, 0x01]); // enable main counter
    code.bytes(&[0x49, 0x89, 0x40, 0x10]);
    code.bytes(&[0x49, 0x8b, 0x80, 0xf0, 0x00, 0x00, 0x00]); // raw counter
    code.bytes(&[0x80, 0x7f, 0x10, 0x00]); // initialized?
    code.jump_if(0x85, "subsequent");
    code.bytes(&[0x48, 0x89, 0x07]); // retain raw low word
    code.bytes(&[0x31, 0xd2]); // first epoch is zero
    code.bytes(&[0x48, 0x89, 0x57, 0x08]);
    code.bytes(&[0xc6, 0x47, 0x10, 0x01]);
    code.jump("success");

    code.bind("subsequent")?;
    code.bytes(&[0x48, 0x8b, 0x57, 0x08]); // retained wrap epoch
    code.bytes(&[0x48, 0x3b, 0x07]); // unsigned raw >= previous raw
    code.jump_if(0x83, "retain");
    code.bytes(&[0x4c, 0x8b, 0x0f]); // a decrease is only a qualified boundary wrap
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0xc000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x39, 0xd1]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x4000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x48, 0x83, 0xc2, 0x01]); // extend a 64-bit wrap
    code.jump_if(0x82, "fail");
    code.bind("retain")?;
    code.bytes(&[0x48, 0x89, 0x07, 0x48, 0x89, 0x57, 0x08]);
    code.bind("success")?;
    code.bytes(&[0xb9, 0x01, 0x00, 0x00, 0x00, 0xc3]); // rcx = success
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc9, 0xc3]);
    code.finish()
}

fn deadline_after() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // private clock/deadline state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc7, 0x07]); // 8-byte aligned
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x7f, 0x10, 0x01]); // clock initialized
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x07]); // last accepted raw counter
    code.bytes(&[0x48, 0x8b, 0x57, 0x08]); // wrap epoch
    code.bytes(&[0x48, 0x05]);
    code.bytes(&100_000_u32.to_le_bytes()); // 1 ms at the validated 10 ns period
    code.bytes(&[0x48, 0x83, 0xd2, 0x00]); // extend carry into epoch
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x89, 0x47, 0x18]); // scheduled raw
    code.bytes(&[0x48, 0x89, 0x57, 0x20]); // scheduled epoch
    code.bytes(&[0xc6, 0x47, 0x28, 0x01]); // constructed
    code.bytes(&[0xc6, 0x47, 0x29, 0x00]); // not completed
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn deadline_arm() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc7, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x7f, 0x28, 0x01]); // exactly one constructed event
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58]); // require the sealed prior IF=0 state
    code.bytes(&[0xa9, 0x00, 0x02, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb0, 0xff, 0xe6, 0x21, 0xe6, 0xa1]); // mask both legacy PICs
    code.bytes(&[0x49, 0xb8]);
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes()); // HPET
    code.bytes(&[0x49, 0x8b, 0x00]); // capabilities
    code.bytes(&[0xa9, 0x00, 0x20, 0x00, 0x00]); // 64-bit main counter
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0x81, 0xfa, 0x80, 0x96, 0x98, 0x00]); // 10 ns period
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x90, 0x00, 0x01, 0x00, 0x00]); // timer 0 config/cap
    code.bytes(&[0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0xf7, 0xc2, 0x04, 0x00, 0x00, 0x00]); // route 2 capability
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x8b, 0x47, 0x20]); // scheduled epoch
    code.bytes(&[0x48, 0x3b, 0x47, 0x08]); // same current epoch in initial slice
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x88, 0xf0, 0x00, 0x00, 0x00]); // current raw
    code.bytes(&[0x48, 0x8b, 0x47, 0x18]); // scheduled raw
    code.bytes(&[0x48, 0x39, 0xc1]);
    code.jump_if(0x83, "immediate"); // current >= scheduled

    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x0000_0000_fec0_0000_u64.to_le_bytes()); // I/O APIC
    code.bytes(&[0x41, 0xc7, 0x01, 0x14, 0x00, 0x00, 0x00]); // redirection low 2
    code.bytes(&[0x41, 0xc7, 0x41, 0x10]);
    code.bytes(&0x0001_00f2_u32.to_le_bytes()); // masked, fixed vector f2
    code.bytes(&[0x41, 0xc7, 0x01, 0x15, 0x00, 0x00, 0x00]); // redirection high 2
    code.bytes(&[0x41, 0xc7, 0x41, 0x10, 0x00, 0x00, 0x00, 0x00]); // APIC ID 0
    code.bytes(&[0x41, 0xc7, 0x80, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&0x0000_0400_u32.to_le_bytes()); // edge, one-shot, route 2, disabled
    code.bytes(&[0x49, 0x89, 0x80, 0x08, 0x01, 0x00, 0x00]); // comparator
    code.bytes(&[0x49, 0x8b, 0x50, 0x10]);
    code.bytes(&[0x48, 0x83, 0xca, 0x01]); // enable HPET globally
    code.bytes(&[0x49, 0x89, 0x50, 0x10]);
    code.bytes(&[0x41, 0xc7, 0x01, 0x14, 0x00, 0x00, 0x00]);
    code.bytes(&[0x41, 0xc7, 0x41, 0x10]);
    code.bytes(&0x0000_00f2_u32.to_le_bytes()); // unmask route
    code.bytes(&[0x41, 0xc7, 0x80, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&0x0000_0404_u32.to_le_bytes()); // enable timer interrupt
    code.jump("armed");

    code.bind("immediate")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fee0_0300_u64.to_le_bytes()); // local APIC ICR low
    code.bytes(&[0xc7, 0x00]);
    code.bytes(&0x0004_40f2_u32.to_le_bytes()); // fixed self notification

    code.bind("armed")?;
    code.bytes(&[0xc6, 0x47, 0x29, 0x00]);
    code.bytes(&[0xc6, 0x47, 0x28, 0x02]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn deadline_wait() -> Vec<u8> {
    vec![
        0xfb, 0xf4, 0xfa, // sti; hlt; restore sealed prior IF=0 state
        0x80, 0x7f, 0x29, 0x01, // completion flag
        0x75, 0xf7, // unrelated wake: wait again
        0xc6, 0x47, 0x29, 0x00, // consume completion flag
        0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3,
    ]
}

fn deadline_complete() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x28, 0x02]); // armed
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xb8]);
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes());
    code.bytes(&[0x49, 0x8b, 0x80, 0xf0, 0x00, 0x00, 0x00]); // observed raw
    code.bytes(&[0x48, 0x3b, 0x47, 0x18]); // no earlier than scheduled
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x89, 0x47, 0x30]); // retain observed raw
    // Close route 2 before source disable/clear and local EOI. A notification
    // already queued locally is handled by the generated entry filter.
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0xfec0_0000_u64.to_le_bytes());
    code.bytes(&[0x41, 0xc7, 0x01, 0x14, 0, 0, 0]);
    code.bytes(&[0x41, 0xc7, 0x41, 0x10]);
    code.bytes(&0x0001_00f2_u32.to_le_bytes());
    code.bytes(&[0x41, 0x8b, 0x41, 0x10]); // complete route publication
    code.bytes(&[0x41, 0x81, 0xa0, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&0xffff_fffb_u32.to_le_bytes()); // disable timer 0 interrupt
    code.bytes(&[0x41, 0xc7, 0x40, 0x20, 0x01, 0x00, 0x00, 0x00]); // clear status
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fee0_00b0_u64.to_le_bytes());
    code.bytes(&[0xc7, 0x00, 0x00, 0x00, 0x00, 0x00]); // local APIC EOI
    code.bytes(&[0xc6, 0x47, 0x28, 0x03]); // completed
    code.bytes(&[0xc6, 0x47, 0x29, 0x01]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn atomic_word_create() -> Vec<u8> {
    vec![
        0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, // test rdi, 7
        0x75, 0x09, // jnz failure
        0x48, 0x89, 0x37, // mov [rdi], rsi
        0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn atomic_word_compare_exchange() -> Vec<u8> {
    vec![
        0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, // test rdi, 7
        0x75, 0x0f, // jnz failure
        0x48, 0x89, 0xf0, // mov rax, rsi
        0xf0, 0x48, 0x0f, 0xb1, 0x17, // lock cmpxchg [rdi], rdx
        0x0f, 0x94, 0xc0, // sete al
        0x0f, 0xb6, 0xc0, // movzx eax, al
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn atomic_word_load_acquire() -> Vec<u8> {
    vec![
        0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, // test rdi, 7
        0x75, 0x04, // jnz failure
        0x48, 0x8b, 0x07, // mov rax, [rdi]
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn critical_enter() -> Vec<u8> {
    vec![
        0x9c, // pushfq
        0x58, // pop rax -- opaque exact prior flags token
        0xfa, // cli
        0xc3, // ret
    ]
}

fn critical_restore() -> Vec<u8> {
    vec![
        0x40, 0xf6, 0xc7, 0x02, // test dil, 2 -- validate architectural fixed bit
        0x74, 0x16, // jz failure
        0xf7, 0xc7, 0x00, 0x02, 0x00, 0x00, // test edi, RFLAGS.IF
        0x74, 0x07, // jz restore-disabled
        0xfb, // sti
        0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1
        0xc3, // ret
        0xfa, // restore-disabled: cli
        0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}
