//! Minimal editor-facing Topal language server state and protocol handling.

use std::collections::BTreeMap;
use std::fs;

use serde_json::{Value, json};
use topal_linter::{LintControl, LintEngine};
use topal_source::{Diagnostic, Severity, SourceText};
use topal_syntax::{Statement, TokenKind, lex, parse};

#[derive(Default)]
pub struct Server {
    documents: BTreeMap<String, String>,
    lint_engine: LintEngine,
    lint_controls: Vec<LintControl>,
    shutdown: bool,
    exit: bool,
}

impl Server {
    #[must_use]
    pub fn handle(&mut self, message: &Value) -> Vec<Value> {
        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").cloned();
        match method {
            Some("initialize") => self.initialize(id, message),
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
            Some("textDocument/completion") => {
                vec![response(
                    id,
                    &completion_items(
                        self.document_has_feature(message, "debug"),
                        self.document_has_feature(message, "lint"),
                    ),
                )]
            }
            Some(_) if id.is_some() => vec![error_response(id, -32601, "method not found")],
            Some(_) | None => Vec::new(),
        }
    }

    fn initialize(&mut self, id: Option<Value>, message: &Value) -> Vec<Value> {
        let controls = match lint_controls(message) {
            Ok(controls) => controls,
            Err(error) => return vec![error_response(id, -32602, &error)],
        };
        let mut lint_engine = LintEngine::builtin();
        if let Err(error) = load_lint_catalogs(message, &mut lint_engine) {
            return vec![error_response(id, -32602, &error)];
        }
        if let Err(error) = lint_engine.lint_text("use language ( version is v0.1 )\n", &controls) {
            return vec![error_response(id, -32602, &error)];
        }
        self.lint_engine = lint_engine;
        self.lint_controls = controls;
        vec![response(
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
        )]
    }

    #[must_use]
    pub const fn should_exit(&self) -> bool {
        self.exit
    }

    #[must_use]
    pub const fn shutdown_requested(&self) -> bool {
        self.shutdown
    }

    fn document_has_feature(&self, message: &Value, expected: &str) -> bool {
        let Some(uri) = message["params"]["textDocument"]["uri"].as_str() else {
            return false;
        };
        self.documents
            .get(uri)
            .is_some_and(|text| source_has_feature(text, expected))
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
        vec![publish_diagnostics(
            uri,
            text,
            &self.lint_engine,
            &self.lint_controls,
        )]
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
        vec![publish_diagnostics(
            uri,
            text,
            &self.lint_engine,
            &self.lint_controls,
        )]
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

fn source_has_feature(text: &str, expected: &str) -> bool {
    let Ok(source) = SourceText::new(text) else {
        return false;
    };
    let parsed = parse(&source, &lex(&source));
    matches!(
        parsed.statements.first(),
        Some(Statement::LanguageSelection { features, .. })
            if features.iter().any(|feature| source.slice(*feature) == expected)
    )
}

#[allow(clippy::too_many_lines)] // Keep the deterministic completion catalog together.
fn completion_items(debug_variant: bool, lint_variant: bool) -> Value {
    let mut result = json!({
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
    });
    if debug_variant {
        let items = result["items"].as_array_mut().unwrap();
        for command in [
            "lang debug break",
            "lang debug continue",
            "lang debug reverse-step",
            "lang debug step",
        ] {
            items.push(json!({
                "label": command,
                "kind": 3,
                "detail": "debug language variant command"
            }));
        }
    }
    if lint_variant {
        result["items"].as_array_mut().unwrap().push(json!({
            "label": "lang lint",
            "kind": 9,
            "detail": "lint language variant namespace"
        }));
    }
    result
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

fn publish_diagnostics(
    uri: &str,
    text: &str,
    engine: &LintEngine,
    controls: &[LintControl],
) -> Value {
    let diagnostics = diagnostics_with_lint(text, engine, controls);
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diagnostics }
    })
}

#[cfg(test)]
fn diagnostics(text: &str) -> Vec<Value> {
    diagnostics_with_lint(text, &LintEngine::builtin(), &[])
}

