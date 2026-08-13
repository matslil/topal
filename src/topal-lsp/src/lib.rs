//! Minimal editor-facing Topal language server state and protocol handling.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use topal_source::{SourceText, Span};
use topal_syntax::{SyntaxDiagnostic, TokenKind, lex, parse};

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
                        "textDocumentSync": { "openClose": true, "change": 1 },
                        "completionProvider": {},
                        "semanticTokensProvider": {
                            "legend": {
                                "tokenTypes": [
                                    "variable", "number", "string", "comment", "keyword", "operator"
                                ],
                                "tokenModifiers": []
                            },
                            "full": true
                        }
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
            Some("textDocument/semanticTokens/full") => {
                vec![response(id, &self.semantic_tokens(message))]
            }
            Some("textDocument/completion") => vec![response(id, &completion_items())],
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

    fn semantic_tokens(&self, message: &Value) -> Value {
        let Some(uri) = message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)
        else {
            return json!({ "data": [] });
        };
        self.documents
            .get(uri)
            .map_or_else(|| json!({ "data": [] }), |text| semantic_tokens(text))
    }
}

fn completion_items() -> Value {
    json!({
        "isIncomplete": false,
        "items": [
            {
                "label": "absolute",
                "kind": 3,
                "detail": "Int -> Int; Rational -> Rational"
            },
            {
                "label": "byte-count",
                "kind": 3,
                "detail": "String, Utf8 -> Int"
            },
            {
                "label": "canonically-equals",
                "kind": 3,
                "detail": "String, String -> Boolean"
            },
            {
                "label": "case-fold",
                "kind": 3,
                "detail": "String -> String"
            },
            {
                "label": "character-count",
                "kind": 3,
                "detail": "String -> Int"
            },
            {
                "label": "characters",
                "kind": 3,
                "detail": "String -> Generator Character Unit Unit"
            },
            {
                "label": "collect",
                "kind": 3,
                "detail": "Generator Character Unit Unit, String -> String"
            },
            {
                "label": "concat",
                "kind": 3,
                "detail": "String, String -> String"
            },
            {
                "label": "empty",
                "kind": 3,
                "detail": "String -> String"
            },
            {
                "label": "empty?",
                "kind": 3,
                "detail": "String -> Boolean"
            },
            {
                "label": "entry-count",
                "kind": 3,
                "detail": "String -> Int"
            },
            {
                "label": "lower",
                "kind": 3,
                "detail": "String -> String"
            },
            {
                "label": "normalize",
                "kind": 3,
                "detail": "String, NFC -> String"
            },
            {
                "label": "upper",
                "kind": 3,
                "detail": "String -> String"
            },
            {
                "label": "negate",
                "kind": 3,
                "detail": "Int -> Int; Rational -> Rational"
            },
            {
                "label": "one",
                "kind": 3,
                "detail": "Type -> Value"
            },
            {
                "label": "zero",
                "kind": 3,
                "detail": "Type -> Value"
            }
        ]
    })
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
        &diagnostic.message,
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
    let (line, character) = protocol_coordinates(text, offset);
    json!({ "line": line, "character": character })
}

fn protocol_coordinates(text: &str, offset: usize) -> (usize, usize) {
    let before = text.get(..offset).unwrap_or(text);
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let tail = before.rsplit_once('\n').map_or(before, |(_, tail)| tail);
    (line, tail.encode_utf16().count())
}

fn semantic_tokens(text: &str) -> Value {
    let Ok(source) = SourceText::new(text) else {
        return json!({ "data": [] });
    };
    let mut absolute = Vec::new();
    for token in lex(&source).tokens {
        let Some(token_type) = semantic_token_type(token.kind, source.slice(token.span)) else {
            continue;
        };
        let mut offset = token.span.start;
        for segment in source.slice(token.span).split_inclusive('\n') {
            let content = segment.strip_suffix('\n').unwrap_or(segment);
            if !content.is_empty() {
                let (line, start) = protocol_coordinates(source.as_str(), offset);
                absolute.push((line, start, content.encode_utf16().count(), token_type));
            }
            offset += segment.len();
        }
    }
    let mut data = Vec::with_capacity(absolute.len() * 5);
    let mut previous_line = 0;
    let mut previous_start = 0;
    for (line, start, length, token_type) in absolute {
        let delta_line = line - previous_line;
        let delta_start = if delta_line == 0 {
            start - previous_start
        } else {
            start
        };
        data.extend([delta_line, delta_start, length, token_type, 0]);
        previous_line = line;
        previous_start = start;
    }
    json!({ "data": data })
}

