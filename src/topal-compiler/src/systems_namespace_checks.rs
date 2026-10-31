//! Semantic ABI cases are shared data; Rust/Python independently encode them.
use super::checks::Witness;
use serde_json::Value;

pub(in super::super) const ENTRY: u32 = 0x004b_4000;
pub(in super::super) const PRE_EXEC: u32 = 0x004c_0000;
const CASES: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/namespace-cases.json");

fn number(value: &Value) -> u64 {
    if let Some(s) = value.as_str() {
        u64::from_str_radix(s.strip_prefix("0x").unwrap(), 16).unwrap()
    } else if let Some(n) = value.as_u64() {
        n
    } else {
        value.as_i64().unwrap() as u64
    }
}

pub(in super::super) fn code(stage: &str) -> Vec<u8> {
    let cases: Value = serde_json::from_str(CASES).expect("valid namespace fixture data");
    assert_eq!(cases["schema"], "topal-kernel-namespace-witness/1");
    let mut w = Witness::default();
    for case in cases[stage].as_array().unwrap() {
        let operation = case.as_array().unwrap();
        match operation[0].as_str().unwrap() {
            "call" => {
                let args: Vec<_> = operation[2]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .collect();
                w.call(number(&operation[1]), &args, number(&operation[3]) as i64);
            }
            "store" => w.store(number(&operation[1]), number(&operation[2])),
            "check" => w.memory(number(&operation[1]), number(&operation[2])),
            "byte" => {
                w.0.extend_from_slice(&[0xc6, 0x04, 0x25]);
                w.0.extend_from_slice(&(number(&operation[1]) as u32).to_le_bytes());
                w.0.push(number(&operation[2]) as u8);
            }
            "word32" => {
                w.0.extend_from_slice(&[0xc7, 0x04, 0x25]);
                w.0.extend_from_slice(&(number(&operation[1]) as u32).to_le_bytes());
                w.0.extend_from_slice(&(number(&operation[2]) as u32).to_le_bytes());
            }
            "check32" => {
                w.0.push(0xa1);
                w.0.extend_from_slice(&number(&operation[1]).to_le_bytes());
                w.expect(number(&operation[2]));
            }
            "text" => {
                let mut bytes = operation[2].as_str().unwrap().as_bytes().to_vec();
                bytes.push(0);
                bytes.resize(bytes.len().next_multiple_of(8), 0);
                for (i, chunk) in bytes.chunks_exact(8).enumerate() {
                    w.store(
                        number(&operation[1]) + i as u64 * 8,
                        u64::from_le_bytes(chunk.try_into().unwrap()),
                    );
                }
            }
            "fill" => {
                w.imm(&[0x48, 0xbf], number(&operation[1]));
                w.imm(&[0x48, 0xb9], number(&operation[2]));
                w.imm(&[0x48, 0xb8], number(&operation[3]));
                w.0.extend_from_slice(&[0xfc, 0xf3, 0xaa]);
            }
            "time" => {
                w.imm(&[0x48, 0xbf], number(&operation[1]));
                w.imm(&[0x48, 0xb8], 201);
                w.0.extend_from_slice(&[
                    0x0f, 0x05, 0x48, 0x85, 0xc0, 0x78, 0xfe, 0x49, 0x81, 0xfc, 0xab, 0x51, 0, 0,
                    0x75, 0xfe,
                ]);
                w.0.extend_from_slice(&[0x48, 0x89, 0xc3]); // retain sample in native rbx
                w.imm(&[0x48, 0xa1], number(&operation[1]));
                w.0.extend_from_slice(&[0x48, 0x39, 0xd8, 0x75, 0xfe]);
                w.imm(&[0x48, 0xbf], 0);
                w.imm(&[0x48, 0xb8], 201);
                w.0.extend_from_slice(&[
                    0x0f, 0x05, 0x48, 0x39, 0xd8, 0x72, 0xfe, 0x48, 0x83, 0xc3, 5, 0x48, 0x39,
                    0xd8, 0x77, 0xfe,
                ]);
            }
            unknown => panic!("unknown namespace fixture operation {unknown}"),
        }
    }
    // Scratch is ordinary admitted user heap, separate from argv and CALL RSP.
    w.imm(&[0x48, 0xbf], 0x607000);
    w.imm(&[0x48, 0xb9], 0x2000 / 8);
    w.0.extend_from_slice(&[0x31, 0xc0, 0xfc, 0xf3, 0x48, 0xab]);
    let code = w.finish();
    assert!(code.len() < (PRE_EXEC - ENTRY) as usize);
    code
}
