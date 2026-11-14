#[derive(Clone, Copy)]
enum PciRegionGatewayOperation {
    ReadEquals,
    ReadEventuallyEquals,
    Write,
}

fn pci_region_configuration_gateway(
    operation: PciRegionGatewayOperation,
) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x89, 0xd1]); // preserve target offset in r9d
    code.bytes(&[0x41, 0x89, 0xfa]); // preserve configuration selector in r10d

    for (selector_delta, value) in [(4_u8, 0_u8), (8, 1), (12, 2)] {
        code.bytes(&[0x41, 0x8d, 0x42, selector_delta]); // selector in eax
        code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]); // select through 0xcf8
        code.bytes(&[0x66, 0xba, 0xfc, 0x0c]); // configuration data port
        match value {
            0 => code.bytes(&[0x89, 0xf0]),       // BAR index from esi
            1 => code.bytes(&[0x44, 0x89, 0xc8]), // target offset from r9d
            2 => code.bytes(&[0x89, 0xc8]),       // width from ecx
            _ => unreachable!(),
        }
        code.bytes(&[0xef]);
    }

    code.bytes(&[0x41, 0x8d, 0x42, 0x10]); // configuration data selector
    code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c]);
    if matches!(operation, PciRegionGatewayOperation::ReadEventuallyEquals) {
        code.bytes(&[0x41, 0xbb]); // bounded attempts in r11d
        code.bytes(&1_000_000_u32.to_le_bytes());
    }
    code.bytes(&[0x83, 0xf9, 0x01]); // exact byte access
    code.jump_if(0x84, "byte");
    code.bytes(&[0x83, 0xf9, 0x02]); // exact word access
    code.jump_if(0x84, "word");
    code.bytes(&[0x83, 0xf9, 0x04]); // exact dword access
    code.jump_if(0x85, "fail");
    match operation {
        PciRegionGatewayOperation::ReadEquals => {
            code.bytes(&[0xed, 0x44, 0x39, 0xc0]); // in eax, dx; cmp eax, r8d
            code.jump_if(0x85, "fail");
        }
        PciRegionGatewayOperation::ReadEventuallyEquals => {
            code.bind("dword-retry")?;
            code.bytes(&[0xed, 0x44, 0x39, 0xc0]);
            code.jump_if(0x84, "success");
            code.bytes(&[0x41, 0xff, 0xcb]); // dec r11d
            code.jump_if(0x85, "dword-retry");
            code.jump("fail");
        }
        PciRegionGatewayOperation::Write => {
            code.bytes(&[0x44, 0x89, 0xc0, 0xef]); // mov eax, r8d; out dx, eax
        }
    }
    code.jump("success");

    code.bind("word")?;
    match operation {
        PciRegionGatewayOperation::ReadEquals => {
            code.bytes(&[0x66, 0xed, 0x66, 0x44, 0x39, 0xc0]); // in ax, dx; cmp ax, r8w
            code.jump_if(0x85, "fail");
        }
        PciRegionGatewayOperation::ReadEventuallyEquals => {
            code.bind("word-retry")?;
            code.bytes(&[0x66, 0xed, 0x66, 0x44, 0x39, 0xc0]);
            code.jump_if(0x84, "success");
            code.bytes(&[0x41, 0xff, 0xcb]);
            code.jump_if(0x85, "word-retry");
            code.jump("fail");
        }
        PciRegionGatewayOperation::Write => {
            code.bytes(&[0x44, 0x89, 0xc0, 0x66, 0xef]); // mov eax, r8d; out dx, ax
        }
    }
    code.jump("success");

    code.bind("byte")?;
    match operation {
        PciRegionGatewayOperation::ReadEquals => {
            code.bytes(&[0xec, 0x44, 0x38, 0xc0]); // in al, dx; cmp al, r8b
            code.jump_if(0x85, "fail");
        }
        PciRegionGatewayOperation::ReadEventuallyEquals => {
            code.bind("byte-retry")?;
            code.bytes(&[0xec, 0x44, 0x38, 0xc0]);
            code.jump_if(0x84, "success");
            code.bytes(&[0x41, 0xff, 0xcb]);
            code.jump_if(0x85, "byte-retry");
            code.jump("fail");
        }
        PciRegionGatewayOperation::Write => {
            code.bytes(&[0x44, 0x89, 0xc0, 0xee]); // mov eax, r8d; out dx, al
        }
    }

    code.bind("success")?;
    for selector_delta in [4_u8, 8, 12] {
        code.bytes(&[0x41, 0x8d, 0x42, selector_delta]);
        code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]);
        code.bytes(&[0x66, 0xba, 0xfc, 0x0c]);
        code.bytes(&[0x31, 0xc0, 0xef, 0xed, 0x85, 0xc0]);
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn pci_configuration_update_word() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x89, 0xd1]); // preserve expected value in r9d
    code.bytes(&[0x41, 0x89, 0xca]); // preserve desired value in r10d
    code.bytes(&[0x89, 0xf8]); // selector in eax
    code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]); // select through 0xcf8
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c]); // base configuration data port
    code.bytes(&[0x66, 0x44, 0x01, 0xc2]); // add word-within-dword offset
    code.bytes(&[0x66, 0xed, 0x0f, 0xb7, 0xc0]); // read command word
    code.bytes(&[0x41, 0x89, 0xc3]); // preserve current word in r11d
    code.bytes(&[0x21, 0xf0, 0x44, 0x39, 0xc8]); // mask and compare pre-state
    code.jump_if(0x85, "fail");
    code.bytes(&[0xf7, 0xd6, 0x41, 0x21, 0xf3]); // retain unmasked current bits
    code.bytes(&[0x45, 0x09, 0xd3, 0x44, 0x89, 0xd8]); // apply desired bits
    code.bytes(&[0x66, 0xef]); // exact word write
    code.bytes(&[0xf7, 0xd6]); // restore mask
    code.bytes(&[0x66, 0xed, 0x0f, 0xb7, 0xc0]); // exact word readback
    code.bytes(&[0x21, 0xf0, 0x44, 0x39, 0xd0]); // mask and compare post-state
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    code.jump("return");
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0]);
    code.bind("return")?;
    code.bytes(&[0xc3]);
    code.finish()
}