fn diagnostics_with_lint(text: &str, engine: &LintEngine, controls: &[LintControl]) -> Vec<Value> {
    match engine.lint_text(text, controls) {
        Ok(report) => {
            let normalized = SourceText::new(text).ok();
            let coordinate_text = normalized.as_ref().map_or(text, SourceText::as_str);
            report
                .diagnostics
                .iter()
                .map(|diagnostic| protocol_diagnostic(coordinate_text, diagnostic))
                .collect()
        }
        Err(error) => vec![protocol_diagnostic(
            text,
            &Diagnostic::error("L-LINT-ENGINE", 1, 1, error),
        )],
    }
}

fn load_lint_catalogs(message: &Value, engine: &mut LintEngine) -> Result<(), String> {
    let Some(value) = message.pointer("/params/initializationOptions/lint/catalogs") else {
        return Ok(());
    };
    let paths = value.as_array().ok_or_else(|| {
        "initializationOptions.lint.catalogs must be an array of paths".to_string()
    })?;
    for value in paths {
        let path = value.as_str().ok_or_else(|| {
            "initializationOptions.lint.catalogs must contain only paths".to_string()
        })?;
        let source = fs::read_to_string(path)
            .map_err(|error| format!("cannot read lint catalog `{path}`: {error}"))?;
        engine
            .add_catalog_json(&source)
            .map_err(|error| format!("invalid lint catalog `{path}`: {error}"))?;
    }
    Ok(())
}

fn lint_controls(message: &Value) -> Result<Vec<LintControl>, String> {
    let mut controls = Vec::new();
    for (name, enable) in [("enable", true), ("disable", false)] {
        let Some(value) = message.pointer(&format!("/params/initializationOptions/lint/{name}"))
        else {
            continue;
        };
        let entries = value.as_array().ok_or_else(|| {
            format!("initializationOptions.lint.{name} must be an array of selectors")
        })?;
        for entry in entries {
            let selector = entry.as_str().ok_or_else(|| {
                format!("initializationOptions.lint.{name} must contain only selectors")
            })?;
            controls.push(if enable {
                LintControl::Enable(selector.to_owned())
            } else {
                LintControl::Disable(selector.to_owned())
            });
        }
    }
    if let Some(value) = message.pointer("/params/initializationOptions/lint/severity") {
        let settings = value.as_object().ok_or_else(|| {
            "initializationOptions.lint.severity must map selectors to levels".to_string()
        })?;
        for (selector, level) in settings {
            controls.push(match level.as_str() {
                Some("warning") => LintControl::Severity {
                    selector: selector.clone(),
                    severity: Severity::Warning,
                },
                Some("error") => LintControl::Severity {
                    selector: selector.clone(),
                    severity: Severity::Error,
                },
                Some("off") => LintControl::Off(selector.clone()),
                _ => {
                    return Err(format!(
                        "initializationOptions.lint.severity level for `{selector}` must be warning, error, or off"
                    ));
                }
            });
        }
    }
    Ok(controls)
}

fn protocol_diagnostic(text: &str, diagnostic: &Diagnostic) -> Value {
    let start_offset = diagnostic.source_span.as_deref().map_or_else(
        || byte_offset(text, diagnostic.line, diagnostic.column),
        |span| span.start,
    );
    let end_offset = diagnostic
        .source_span
        .as_deref()
        .map_or(start_offset, |span| span.end);
    let source = if diagnostic.best_practice.is_some() {
        "topal-lint"
    } else {
        "topal"
    };
    let mut result = json!({
        "range": {
            "start": protocol_position(text, start_offset),
            "end": protocol_position(text, end_offset)
        },
        "severity": match diagnostic.severity {
            Severity::Error => 1,
            Severity::Warning => 2,
        },
        "code": diagnostic.code,
        "source": source,
        "message": diagnostic.message
    });
    if let Some(best_practice) = &diagnostic.best_practice {
        result["data"] = json!({
            "bestPractice": best_practice.identity,
            "bestPracticeVersion": best_practice.version,
            "ruleVersion": best_practice.rule_version,
            "checkability": best_practice.checkability.as_deref(),
            "confidence": best_practice.confidence.as_deref(),
            "rectification": best_practice.suggestion.as_deref().map(|message| json!({
                "kind": "suggestion",
                "message": message
            })),
            "help": diagnostic.help.as_deref()
        });
    }
    result
}