fn semantic_token_type(kind: TokenKind, lexeme: &str) -> Option<usize> {
    match kind {
        TokenKind::Identifier
            if matches!(
                lexeme,
                "fn" | "generator"
                    | "is"
                    | "otherwise"
                    | "resumes"
                    | "return"
                    | "static"
                    | "then"
                    | "yield"
                    | "yields"
            ) =>
        {
            Some(4)
        }
        TokenKind::Identifier => Some(0),
        TokenKind::Integer | TokenKind::Rational => Some(1),
        TokenKind::String => Some(2),
        TokenKind::Comment | TokenKind::Hashbang => Some(3),
        TokenKind::Boolean | TokenKind::Discard => Some(4),
        TokenKind::LeftParen
        | TokenKind::RightParen
        | TokenKind::LeftBrace
        | TokenKind::RightBrace
        | TokenKind::Comma
        | TokenKind::Colon
        | TokenKind::Arrow
        | TokenKind::Equals
        | TokenKind::NotEquals
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::LessEqual
        | TokenKind::Compare
        | TokenKind::Range
        | TokenKind::GreaterEqual
        | TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::SlashPercent
        | TokenKind::Percent
        | TokenKind::Caret => Some(5),
        TokenKind::Whitespace | TokenKind::Newline | TokenKind::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

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
        assert_eq!(
            output[0]["result"]["capabilities"]["completionProvider"],
            json!({})
        );
    }

    #[test]
    fn completes_only_implemented_named_root_operations() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "jsonrpc": "2.0", "id": 7, "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": "file:///completion.t" },
                "position": { "line": 0, "character": 0 }
            }
        }));
        let labels = output[0]["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["label"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                "absolute",
                "byte-count",
                "canonically-equals",
                "case-fold",
                "character-count",
                "characters",
                "collect",
                "concat",
                "empty",
                "empty?",
                "entry-count",
                "lower",
                "normalize",
                "upper",
                "negate",
                "one",
                "zero"
            ]
        );
        let normalize = output[0]["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["label"] == "normalize")
            .unwrap();
        assert_eq!(normalize["detail"], "String, NFC -> String");
        let characters = output[0]["result"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["label"] == "characters")
            .unwrap();
        assert_eq!(
            characters["detail"],
            "String -> Generator Character Unit Unit"
        );
        assert!(
            output[0]["result"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item["kind"] == 3)
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
    fn publishes_missing_error_code_alternatives() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///result.t", "version": 1,
                "text": "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\""
            }}
        }));
        let diagnostic = &output[0]["params"]["diagnostics"][0];
        assert_eq!(diagnostic["code"], "E-INCOMPLETE-ERROR-CODE-DECISION");
        assert!(
            diagnostic["message"]
                .as_str()
                .unwrap()
                .contains("not-representable")
        );
    }

    #[test]
    fn publishes_duplicate_error_code_pattern_diagnostic() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///duplicate-result.t", "version": 1,
                "text": "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error ( code is lang arithmetic division-by-zero ) then \"first\"\n    Error ( code is lang arithmetic division-by-zero ) then \"second\"\n    Error problem then \"other\""
            }}
        }));
        assert_eq!(
            output[0]["params"]["diagnostics"][0]["code"],
            "E-DUPLICATE-ERROR-CODE-PATTERN"
        );
    }

    #[test]
    fn publishes_unreachable_error_code_pattern_diagnostic() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///unreachable-result.t", "version": 1,
                "text": "describe is fn (attempt : Result) -> String\n  attempt\n    Ok value then \"ok\"\n    Error problem then \"other\"\n    Error ( code is lang arithmetic division-by-zero ) then \"zero\""
            }}
        }));
        assert_eq!(
            output[0]["params"]["diagnostics"][0]["code"],
            "E-UNREACHABLE-ERROR-CODE-PATTERN"
        );
    }

    #[test]
    fn publishes_rule_after_otherwise_diagnostic() {
        let mut server = Server::default();
        let output = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///fallback-order.t", "version": 1,
                "text": "choose is fn (condition : Boolean) -> Int\n  condition\n    otherwise 0\n    true then 1"
            }}
        }));
        let diagnostic = &output[0]["params"]["diagnostics"][0];
        assert_eq!(diagnostic["code"], "E-UNREACHABLE-DECISION-RULE");
        assert_eq!(diagnostic["range"]["start"]["line"], 3);
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

    #[test]
    fn returns_utf16_semantic_tokens_for_incomplete_unicode_source() {
        let mut server = Server::default();
        let _ = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///tokens.t", "text": "𐐀 + true\ntag\"one\ntwo"
            }}
        }));
        let output = server.handle(&json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/semanticTokens/full",
            "params": { "textDocument": { "uri": "file:///tokens.t" } }
        }));
        assert_eq!(
            output[0]["result"]["data"],
            json!([
                0, 0, 2, 0, 0, 0, 3, 1, 5, 0, 0, 2, 4, 4, 0, 1, 0, 7, 2, 0, 1, 0, 3, 2, 0
            ])
        );
    }

    #[test]
    fn highlights_return_as_a_keyword() {
        assert_eq!(
            semantic_tokens("return 42")["data"],
            json!([0, 0, 6, 4, 0, 0, 7, 2, 1, 0])
        );
    }

    #[test]
    fn highlights_function_declaration_words_as_keywords() {
        for keyword in [
            "fn",
            "generator",
            "is",
            "otherwise",
            "resumes",
            "static",
            "then",
            "yield",
            "yields",
        ] {
            assert_eq!(semantic_token_type(TokenKind::Identifier, keyword), Some(4));
        }
    }

    #[test]
    fn every_interpreter_example_has_clean_diagnostics_and_highlighting() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/interpreter");
        let mut examples = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
            .collect::<Vec<_>>();
        examples.sort();
        assert_eq!(examples.len(), 138);

        let mut server = Server::default();
        for (version, example) in examples.iter().enumerate() {
            let text = std::fs::read_to_string(example).unwrap();
            let uri = format!("file://{}", example.display());
            let published = server.handle(&json!({
                "method": "textDocument/didOpen",
                "params": { "textDocument": {
                    "uri": uri, "languageId": "topal", "version": version, "text": text
                }}
            }));
            assert_eq!(published[0]["params"]["diagnostics"], json!([]));

            let highlighted = server.handle(&json!({
                "jsonrpc": "2.0", "id": version, "method": "textDocument/semanticTokens/full",
                "params": { "textDocument": { "uri": uri } }
            }));
            assert!(
                highlighted[0]["result"]["data"]
                    .as_array()
                    .is_some_and(|tokens| !tokens.is_empty())
            );
        }
    }

    #[test]
    fn accepts_yield_after_close_diagnostic_example_syntax() {
        let source =
            include_str!("../../../examples/debugger/custom-generator-yield-after-close.t");
        assert!(diagnostics(source).is_empty());
    }
}