fn dma_buffer_create() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xff, 0x01]); // cmp edi, 1
    code.jump_if(0x85, "second");
    code.bytes(&[0x83, 0xfe, 0x01]); // cmp esi, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // cmp edx, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xf9, 0x00, 0x10, 0x00, 0x00]); // cmp ecx, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x81, 0xf8, 0x00, 0x10, 0x00, 0x00]); // cmp r8d, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf9, 0x01]); // packed bidirectional/control metadata
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("second")?;
    code.bytes(&[0x83, 0xff, 0x02]); // cmp edi, 2
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfe, 0x01]); // cmp esi, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // cmp edx, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xf9, 0x00, 0x10, 0x00, 0x00]); // cmp ecx, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x81, 0xf8, 0x00, 0x10, 0x00, 0x00]); // cmp r8d, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf9, 0x12]); // packed device-write/payload metadata
    code.jump_if(0x85, "fail");
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    code.bytes(&[0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_bind_reference() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x83, 0xf8, 0x08]); // cmp r8d, 8
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0xf8, 0x0f, 0x00, 0x00]); // cmp rsi, 4088
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x07, 0x00, 0x00, 0x00]); // test rsi, 7
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0xff, 0x0f, 0x00, 0x00]); // cmp rcx, 4095
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x01, 0xca]); // add rdx, rcx
    code.bytes(&[0x48, 0x89, 0x14, 0x37]); // mov [rdi+rsi], rdx
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_store_integer() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x02]); // cmp edx, 2
    code.jump_if(0x85, "dword");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfe, 0x0f, 0x00, 0x00]); // cmp rsi, 4094
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x01, 0x00, 0x00, 0x00]); // test rsi, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x66, 0x89, 0x0c, 0x37]); // mov [rdi+rsi], cx
    code.jump("success");
    code.bind("dword")?;
    code.bytes(&[0x83, 0xfa, 0x04]); // cmp edx, 4
    code.jump_if(0x85, "qword");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfc, 0x0f, 0x00, 0x00]); // cmp rsi, 4092
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x03, 0x00, 0x00, 0x00]); // test rsi, 3
    code.jump_if(0x85, "fail");
    code.bytes(&[0x89, 0x0c, 0x37]); // mov [rdi+rsi], ecx
    code.jump("success");
    code.bind("qword")?;
    code.bytes(&[0x83, 0xfa, 0x08]); // cmp edx, 8
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0xf8, 0x0f, 0x00, 0x00]); // cmp rsi, 4088
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x07, 0x00, 0x00, 0x00]); // test rsi, 7
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0x0c, 0x37]); // mov [rdi+rsi], rcx
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_prepare_device() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]); // page-aligned storage
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // device identity one
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xf9, 0x01]); // translation domain one
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfe, 0x02]); // payload identity
    code.jump_if(0x85, "control");
    code.bytes(&[0x41, 0x83, 0xf8, 0x12]); // device-write/payload metadata
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("control")?;
    code.bytes(&[0x83, 0xfe, 0x01]); // control identity
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf8, 0x01]); // bidirectional/control metadata
    code.jump_if(0x85, "fail");
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_load_integer() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x02]);
    code.jump_if(0x85, "dword");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfe, 0x0f, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x01, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb7, 0x04, 0x37, 0x39, 0xc8]);
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("dword")?;
    code.bytes(&[0x83, 0xfa, 0x04]);
    code.jump_if(0x85, "qword");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfc, 0x0f, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x03, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x8b, 0x04, 0x37, 0x39, 0xc8]);
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("qword")?;
    code.bytes(&[0x83, 0xfa, 0x08]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0xf8, 0x0f, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x07, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x04, 0x37, 0x48, 0x39, 0xc8]);
    code.jump_if(0x85, "fail");
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_capture_dma_sample() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x01]); // sample identity one
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xf9, 0x02]); // payload buffer identity two
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf8, 0x20]); // exact 32-byte range
    code.jump_if(0x85, "fail");
    code.bytes(&[0x45, 0x85, 0xc9]); // explicitly uncredited
    code.jump_if(0x85, "fail");
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x07]);
            code.bytes(&[0x48, 0x89, 0x06]);
            code.bytes(&[0x48, 0x3b, 0x06]);
        } else {
            code.bytes(&[0x48, 0x8b, 0x47, offset]);
            code.bytes(&[0x48, 0x89, 0x46, offset]);
            code.bytes(&[0x48, 0x3b, 0x46, offset]);
        }
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