fn byte_offset(text: &str, line: usize, column: usize) -> usize {
    let line_start = match line {
        0 | 1 => 0,
        _ => text
            .match_indices('\n')
            .nth(line - 2)
            .map_or(0, |(offset, _)| offset + 1),
    };
    line_start
        + text[line_start..]
            .chars()
            .take(column.saturating_sub(1))
            .map(char::len_utf8)
            .sum::<usize>()
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
                    | "Interface"
                    | "language"
                    | "is"
                    | "otherwise"
                    | "resumes"
                    | "return"
                    | "static"
                    | "then"
                    | "use"
                    | "yield"
                    | "yields"
            ) =>
        {
            Some(4)
        }
        TokenKind::Identifier => Some(0),
        TokenKind::Integer | TokenKind::Rational | TokenKind::Version => Some(1),
        TokenKind::String => Some(2),
        TokenKind::Comment | TokenKind::Hashbang => Some(3),
        TokenKind::Boolean | TokenKind::Discard => Some(4),
        TokenKind::LeftParen
        | TokenKind::At
        | TokenKind::RightParen
        | TokenKind::LeftBrace
        | TokenKind::RightBrace
        | TokenKind::LeftBracket
        | TokenKind::RightBracket
        | TokenKind::Comma
        | TokenKind::Colon
        | TokenKind::Arrow
        | TokenKind::Equals
        | TokenKind::Bang
        | TokenKind::NotEquals
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::LessEqual
        | TokenKind::Compare
        | TokenKind::Range
        | TokenKind::Ellipsis
        | TokenKind::GreaterEqual
        | TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::SlashPercent
        | TokenKind::Percent
        | TokenKind::Caret
        | TokenKind::Dot => Some(5),
        TokenKind::Whitespace | TokenKind::Newline | TokenKind::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use topal_best_practices::Catalog;

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
    fn completes_debug_commands_only_in_the_debug_variant() {
        let mut server = Server::default();
        let _ = server.handle(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///commands.debug",
                "languageId": "topal",
                "version": 1,
                "text": "use language ( version is v0.1, features is ( debug ) )\n"
            }}
        }));
        let output = server.handle(&json!({
            "jsonrpc": "2.0", "id": 8, "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": "file:///commands.debug" },
                "position": { "line": 1, "character": 0 }
            }
        }));
        assert!(
            output[0]["result"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "lang debug break")
        );
    }

    #[test]
    fn completes_lint_namespace_only_in_the_lint_variant() {
        let mut server = Server::default();
        let opened = server.handle(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///rule.t",
                "languageId": "topal",
                "version": 1,
                "text": include_str!("../../../examples/linter/task-declaration-order-rule.t")
            }}
        }));
        assert!(
            opened[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let output = server.handle(&json!({
            "jsonrpc": "2.0", "id": 9, "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": "file:///rule.t" },
                "position": { "line": 1, "character": 0 }
            }
        }));
        let items = output[0]["result"]["items"].as_array().unwrap();
        assert!(items.iter().any(|item| item["label"] == "lang lint"));
        assert!(!items.iter().any(|item| item["label"] == "lang debug step"));
    }

    #[test]
    fn accepts_the_task_state_machine_lint_rule_module() {
        let source = include_str!("../../../examples/linter/task-state-machine-rule.t");
        assert!(diagnostics(source).is_empty());
        assert!(
            !semantic_tokens(source)["data"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn publishes_explicitly_enabled_shared_lint_findings() {
        let mut server = Server::default();
        let initialized = server.handle(&json!({
            "jsonrpc": "2.0", "id": 20, "method": "initialize",
            "params": { "initializationOptions": { "lint": { "enable": [
                "lang best-practice task state-machine"
            ] } } }
        }));
        assert_eq!(initialized[0]["result"]["serverInfo"]["name"], "topal-lsp");
        let source = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\n😀 is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        let opened = server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///state-machine.t", "languageId": "topal", "version": 1,
                "text": source
            } }
        }));
        let diagnostics = opened[0]["params"]["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic["code"], "L-TASK-STATE-MACHINE");
        assert_eq!(diagnostic["severity"], 2);
        assert_eq!(diagnostic["source"], "topal-lint");
        assert_eq!(
            diagnostic["data"]["bestPractice"],
            "lang best-practice task state-machine"
        );
        assert_eq!(diagnostic["data"]["rectification"]["kind"], "suggestion");
        assert!(
            diagnostic["data"]["help"]
                .as_str()
                .unwrap()
                .contains("Start an event-driven state machine")
        );
        assert_eq!(diagnostic["data"]["checkability"], "heuristic");
        assert!(
            diagnostic["data"]["confidence"]
                .as_str()
                .unwrap()
                .contains("indirect transitions")
        );
        assert_eq!(diagnostic["range"]["start"]["line"], 4);
        assert_eq!(diagnostic["range"]["start"]["character"], 0);
        assert_eq!(diagnostic["range"]["end"]["character"], 2);

        let corrected = "use language (\r\n  version is v0.1\r\n)\r\nCounter is Task (queue-size is 2)\r\nservice is Counter\r\n  count : Nat\r\n  start is fn (initial : Nat) -> Completed\r\n    @ count is initial\r\n    Completed\r\n  increment is fn (_ : MessageContext, amount : Nat) -> Unit\r\n    @ count is @ count + amount\r\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\r\n    @ count\r\n";
        let changed = server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": "file:///state-machine.t", "version": 2 },
                "contentChanges": [{ "text": corrected }]
            }
        }));
        assert!(
            changed[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_unknown_lint_identity_during_initialization() {
        let mut server = Server::default();
        let response = server.handle(&json!({
            "jsonrpc": "2.0", "id": 21, "method": "initialize",
            "params": { "initializationOptions": { "lint": { "enable": [
                "lang best-practice missing"
            ] } } }
        }));
        assert_eq!(response[0]["error"]["code"], -32602);
        assert!(
            response[0]["error"]["message"]
                .as_str()
                .unwrap()
                .contains("matches no best-practice")
        );
    }

    #[test]
    fn applies_shared_lint_selector_and_severity_policy() {
        let mut server = Server::default();
        let initialized = server.handle(&json!({
            "jsonrpc": "2.0", "id": 22, "method": "initialize",
            "params": { "initializationOptions": { "lint": {
                "enable": ["tag:lang best-practice tag architecture"],
                "severity": { "lang best-practice task state-machine": "error" }
            } } }
        }));
        assert!(initialized[0].get("error").is_none());
        let source = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        let opened = server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///policy.t", "languageId": "topal", "version": 1,
                "text": source
            } }
        }));
        assert_eq!(opened[0]["params"]["diagnostics"][0]["severity"], 1);

        let mut disabled = Server::default();
        let initialized = disabled.handle(&json!({
            "jsonrpc": "2.0", "id": 23, "method": "initialize",
            "params": { "initializationOptions": { "lint": {
                "enable": ["tag:lang best-practice tag architecture"],
                "disable": ["lang best-practice task state-machine"]
            } } }
        }));
        assert!(initialized[0].get("error").is_none());
        let opened = disabled.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///disabled.t", "languageId": "topal", "version": 1,
                "text": source
            } }
        }));
        assert!(
            opened[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn honors_structured_source_suppression_after_severity_policy() {
        let mut server = Server::default();
        let initialized = server.handle(&json!({
            "jsonrpc": "2.0", "id": 27, "method": "initialize",
            "params": { "initializationOptions": { "lint": {
                "enable": ["lang best-practice task state-machine"],
                "severity": { "lang best-practice task state-machine": "error" }
            } } }
        }));
        assert!(initialized[0].get("error").is_none());
        let source = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nlang disable-diagnostic ( lang best-practice task state-machine )\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        let opened = server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///source-suppressed.t", "languageId": "topal", "version": 1,
                "text": source
            } }
        }));
        assert!(
            opened[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_malformed_lsp_lint_policy() {
        let mut server = Server::default();
        let response = server.handle(&json!({
            "jsonrpc": "2.0", "id": 24, "method": "initialize",
            "params": { "initializationOptions": { "lint": {
                "severity": { "lang best-practice task state-machine": "fatal" }
            } } }
        }));
        assert_eq!(response[0]["error"]["code"], -32602);
        assert!(
            response[0]["error"]["message"]
                .as_str()
                .unwrap()
                .contains("warning, error, or off")
        );
    }

    #[test]
    fn loads_only_explicit_external_lint_catalogs() {
        let mut catalog = Catalog::builtin();
        catalog
            .entries
            .retain(|entry| entry.identity.ends_with("state-machine"));
        catalog.entries[0].identity = "org.example best-practice task state-machine".into();
        let path = std::env::temp_dir().join(format!(
            "topal-lsp-external-catalog-{}.json",
            std::process::id()
        ));
        fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();

        let mut server = Server::default();
        let initialized = server.handle(&json!({
            "jsonrpc": "2.0", "id": 25, "method": "initialize",
            "params": { "initializationOptions": { "lint": {
                "catalogs": [path.to_str().unwrap()],
                "enable": ["org.example best-practice task state-machine"]
            } } }
        }));
        assert!(initialized[0].get("error").is_none());
        let source = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        let opened = server.handle(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///external.t", "languageId": "topal", "version": 1,
                "text": source
            } }
        }));
        assert_eq!(
            opened[0]["params"]["diagnostics"][0]["data"]["bestPractice"],
            "org.example best-practice task state-machine"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unreadable_external_lint_catalog_path() {
        let mut server = Server::default();
        let response = server.handle(&json!({
            "jsonrpc": "2.0", "id": 26, "method": "initialize",
            "params": { "initializationOptions": { "lint": {
                "catalogs": ["/topal-test/catalog-does-not-exist.json"]
            } } }
        }));
        assert_eq!(response[0]["error"]["code"], -32602);
        assert!(
            response[0]["error"]["message"]
                .as_str()
                .unwrap()
                .contains("cannot read lint catalog")
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
                "text": "𐐀 #"
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
    fn accepts_and_highlights_function_interfaces() {
        let source = include_str!("../../../examples/language/function-interface.t");
        assert!(diagnostics(source).is_empty());
        assert_eq!(
            semantic_token_type(TokenKind::Identifier, "Interface"),
            Some(4)
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
    fn highlights_broad_unicode_identifiers_as_single_variables() {
        assert_eq!(
            semantic_tokens("🙂 is 40\nleft+right is 2")["data"],
            json!([
                0, 0, 2, 0, 0, 0, 3, 2, 4, 0, 0, 3, 2, 1, 0, 1, 0, 10, 0, 0, 0, 11, 2, 4, 0, 0, 3,
                1, 1, 0
            ])
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
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language");
        let mut examples = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
            .collect::<Vec<_>>();
        examples.sort();
        assert!(
            examples.len() >= 180,
            "the accepted example corpus must not silently shrink"
        );

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
    fn accepts_the_standard_library_fundamental_sources() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library/fundamental/ordering.t");
        let text = std::fs::read_to_string(&path).unwrap();
        let mut server = Server::default();
        let output = server.handle(&json!({
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": format!("file://{}", path.display()),
                "languageId": "topal",
                "version": 1,
                "text": text
            } }
        }));
        assert!(
            output[0]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty(),
            "{}",
            path.display()
        );
        let numeric = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../library/numeric/exact.t");
        assert!(diagnostics(&std::fs::read_to_string(numeric).unwrap()).is_empty());
    }

    #[test]
    fn accepts_yield_after_close_diagnostic_example_syntax() {
        let source = include_str!(
            "../../../examples/language-diagnostics/custom-generator-yield-after-close.t"
        );
        assert!(diagnostics(source).is_empty());
    }
}
