use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn framed(message: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(message).unwrap();
    let mut framed = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    framed.extend(body);
    framed
}

fn responses(output: &[u8]) -> Vec<Value> {
    let mut remaining = output;
    let mut messages = Vec::new();
    while !remaining.is_empty() {
        let header_end = remaining
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap();
        let headers = std::str::from_utf8(&remaining[..header_end]).unwrap();
        let length = headers
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let body_start = header_end + 4;
        messages.push(serde_json::from_slice(&remaining[body_start..body_start + length]).unwrap());
        remaining = &remaining[body_start + length..];
    }
    messages
}

#[test]
fn stdio_transcript_initializes_publishes_and_shuts_down() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_topal-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = child.stdin.as_mut().unwrap();
    for message in [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///test.t", "languageId": "topal", "version": 1, "text": "?"
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": "file:///test.t" },
                "position": { "line": 0, "character": 1 }
            }
        }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null }),
        json!({ "jsonrpc": "2.0", "method": "exit", "params": null }),
    ] {
        input.write_all(&framed(&message)).unwrap();
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let messages = responses(&output.stdout);
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0]["id"], 1);
    assert_eq!(messages[1]["method"], "textDocument/publishDiagnostics");
    assert_eq!(
        messages[1]["params"]["diagnostics"][0]["code"],
        "E-UNKNOWN-TOKEN"
    );
    assert_eq!(messages[2]["id"], 3);
    assert_eq!(messages[2]["result"]["items"][3]["label"], "concat");
    assert_eq!(messages[3]["id"], 2);
}