const BLAKE2S_IV: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const BLAKE2S_SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

fn emit_stack_dword_load(code: &mut X86FunctionEncoder, register: u8, offset: usize) {
    let displacement = u8::try_from(offset).expect("BLAKE2s stack offset fits disp8");
    match register {
        0 => code.bytes(&[0x8b, 0x44, 0x24, displacement]), // eax
        1 => code.bytes(&[0x8b, 0x4c, 0x24, displacement]), // ecx
        2 => code.bytes(&[0x8b, 0x54, 0x24, displacement]), // edx
        8 => code.bytes(&[0x44, 0x8b, 0x44, 0x24, displacement]), // r8d
        _ => unreachable!(),
    }
}

fn emit_stack_dword_store(code: &mut X86FunctionEncoder, register: u8, offset: usize) {
    let displacement = u8::try_from(offset).expect("BLAKE2s stack offset fits disp8");
    match register {
        0 => code.bytes(&[0x89, 0x44, 0x24, displacement]), // eax
        1 => code.bytes(&[0x89, 0x4c, 0x24, displacement]), // ecx
        2 => code.bytes(&[0x89, 0x54, 0x24, displacement]), // edx
        8 => code.bytes(&[0x44, 0x89, 0x44, 0x24, displacement]), // r8d
        _ => unreachable!(),
    }
}

fn emit_blake2s_g(code: &mut X86FunctionEncoder, lanes: [usize; 4], message_words: [usize; 2]) {
    let [a, b, c, d] = lanes.map(|lane| lane * 4);
    let [x, y] = message_words.map(|word| 64 + word * 4);
    emit_stack_dword_load(code, 0, a);
    emit_stack_dword_load(code, 1, b);
    emit_stack_dword_load(code, 2, c);
    emit_stack_dword_load(code, 8, d);
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x03, 0x44, 0x24, u8::try_from(x).unwrap()]); // a += x
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc8, 16]); // d = ror(d ^ a, 16)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc9, 12]); // b = ror(b ^ c, 12)
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x03, 0x44, 0x24, u8::try_from(y).unwrap()]); // a += y
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc8, 8]); // d = ror(d ^ a, 8)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc9, 7]); // b = ror(b ^ c, 7)
    emit_stack_dword_store(code, 0, a);
    emit_stack_dword_store(code, 1, b);
    emit_stack_dword_store(code, 2, c);
    emit_stack_dword_store(code, 8, d);
}

