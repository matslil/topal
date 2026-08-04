//! Minimal editor-facing Topal language server state and protocol handling.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use topal_source::{SourceText, Span};
use topal_syntax::{SyntaxDiagnostic, lex, parse};

#[derive(Default)]
pub struct Server {
    documents: BTreeMap<String, String>,
    shutdown: bool,
    exit: bool,
}

impl Server {
    #[must_use]
    pub fn handle(&mut self, message: &Value) -> Vec<Value> {
        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").cloned();
        match method {
            Some("initialize") => vec![response(
                id,
                &json!({
                    "capabilities": {
                        "positionEncoding": "utf-16",
                        "textDocumentSync": { "openClose": true, "change": 1 }
                    },
                    "serverInfo": {
                        "name": "topal-lsp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )],
            Some("initialized") => Vec::new(),
            Some("shutdown") => {
                self.shutdown = true;
                vec![response(id, &Value::Null)]
            }
            Some("exit") => {
                self.exit = true;
                Vec::new()
            }
            Some("textDocument/didOpen") => self.did_open(message),
            Some("textDocument/didChange") => self.did_change(message),
            Some("textDocument/didClose") => self.did_close(message),
            Some(_) if id.is_some() => vec![error_response(id, -32601, "method not found")],
            Some(_) | None => Vec::new(),
        }
    }

    #[must_use]
    pub const fn should_exit(&self) -> bool {
        self.exit
    }

    #[must_use]
    pub const fn shutdown_requested(&self) -> bool {
        self.shutdown
    }

    fn did_open(&mut self, message: &Value) -> Vec<Value> {
        let Some(document) = message.pointer("/params/textDocument") else {
            return Vec::new();
        };
        let (Some(uri), Some(text)) = (
            document.get("uri").and_then(Value::as_str),
            document.get("text").and_then(Value::as_str),
        ) else {
            return Vec::new();
        };
        self.documents.insert(uri.to_owned(), text.to_owned());
        vec![publish_diagnostics(uri, text)]
    }

    fn did_change(&mut self, message: &Value) -> Vec<Value> {
        let Some(uri) = message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)
        else {
            return Vec::new();
        };
        let Some(text) = message
            .pointer("/params/contentChanges")
            .and_then(Value::as_array)
            .and_then(|changes| changes.last())
            .and_then(|change| change.get("text"))
            .and_then(Value::as_str)
        else {
            return Vec::new();
        };
        self.documents.insert(uri.to_owned(), text.to_owned());
        vec![publish_diagnostics(uri, text)]
    }

    fn did_close(&mut self, message: &Value) -> Vec<Value> {
        let Some(uri) = message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)
        else {
            return Vec::new();
        };
        self.documents.remove(uri);
        vec![json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": { "uri": uri, "diagnostics": [] }
        })]
    }
}

fn response(id: Option<Value>, result: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id.unwrap_or(Value::Null), "result": result })
}

fn error_response(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": { "code": code, "message": message }
    })
}

fn publish_diagnostics(uri: &str, text: &str) -> Value {
    let diagnostics = diagnostics(text);
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diagnostics }
    })
}

fn diagnostics(text: &str) -> Vec<Value> {
    let source = match SourceText::new(text) {
        Ok(source) => source,
        Err(error) => {
            return vec![protocol_diagnostic(
                text,
                error.span,
                error.code,
                error.message,
            )];
        }
    };
    let lexed = lex(&source);
    parse(&source, &lexed)
        .diagnostics
        .iter()
        .map(|diagnostic| syntax_diagnostic(&source, diagnostic))
        .collect()
}

fn syntax_diagnostic(source: &SourceText, diagnostic: &SyntaxDiagnostic) -> Value {
    protocol_diagnostic(
        source.as_str(),
        diagnostic.span,
        diagnostic.code,
        diagnostic.message,
    )
}

fn protocol_diagnostic(text: &str, span: Span, code: &str, message: &str) -> Value {
    json!({
        "range": {
            "start": protocol_position(text, span.start),
            "end": protocol_position(text, span.end)
        },
        "severity": 1,
        "code": code,
        "source": "topal",
        "message": message
    })
}

fn protocol_position(text: &str, offset: usize) -> Value {
    let before = text.get(..offset).unwrap_or(text);
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let tail = before.rsplit_once('\n').map_or(before, |(_, tail)| tail);
    json!({ "line": line, "character": tail.encode_utf16().count() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_with_full_utf16_document_sync() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
        }));
        assert_eq!(
            output[0]["result"]["capabilities"]["positionEncoding"],
            "utf-16"
        );
        assert_eq!(
            output[0]["result"]["capabilities"]["textDocumentSync"]["change"],
            1
        );
    }

    #[test]
    fn publishes_all_shared_syntax_diagnostics_with_utf16_ranges() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///unicode.t", "languageId": "topal", "version": 1,
                "text": "𐐀 ?"
            }}
        }));
        let diagnostic = &output[0]["params"]["diagnostics"][0];
        assert_eq!(diagnostic["code"], "E-UNKNOWN-TOKEN");
        assert_eq!(diagnostic["range"]["start"]["character"], 3);
    }

    #[test]
    fn change_replaces_content_and_close_clears_diagnostics() {
        let mut server = Server::default();
        let _ = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": "file:///a.t", "text": "?" } }
        }));
        let changed = server.handle(&json!({
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///a.t", "version": 2 },
                "contentChanges": [{ "text": "1" }]
            }
        }));
        assert_eq!(changed[0]["params"]["diagnostics"], json!([]));
        let closed = server.handle(&json!({
            "method": "textDocument/didClose",
            "params": { "textDocument": { "uri": "file:///a.t" } }
        }));
        assert_eq!(closed[0]["params"]["diagnostics"], json!([]));
    }
}