fn entropy_mix_blake2s_pool_sample() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // sample pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // pool pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]); // storage must be distinct
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xfa, 0x49, 0x89, 0xf3]); // r10=sample, r11=pool
    code.bytes(&[0x48, 0x81, 0xec, 0x80, 0, 0, 0]); // v[16], m[16]

    for word in 0_u8..8 {
        code.bytes(&[0x41, 0x8b, 0x42, word * 4]);
        code.bytes(&[0x89, 0x44, 0x24, 64 + word * 4]);
    }
    for word in 8_u8..16 {
        code.bytes(&[0xc7, 0x44, 0x24, 64 + word * 4, 0, 0, 0, 0]);
    }

    let mut initial = [0_u32; 16];
    initial[..8].copy_from_slice(&BLAKE2S_IV);
    initial[0] ^= 0x0101_0020; // digest length 32, fanout one, depth one
    initial[6] ^= u32::from_le_bytes(*b"Topa");
    initial[7] ^= u32::from_le_bytes(*b"lRNG");
    initial[8..].copy_from_slice(&BLAKE2S_IV);
    initial[12] ^= 32; // final block contains 32 bytes
    initial[14] ^= u32::MAX; // final-block flag
    for (lane, value) in initial.into_iter().enumerate() {
        code.bytes(&[0xc7, 0x44, 0x24, u8::try_from(lane * 4).unwrap()]);
        code.bytes(&value.to_le_bytes());
    }

    for sigma in BLAKE2S_SIGMA {
        for (lanes, pair) in [
            ([0, 4, 8, 12], [sigma[0], sigma[1]]),
            ([1, 5, 9, 13], [sigma[2], sigma[3]]),
            ([2, 6, 10, 14], [sigma[4], sigma[5]]),
            ([3, 7, 11, 15], [sigma[6], sigma[7]]),
            ([0, 5, 10, 15], [sigma[8], sigma[9]]),
            ([1, 6, 11, 12], [sigma[10], sigma[11]]),
            ([2, 7, 8, 13], [sigma[12], sigma[13]]),
            ([3, 4, 9, 14], [sigma[14], sigma[15]]),
        ] {
            emit_blake2s_g(&mut code, lanes, pair);
        }
    }

    let mut hash = BLAKE2S_IV;
    hash[0] ^= 0x0101_0020;
    hash[6] ^= u32::from_le_bytes(*b"Topa");
    hash[7] ^= u32::from_le_bytes(*b"lRNG");
    for (word, value) in hash.into_iter().enumerate() {
        emit_stack_dword_load(&mut code, 0, word * 4);
        code.bytes(&[0x33, 0x44, 0x24, u8::try_from((word + 8) * 4).unwrap()]);
        code.bytes(&[0x35]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x41, 0x89, 0x43, u8::try_from(word * 4).unwrap()]);
    }

    code.bytes(&[0x49, 0x8b, 0x03]);
    for offset in [8_u8, 16, 24] {
        code.bytes(&[0x49, 0x0b, 0x43, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail_stack");

    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x89, 0x42, offset]);
    }
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x0b, 0x42, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_stack");
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fail_stack")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x89, 0x43, offset, 0x49, 0x89, 0x42, offset]);
    }
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_initialize_chacha20_generator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x89, 0x06, 0x48, 0x3b, 0x06]);
        } else {
            code.bytes(&[
                0x48, 0x8b, 0x47, offset, 0x48, 0x89, 0x46, offset, 0x48, 0x3b, 0x46, offset,
            ]);
        }
        code.jump_if(0x85, "fail_wipe");
    }
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x07]);
        } else {
            code.bytes(&[0x48, 0x89, 0x47, offset]);
        }
    }
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x0b, 0x07]);
        } else {
            code.bytes(&[0x48, 0x0b, 0x47, offset]);
        }
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_wipe");
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail_wipe")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x07, 0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x47, offset, 0x48, 0x89, 0x46, offset]);
        }
    }
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn emit_chacha20_quarter_round(code: &mut X86FunctionEncoder, lanes: [usize; 4]) {
    let [a, b, c, d] = lanes.map(|lane| 64 + lane * 4);
    emit_stack_dword_load(code, 0, a);
    emit_stack_dword_load(code, 1, b);
    emit_stack_dword_load(code, 2, c);
    emit_stack_dword_load(code, 8, d);
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc0, 16]); // d = rol(d ^ a, 16)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc1, 12]); // b = rol(b ^ c, 12)
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc0, 8]); // d = rol(d ^ a, 8)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc1, 7]); // b = rol(b ^ c, 7)
    emit_stack_dword_store(code, 0, a);
    emit_stack_dword_store(code, 1, b);
    emit_stack_dword_store(code, 2, c);
    emit_stack_dword_store(code, 8, d);
}

fn entropy_extract_chacha20_generator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // key pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // output pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]); // storage must be distinct
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xfa, 0x49, 0x89, 0xf3]); // r10=key, r11=output
    code.bytes(&[0x48, 0x81, 0xec, 0x80, 0, 0, 0]); // input[16], working[16]

    for (word, value) in [0x6170_7865_u32, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]
        .into_iter()
        .enumerate()
    {
        code.bytes(&[0xc7, 0x44, 0x24, u8::try_from(word * 4).unwrap()]);
        code.bytes(&value.to_le_bytes());
    }
    for word in 0_u8..8 {
        code.bytes(&[0x41, 0x8b, 0x42, word * 4]);
        code.bytes(&[0x89, 0x44, 0x24, 16 + word * 4]);
    }
    for word in 12_u8..16 {
        code.bytes(&[0xc7, 0x44, 0x24, word * 4, 0, 0, 0, 0]);
    }
    for word in 0..16 {
        emit_stack_dword_load(&mut code, 0, word * 4);
        emit_stack_dword_store(&mut code, 0, 64 + word * 4);
    }

    for _ in 0..10 {
        for lanes in [
            [0, 4, 8, 12],
            [1, 5, 9, 13],
            [2, 6, 10, 14],
            [3, 7, 11, 15],
            [0, 5, 10, 15],
            [1, 6, 11, 12],
            [2, 7, 8, 13],
            [3, 4, 9, 14],
        ] {
            emit_chacha20_quarter_round(&mut code, lanes);
        }
    }
    for word in 0..16 {
        emit_stack_dword_load(&mut code, 0, 64 + word * 4);
        code.bytes(&[0x03, 0x44, 0x24, u8::try_from(word * 4).unwrap()]);
        emit_stack_dword_store(&mut code, 0, 64 + word * 4);
    }

    // Replace and verify the persistent key before publishing any output.
    for word in 0_u8..8 {
        emit_stack_dword_load(&mut code, 0, 64 + usize::from(word) * 4);
        code.bytes(&[0x41, 0x89, 0x42, word * 4]);
        code.bytes(&[0x41, 0x3b, 0x42, word * 4]);
        code.jump_if(0x85, "fail_stack");
    }
    for word in 0_u8..8 {
        emit_stack_dword_load(&mut code, 0, 96 + usize::from(word) * 4);
        code.bytes(&[0x41, 0x89, 0x43, word * 4]);
        code.bytes(&[0x41, 0x3b, 0x43, word * 4]);
        code.jump_if(0x85, "fail_stack");
    }

    code.bytes(&[0x31, 0xc0]);
    for offset in (0_u8..128).step_by(8) {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else {
            code.bytes(&[0x48, 0x89, 0x44, 0x24, offset]);
        }
    }
    code.bytes(&[0x48, 0x8b, 0x04, 0x24]);
    for offset in (8_u8..128).step_by(8) {
        code.bytes(&[0x48, 0x0b, 0x44, 0x24, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_stack");
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fail_stack")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x89, 0x42, offset, 0x49, 0x89, 0x43, offset]);
    }
    for offset in (0_u8..128).step_by(8) {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else {
            code.bytes(&[0x48, 0x89, 0x44, 0x24, offset]);
        }
    }
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_deliver_initial_auxv_random() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // prepared address-space state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // affine 32-byte output
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x01]); // prepared probe, not entered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 0]); // not previously delivered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]); // allocator-owned backing
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");

    for offset in [0_u8, 8] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x06]);
        } else {
            code.bytes(&[0x48, 0x8b, 0x46, offset]);
        }
        code.bytes(&[0x48, 0x89, 0x81]);
        code.bytes(&(0x1fe0_u32 + u32::from(offset)).to_le_bytes());
        code.bytes(&[0x48, 0x3b, 0x81]);
        code.bytes(&(0x1fe0_u32 + u32::from(offset)).to_le_bytes());
        code.jump_if(0x85, "fail_wipe");
    }
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x46, offset]);
        }
    }
    code.bytes(&[0x48, 0x8b, 0x06]);
    for offset in [8_u8, 16, 24] {
        code.bytes(&[0x48, 0x0b, 0x46, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_wipe");
    code.bytes(&[0xc6, 0x87, 0xd0, 0, 0, 0, 1]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fail_wipe")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x46, offset]);
        }
    }
    for offset in [0x1fe0_u32, 0x1fe8] {
        code.bytes(&[0x48, 0x89, 0x81]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0xc6, 0x87, 0xd0, 0, 0, 0, 0]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_reserve_initial_user_random() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // prepared address-space state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // affine 32-byte output
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x01]); // prepared probe, not entered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 1]); // AT_RANDOM delivered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xd8, 0, 0, 0, 0]); // no live pointer
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0xb7, 0xd8, 0, 0, 0]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_bind_reusable_user_random() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // prepared address-space state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // live generator key
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]); // fresh service state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xd7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xd6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 1]); // AT_RANDOM delivered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x8f, 0xd8, 0, 0, 0]); // reserved first output
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xd1]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf1]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x0b, 0x02]);
        } else {
            code.bytes(&[0x48, 0x0b, 0x42, offset]);
        }
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0x32]);
    code.bytes(&[0x48, 0x89, 0x4a, 0x08]);
    code.bytes(&[0xc6, 0x42, 0x10, 0x01]); // reserved first output is live
    code.bytes(&[0x48, 0x89, 0x97, 0xd8, 0, 0, 0]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn pci_region_write_dma_reference() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x81, 0xff, 0x84, 0x28, 0x00, 0x80]); // pinned access selector
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfe, 0x04]); // BAR four
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xf7, 0xc1, 0xff, 0x0f, 0x00, 0x00]); // page-aligned buffer
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x20]); // descriptor register
    code.jump_if(0x85, "driver");
    code.bytes(&[0x45, 0x85, 0xc0]); // buffer offset zero
    code.jump_if(0x85, "fail");
    code.jump("validated");
    code.bind("driver")?;
    code.bytes(&[0x83, 0xfa, 0x28]); // driver register
    code.jump_if(0x85, "device");
    code.bytes(&[0x41, 0x81, 0xf8, 0x80, 0x00, 0x00, 0x00]); // buffer offset 128
    code.jump_if(0x85, "fail");
    code.jump("validated");
    code.bind("device")?;
    code.bytes(&[0x83, 0xfa, 0x30]); // device register
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x81, 0xf8, 0x94, 0x00, 0x00, 0x00]); // buffer offset 148
    code.jump_if(0x85, "fail");
    code.bind("validated")?;
    code.bytes(&[0x4c, 0x01, 0xc1, 0x49, 0x89, 0xcb]); // reference in r11
    code.bytes(&[0x41, 0x89, 0xfa, 0x41, 0x89, 0xd1]); // selector r10d, target r9d

    code.bytes(&[0x41, 0x8d, 0x42, 0x04, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x89, 0xf0, 0xef]); // BAR
    code.bytes(&[0x41, 0x8d, 0x42, 0x08, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x44, 0x89, 0xc8, 0xef]); // target
    code.bytes(&[0x41, 0x8d, 0x42, 0x0c, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0xb8, 0x04, 0x00, 0x00, 0x00, 0xef]); // dword
    code.bytes(&[0x41, 0x8d, 0x42, 0x10, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x44, 0x89, 0xd8, 0xef]); // low dword

    code.bytes(&[0x41, 0x83, 0xc1, 0x04]); // target high dword
    code.bytes(&[0x41, 0x8d, 0x42, 0x08, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x44, 0x89, 0xc8, 0xef]);
    code.bytes(&[0x41, 0x8d, 0x42, 0x10, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x4c, 0x89, 0xd8]);
    code.bytes(&[0x48, 0xc1, 0xe8, 0x20, 0xef]); // high dword
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}
