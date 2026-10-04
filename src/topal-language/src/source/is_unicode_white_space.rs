fn is_unicode_white_space(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

fn apply_count(
    source: &SourceText,
    operation: &str,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (count, classifier, event, rule) = match operand {
        Value::String(text) => (
            character_count(&text),
            "String".to_owned(),
            if operation == "entry-count" {
                "string.entry-count"
            } else {
                "string.character-count"
            },
            if operation == "entry-count" {
                "TOPAL-STRING-ENTRY-COUNT-001"
            } else {
                "TOPAL-STRING-CHARACTER-COUNT-001"
            },
        ),
        Value::List {
            element_classifier,
            entries,
        } if operation == "entry-count" => (
            entries.len(),
            format!("List {element_classifier}"),
            "list.entry-count",
            "TOPAL-LIST-ENTRY-COUNT-001",
        ),
        Value::Array { entries, .. } | Value::Set { entries, .. } if operation == "entry-count" => {
            (
                entries.len(),
                "Collection".into(),
                "collection.entry-count",
                "TOPAL-COLLECTION-ENTRY-COUNT-001",
            )
        }
        Value::Bag { entries, .. } if operation == "entry-count" => (
            entries.iter().map(|(_, count)| count).sum(),
            "Bag".into(),
            "collection.entry-count",
            "TOPAL-COLLECTION-ENTRY-COUNT-001",
        ),
        Value::Map { entries, .. } if operation == "entry-count" => (
            entries.len(),
            "Map".into(),
            "collection.entry-count",
            "TOPAL-COLLECTION-ENTRY-COUNT-001",
        ),
        value => {
            let found = structural_value_classifier(&value);
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("{operation} has no overload accepting `{found}`"),
            ));
        }
    };
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    let detail = if event.starts_with("string.") {
        format!("characters={count}")
    } else {
        format!("entries={count}")
    };
    trace.record(TraceEvent {
        event,
        rule,
        detail: &detail,
    });
    Ok(Value::Int(BigInt::from(count)))
}

#[allow(clippy::too_many_lines)] // Keep the collection-query operations together and auditable.
fn apply_collection_query(
    source: &SourceText,
    operation: &str,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Tuple(mut arguments) = operand else {
        return Err(diagnostic(
            source,
            "E-COLLECTION-QUERY-ARGUMENT",
            span,
            format!("{operation} requires one two-field product"),
        ));
    };
    if arguments.len() != 2 {
        return Err(diagnostic(
            source,
            "E-COLLECTION-QUERY-ARGUMENT",
            span,
            format!("{operation} requires one two-field product"),
        ));
    }
    let query = arguments.pop().unwrap();
    let collection = arguments.pop().unwrap();
    let (value, rule, detail) = match (operation, collection) {
        (
            "array-at?",
            Value::Array {
                element_classifier,
                entries,
            },
        ) => {
            let Value::Int(index) = query else {
                return Err(diagnostic(
                    source,
                    "E-ARRAY-INDEX-CLASSIFIER",
                    span,
                    "array-at? requires a Nat index",
                ));
            };
            let payload = usize::try_from(index)
                .ok()
                .and_then(|index| entries.get(index).cloned())
                .map(Box::new);
            (
                Value::Optional {
                    payload_classifier: element_classifier,
                    payload,
                },
                "TOPAL-ARRAY-GET-CHECKED-001",
                "array.checked-access",
            )
        }
        (
            "map-lookup",
            Value::Map {
                key_classifier,
                value_classifier,
                entries,
            },
        ) => {
            if !value_has_classifier(&query, &key_classifier) {
                return Err(diagnostic(
                    source,
                    "E-MAP-KEY-CLASSIFIER",
                    span,
                    format!("map-lookup requires a `{key_classifier}` key"),
                ));
            }
            let payload = entries
                .into_iter()
                .find_map(|(key, value)| {
                    values_equal(key, query.clone(), &mut Vec::new())?.then_some(value)
                })
                .map(Box::new);
            (
                Value::Optional {
                    payload_classifier: value_classifier,
                    payload,
                },
                "TOPAL-MAP-LOOKUP-001",
                "map.lookup",
            )
        }
        (
            "set-contains?",
            Value::Set {
                element_classifier,
                entries,
            },
        ) => {
            if !value_has_classifier(&query, &element_classifier) {
                return Err(diagnostic(
                    source,
                    "E-SET-ELEMENT-CLASSIFIER",
                    span,
                    format!("set-contains? requires a `{element_classifier}` value"),
                ));
            }
            let present = entries
                .into_iter()
                .any(|entry| values_equal(entry, query.clone(), &mut Vec::new()).unwrap_or(false));
            (
                Value::Boolean(present),
                "TOPAL-SET-CONTAINS-001",
                "set.membership",
            )
        }
        (
            "bag-multiplicity",
            Value::Bag {
                element_classifier,
                entries,
            },
        ) => {
            if !value_has_classifier(&query, &element_classifier) {
                return Err(diagnostic(
                    source,
                    "E-BAG-ELEMENT-CLASSIFIER",
                    span,
                    format!("bag-multiplicity requires a `{element_classifier}` value"),
                ));
            }
            let count = entries
                .into_iter()
                .find_map(|(entry, count)| {
                    values_equal(entry, query.clone(), &mut Vec::new())?.then_some(count)
                })
                .unwrap_or(0);
            (
                Value::Int(BigInt::from(count)),
                "TOPAL-BAG-MULTIPLICITY-001",
                "bag.multiplicity",
            )
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("{operation} received the wrong collection kind"),
            ));
        }
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail,
    });
    trace.record(TraceEvent {
        event: detail,
        rule,
        detail: operation,
    });
    Ok(value)
}

fn apply_list_reverse(value: &mut Value, trace: &mut impl TraceSink) {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        unreachable!("List reverse dispatched only for a List")
    };
    entries.reverse();
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.reverse(List)",
    });
    trace.record(TraceEvent {
        event: "list.reversed",
        rule: "TOPAL-LIST-REVERSE-001",
        detail: &classifier,
    });
}

fn apply_list_stable_sort(
    source: &SourceText,
    value: &mut Value,
    descending: bool,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<(), Diagnostic> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        unreachable!("stable sort dispatched only for a List")
    };
    if !matches!(element_classifier.as_str(), "Int" | "Rational") {
        return Err(diagnostic(
            source,
            "E-LIST-SORT-CLASSIFIER",
            span,
            "stable sorting currently requires List Int or List Rational",
        ));
    }
    entries.sort_by(|left, right| {
        let ordering = values_compare(left.clone(), right.clone(), trace)
            .expect("validated exact numeric entries are totally ordered");
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: if descending {
            "root.stable-sort-descending(List)"
        } else {
            "root.stable-sort(List)"
        },
    });
    trace.record(TraceEvent {
        event: "list.stably-sorted",
        rule: "TOPAL-LIST-STABLE-SORT-001",
        detail: &classifier,
    });
    Ok(())
}

#[allow(clippy::too_many_lines)] // Keep ordered List operation dispatch together.
fn apply_list_operation(
    source: &SourceText,
    operation: &str,
    left: Value,
    right: Value,
    right_span: Span,
    right_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = left
    else {
        unreachable!("List operation is dispatched only for a List left operand")
    };
    if operation.starts_with("contains-") {
        return apply_list_containment(
            source,
            operation,
            &element_classifier,
            &entries,
            right,
            right_span,
            trace,
        );
    }
    if matches!(operation, "remove-first" | "remove-all") {
        return apply_list_value_removal(
            source,
            operation,
            element_classifier,
            entries,
            &right,
            right_span,
            trace,
        );
    }
    if matches!(
        operation,
        "split-at" | "take" | "drop" | "remove" | "remove-indexes"
    ) {
        return apply_list_index_operation(
            source,
            operation,
            element_classifier,
            entries,
            right,
            right_span,
            right_is_closed,
            trace,
        );
    }
    if matches!(operation, "zip-exact" | "zip-shortest") {
        return apply_list_zip(
            source,
            operation,
            &element_classifier,
            entries,
            right,
            right_span,
            trace,
        );
    }
    match operation {
        "prepend" | "append" => {
            if !value_has_classifier(&right, &element_classifier) {
                let found = structural_value_classifier(&right);
                return Err(diagnostic(
                    source,
                    "E-LIST-ENTRY-CLASSIFIER",
                    right_span,
                    format!(
                        "{operation} received `{found}`, but this list requires `{element_classifier}`"
                    ),
                )
                .with_help(format!("use a `{element_classifier}` value here")));
            }
            if operation == "prepend" {
                entries.insert(0, right);
            } else {
                entries.push(right);
            }
        }
        "concat" => {
            let Value::List {
                element_classifier: right_classifier,
                entries: right_entries,
            } = right
            else {
                return Err(diagnostic(
                    source,
                    "E-LIST-CONCAT-OPERAND",
                    right_span,
                    "List concat requires another List",
                ));
            };
            if right_classifier != element_classifier {
                return Err(diagnostic(
                    source,
                    "E-LIST-CONCAT-CLASSIFIER",
                    right_span,
                    format!(
                        "cannot concatenate `List {right_classifier}` with `List {element_classifier}`"
                    ),
                )
                .with_help("use Lists with the same element classifier"));
            }
            entries.extend(right_entries);
        }
        _ => unreachable!("known List operation"),
    }
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: match operation {
            "prepend" => "list.prepended",
            "append" => "list.appended",
            "concat" => "list.concatenated",
            _ => unreachable!("known List operation"),
        },
        rule: match operation {
            "prepend" => "TOPAL-LIST-PREPEND-001",
            "append" => "TOPAL-LIST-APPEND-001",
            "concat" => "TOPAL-LIST-CONCAT-001",
            _ => unreachable!("known List operation"),
        },
        detail: &classifier,
    });
    Ok(Value::List {
        element_classifier,
        entries,
    })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Bounds, source evidence, and trace context remain explicit.
fn apply_list_index_operation(
    source: &SourceText,
    operation: &str,
    element_classifier: String,
    mut entries: Vec<Value>,
    operand: Value,
    operand_span: Span,
    operand_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if operation == "remove-indexes"
        && let Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } = operand
    {
        let Ok(lower) = usize::try_from(lower) else {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        };
        let Ok(upper) = usize::try_from(upper) else {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        };
        let start = lower + usize::from(!lower_inclusive);
        let end = upper + usize::from(upper_inclusive);
        if start > end || end > entries.len() {
            return list_boundary_failure(
                source,
                operation,
                operand_span,
                operand_is_closed,
                trace,
            );
        }
        entries.drain(start..end);
        trace.record(TraceEvent {
            event: "list.entries.removed",
            rule: "TOPAL-LIST-REMOVE-INDEXES-001",
            detail: &format!("start={start};end={end}"),
        });
        return Ok(Value::List {
            element_classifier,
            entries,
        });
    }
    let Value::Int(index) = operand else {
        return Err(diagnostic(
            source,
            "E-LIST-INDEX-CLASSIFIER",
            operand_span,
            format!("{operation} requires a Nat operand"),
        ));
    };
    let Ok(index) = usize::try_from(index) else {
        return list_boundary_failure(source, operation, operand_span, operand_is_closed, trace);
    };
    let valid = if operation == "remove" {
        index < entries.len()
    } else {
        index <= entries.len()
    };
    if !valid {
        return list_boundary_failure(source, operation, operand_span, operand_is_closed, trace);
    }
    let classifier = format!("List {element_classifier}");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &format!("root.{operation}({classifier},Nat)"),
    });
    let value = match operation {
        "split-at" => {
            let suffix = entries.split_off(index);
            Value::Tuple(vec![
                Value::List {
                    element_classifier: element_classifier.clone(),
                    entries,
                },
                Value::List {
                    element_classifier,
                    entries: suffix,
                },
            ])
        }
        "take" => {
            entries.truncate(index);
            Value::List {
                element_classifier,
                entries,
            }
        }
        "drop" => Value::List {
            element_classifier,
            entries: entries.split_off(index),
        },
        "remove" => {
            entries.remove(index);
            Value::List {
                element_classifier,
                entries,
            }
        }
        _ => unreachable!("known indexed List operation"),
    };
    trace.record(TraceEvent {
        event: "list.region.selected",
        rule: match operation {
            "split-at" => "TOPAL-LIST-SPLIT-AT-001",
            "take" => "TOPAL-LIST-TAKE-001",
            "drop" => "TOPAL-LIST-DROP-001",
            "remove" => "TOPAL-LIST-REMOVE-INDEX-001",
            _ => unreachable!("known indexed List operation"),
        },
        detail: &format!("index={index}"),
    });
    Ok(value)
}

fn apply_list_insert_at(
    source: &SourceText,
    list: Value,
    boundary: Value,
    boundary_span: Span,
    inserted: Value,
    inserted_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = list
    else {
        unreachable!("insert-at is dispatched only for List")
    };
    let Value::Int(boundary) = boundary else {
        return Err(diagnostic(
            source,
            "E-LIST-BOUNDARY-CLASSIFIER",
            boundary_span,
            "insert-at boundary must be Nat",
        ));
    };
    let Ok(boundary) = usize::try_from(boundary) else {
        return Ok(list_boundary_error(
            source,
            "insert-at",
            boundary_span,
            trace,
        ));
    };
    if boundary > entries.len() {
        return Ok(list_boundary_error(
            source,
            "insert-at",
            boundary_span,
            trace,
        ));
    }
    let inserted_entries = match inserted {
        Value::List {
            element_classifier: classifier,
            entries,
        } => {
            if classifier != element_classifier {
                return Err(diagnostic(
                    source,
                    "E-LIST-INSERT-CLASSIFIER",
                    inserted_span,
                    format!("cannot insert `List {classifier}` into `List {element_classifier}`"),
                ));
            }
            entries
        }
        value if value_has_classifier(&value, &element_classifier) => vec![value],
        value => {
            return Err(diagnostic(
                source,
                "E-LIST-INSERT-CLASSIFIER",
                inserted_span,
                format!(
                    "insert-at requires `{element_classifier}` or `List {element_classifier}`, found `{}`",
                    structural_value_classifier(&value)
                ),
            ));
        }
    };
    let inserted_count = inserted_entries.len();
    entries.splice(boundary..boundary, inserted_entries);
    trace.record(TraceEvent {
        event: "list.inserted",
        rule: "TOPAL-LIST-INSERT-AT-001",
        detail: &format!("boundary={boundary};count={inserted_count}"),
    });
    Ok(Value::List {
        element_classifier,
        entries,
    })
}

fn apply_list_entries_view(list: Value, trace: &mut impl TraceSink) -> Value {
    let Value::List {
        element_classifier,
        entries,
    } = list
    else {
        unreachable!("entries view is dispatched only for List")
    };
    let entry_classifier = format!("IndexedEntry {element_classifier}");
    let entries = entries
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            Value::Record(vec![
                ("index".into(), Value::Int(BigInt::from(index))),
                ("value".into(), value),
            ])
        })
        .collect();
    trace.record(TraceEvent {
        event: "list.entries.viewed",
        rule: "TOPAL-COLLECTION-ENTRIES-001",
        detail: &entry_classifier,
    });
    Value::List {
        element_classifier: entry_classifier,
        entries,
    }
}

fn list_boundary_error(
    source: &SourceText,
    operation: &str,
    span: Span,
    trace: &mut impl TraceSink,
) -> Value {
    let position = source.position(span.start);
    trace.record(TraceEvent {
        event: "list.boundary.rejected",
        rule: "TOPAL-LIST-BOUNDARY-CHECK-001",
        detail: operation,
    });
    Value::Error {
        domain: if operation.starts_with("zip-") {
            format!("root.{operation}(List,List)")
        } else {
            format!("root.{operation}(List,Nat)")
        },
        code: "out-of-range".into(),
        line: position.line,
        column: position.column,
    }
}

fn list_boundary_failure(
    source: &SourceText,
    operation: &str,
    span: Span,
    operand_is_closed: bool,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if operand_is_closed {
        return Err(diagnostic(
            source,
            "E-LIST-BOUNDARY-OUT-OF-RANGE",
            span,
            format!("{operation} operand is outside the List's valid bounds"),
        )
        .with_help(
            "use a boundary no greater than the entry count, or an existing index for remove",
        ));
    }
    Ok(list_boundary_error(source, operation, span, trace))
}

fn apply_list_zip(
    source: &SourceText,
    operation: &str,
    left_classifier: &str,
    left: Vec<Value>,
    right: Value,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier: right_classifier,
        entries: right,
    } = right
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-OPERAND",
            right_span,
            format!("{operation} requires another List"),
        ));
    };
    if operation == "zip-exact" && left.len() != right.len() {
        return Ok(list_boundary_error(source, operation, right_span, trace));
    }
    let entries = left
        .into_iter()
        .zip(right)
        .map(|(left, right)| Value::Tuple(vec![left, right]))
        .collect();
    let pair_classifier = format!("({left_classifier}, {right_classifier})");
    trace.record(TraceEvent {
        event: "list.zipped",
        rule: if operation == "zip-exact" {
            "TOPAL-LIST-ZIP-EXACT-001"
        } else {
            "TOPAL-LIST-ZIP-SHORTEST-001"
        },
        detail: operation,
    });
    Ok(Value::List {
        element_classifier: pair_classifier,
        entries,
    })
}

fn apply_list_unzip(
    source: &SourceText,
    pairs: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List { entries, .. } = pairs else {
        return Err(diagnostic(
            source,
            "E-LIST-UNZIP-SOURCE",
            span,
            "unzip requires a List of two-field products",
        ));
    };
    let mut left = Vec::with_capacity(entries.len());
    let mut right = Vec::with_capacity(entries.len());
    for entry in entries {
        let Value::Tuple(mut fields) = entry else {
            return Err(diagnostic(
                source,
                "E-LIST-UNZIP-ENTRY",
                span,
                "unzip requires every List entry to be a two-field product",
            ));
        };
        if fields.len() != 2 {
            return Err(diagnostic(
                source,
                "E-LIST-UNZIP-ENTRY",
                span,
                "unzip requires every List entry to contain exactly two fields",
            ));
        }
        right.push(fields.pop().expect("two fields"));
        left.push(fields.pop().expect("two fields"));
    }
    let left_classifier = left
        .first()
        .map_or_else(|| "Object".into(), structural_value_classifier);
    let right_classifier = right
        .first()
        .map_or_else(|| "Object".into(), structural_value_classifier);
    trace.record(TraceEvent {
        event: "list.unzipped",
        rule: "TOPAL-LIST-UNZIP-001",
        detail: &format!("count={}", left.len()),
    });
    Ok(Value::Tuple(vec![
        Value::List {
            element_classifier: left_classifier,
            entries: left,
        },
        Value::List {
            element_classifier: right_classifier,
            entries: right,
        },
    ]))
}

fn apply_list_zip_longest(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::Tuple(mut left_fields) = left else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-LEFT",
            span,
            "zip-longest left operand must be `(List, default)`",
        ));
    };
    let Value::Tuple(mut right_fields) = right else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-RIGHT",
            span,
            "zip-longest right operand must be `(List, default)`",
        ));
    };
    if left_fields.len() != 2 || right_fields.len() != 2 {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-OPERAND",
            span,
            "zip-longest operands must each contain a List and its default",
        ));
    }
    let left_default = left_fields.pop().expect("two fields");
    let right_default = right_fields.pop().expect("two fields");
    let Value::List {
        element_classifier: left_classifier,
        entries: left_entries,
    } = left_fields.pop().expect("two fields")
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-LEFT",
            span,
            "first left field must be a List",
        ));
    };
    let Value::List {
        element_classifier: right_classifier,
        entries: right_entries,
    } = right_fields.pop().expect("two fields")
    else {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-RIGHT",
            span,
            "first right field must be a List",
        ));
    };
    if !value_has_classifier(&left_default, &left_classifier)
        || !value_has_classifier(&right_default, &right_classifier)
    {
        return Err(diagnostic(
            source,
            "E-LIST-ZIP-LONGEST-DEFAULT",
            span,
            "each zip-longest default must match its List element classifier",
        ));
    }
    let count = left_entries.len().max(right_entries.len());
    let entries = (0..count)
        .map(|index| {
            Value::Tuple(vec![
                left_entries
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| left_default.clone()),
                right_entries
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| right_default.clone()),
            ])
        })
        .collect();
    trace.record(TraceEvent {
        event: "list.zipped",
        rule: "TOPAL-LIST-ZIP-LONGEST-001",
        detail: &format!("count={count}"),
    });
    Ok(Value::List {
        element_classifier: format!("({left_classifier}, {right_classifier})"),
        entries,
    })
}

fn collect_unordered(
    source: &SourceText,
    operation: &str,
    value: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        entries,
    } = value
    else {
        return Err(diagnostic(
            source,
            "E-UNORDERED-COLLECT-SOURCE",
            span,
            format!("{operation} requires a finite List"),
        ));
    };
    let mut distinct: Vec<(Value, usize)> = Vec::new();
    for entry in entries {
        let mut found = None;
        for (index, (candidate, _)) in distinct.iter().enumerate() {
            if values_equal(candidate.clone(), entry.clone(), trace).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-UNORDERED-COLLECT-EQUALITY",
                    span,
                    format!("`{element_classifier}` must provide equality for {operation}"),
                )
            })? {
                found = Some(index);
                break;
            }
        }
        if let Some(index) = found {
            distinct[index].1 += 1;
        } else {
            distinct.push((entry, 1));
        }
    }
    let count = distinct.len();
    trace.record(TraceEvent {
        event: if operation == "collect-set" {
            "set.collected"
        } else {
            "bag.collected"
        },
        rule: if operation == "collect-set" {
            "TOPAL-SET-COLLECT-001"
        } else {
            "TOPAL-BAG-COLLECT-001"
        },
        detail: &format!("distinct={count}"),
    });
    if operation == "collect-set" {
        Ok(Value::Set {
            element_classifier,
            entries: distinct.into_iter().map(|(value, _)| value).collect(),
        })
    } else {
        Ok(Value::Bag {
            element_classifier,
            entries: distinct,
        })
    }
}

fn collect_map(
    source: &SourceText,
    value: Value,
    policy: &str,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if !matches!(policy, "reject" | "keep-first" | "keep-last") {
        return Err(diagnostic(
            source,
            "E-MAP-COLLISION-POLICY",
            span,
            "collect-map policy must be reject, keep-first, or keep-last",
        ));
    }
    let Value::List { entries, .. } = value else {
        return Err(diagnostic(
            source,
            "E-MAP-COLLECT-SOURCE",
            span,
            "collect-map requires a List of key/value products",
        ));
    };
    let mut mapping: Vec<(Value, Value)> = Vec::new();
    for entry in entries {
        let Value::Tuple(mut pair) = entry else {
            return Err(diagnostic(
                source,
                "E-MAP-COLLECT-ENTRY",
                span,
                "collect-map entries must be two-field products",
            ));
        };
        if pair.len() != 2 {
            return Err(diagnostic(
                source,
                "E-MAP-COLLECT-ENTRY",
                span,
                "collect-map entries must have exactly two fields",
            ));
        }
        let value = pair.pop().expect("two fields");
        let key = pair.pop().expect("two fields");
        let mut collision = None;
        for (index, (candidate, _)) in mapping.iter().enumerate() {
            if values_equal(candidate.clone(), key.clone(), trace).ok_or_else(|| {
                diagnostic(
                    source,
                    "E-MAP-KEY-EQUALITY",
                    span,
                    "map keys must provide equality",
                )
            })? {
                collision = Some(index);
                break;
            }
        }
        match (collision, policy) {
            (Some(_), "reject") => {
                return Err(diagnostic(
                    source,
                    "E-MAP-KEY-COLLISION",
                    span,
                    "collect-map encountered a duplicate key under reject policy",
                ));
            }
            (Some(_), "keep-first") => {}
            (Some(index), "keep-last") => mapping[index].1 = value,
            (None, _) => mapping.push((key, value)),
            _ => unreachable!("validated collision policy"),
        }
    }
    let key_classifier = mapping.first().map_or_else(
        || "Object".into(),
        |(key, _)| structural_value_classifier(key),
    );
    let value_classifier = mapping.first().map_or_else(
        || "Object".into(),
        |(_, value)| structural_value_classifier(value),
    );
    trace.record(TraceEvent {
        event: "map.collected",
        rule: "TOPAL-MAP-COLLECT-001",
        detail: policy,
    });
    Ok(Value::Map {
        key_classifier,
        value_classifier,
        entries: mapping,
    })
}

fn apply_list_value_removal(
    source: &SourceText,
    operation: &str,
    element_classifier: String,
    entries: Vec<Value>,
    target: &Value,
    target_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if !value_has_classifier(target, &element_classifier) {
        let found = structural_value_classifier(target);
        return Err(diagnostic(
            source,
            "E-LIST-REMOVAL-CLASSIFIER",
            target_span,
            format!("{operation} requires `{element_classifier}`, found `{found}`"),
        ));
    }
    let mut removed = false;
    let mut retained = Vec::with_capacity(entries.len());
    for entry in entries {
        let equal = values_equal(entry.clone(), target.clone(), trace).ok_or_else(|| {
            diagnostic(
                source,
                "E-LIST-REMOVAL-EQUALITY",
                target_span,
                format!("`{element_classifier}` does not provide equality required by {operation}"),
            )
        })?;
        if equal && (operation == "remove-all" || !removed) {
            removed = true;
        } else {
            retained.push(entry);
        }
    }
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: if operation == "remove-first" {
            "list.first.removed"
        } else {
            "list.all.removed"
        },
        rule: if operation == "remove-first" {
            "TOPAL-LIST-REMOVE-FIRST-001"
        } else {
            "TOPAL-LIST-REMOVE-ALL-001"
        },
        detail: if removed {
            "removed=true"
        } else {
            "removed=false"
        },
    });
    Ok(Value::List {
        element_classifier,
        entries: retained,
    })
}

fn apply_list_containment(
    source: &SourceText,
    operation: &str,
    element_classifier: &str,
    entries: &[Value],
    right: Value,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let contained = if operation == "contains-entry" {
        if !value_has_classifier(&right, element_classifier) {
            let found = structural_value_classifier(&right);
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-CLASSIFIER",
                right_span,
                format!("contains-entry requires `{element_classifier}`, found `{found}`"),
            ));
        }
        entries.iter().try_fold(false, |found, entry| {
            values_equal(entry.clone(), right.clone(), trace).map(|equal| found || equal)
        })
    } else {
        let Value::List {
            element_classifier: right_classifier,
            entries: pattern,
        } = right
        else {
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-OPERAND",
                right_span,
                format!("{operation} requires another List"),
            ));
        };
        if right_classifier != element_classifier {
            return Err(diagnostic(
                source,
                "E-LIST-CONTAINMENT-CLASSIFIER",
                right_span,
                format!("{operation} requires `List {element_classifier}`, found `List {right_classifier}`"),
            ));
        }
        if operation == "contains-sequence" {
            contains_consecutive(entries, &pattern, trace)
        } else {
            contains_ordered_subsequence(entries, &pattern, trace)
        }
    }
    .ok_or_else(|| {
        diagnostic(
            source,
            "E-LIST-CONTAINMENT-EQUALITY",
            right_span,
            format!("`{element_classifier}` does not provide equality required by {operation}"),
        )
    })?;
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.{operation}({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    let rule = match operation {
        "contains-entry" => "TOPAL-LIST-CONTAINS-ENTRY-001",
        "contains-sequence" => "TOPAL-LIST-CONTAINS-SEQUENCE-001",
        "contains-subsequence" => "TOPAL-LIST-CONTAINS-SUBSEQUENCE-001",
        _ => unreachable!("known List containment operation"),
    };
    trace.record(TraceEvent {
        event: "list.containment.tested",
        rule,
        detail: if contained { "true" } else { "false" },
    });
    Ok(Value::Boolean(contained))
}

fn contains_consecutive(
    entries: &[Value],
    pattern: &[Value],
    trace: &mut impl TraceSink,
) -> Option<bool> {
    if pattern.is_empty() {
        return Some(true);
    }
    entries
        .windows(pattern.len())
        .try_fold(false, |found, window| {
            window
                .iter()
                .zip(pattern)
                .try_fold(true, |equal, (left, right)| {
                    values_equal(left.clone(), right.clone(), trace).map(|item| equal && item)
                })
                .map(|equal| found || equal)
        })
}

fn contains_ordered_subsequence(
    entries: &[Value],
    pattern: &[Value],
    trace: &mut impl TraceSink,
) -> Option<bool> {
    let mut matched = 0;
    for entry in entries {
        if let Some(expected) = pattern.get(matched)
            && values_equal(entry.clone(), expected.clone(), trace)?
        {
            matched += 1;
        }
    }
    Some(matched == pattern.len())
}

#[allow(clippy::too_many_lines)] // Every recursively derived equality remains explicit.
fn values_equal(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<bool> {
    if (extended_rational(&left).is_some() || extended_rational(&right).is_some())
        && exact_to_extended_rational(&left).is_some()
        && exact_to_extended_rational(&right).is_some()
    {
        if extended_rational(&left).is_none() {
            trace_conversion(trace, "Int->Rational:left");
        }
        if extended_rational(&right).is_none() {
            trace_conversion(trace, "Int->Rational:right");
        }
        return Some(exact_to_extended_rational(&left)? == exact_to_extended_rational(&right)?);
    }
    if let (Some(left), Some(right)) = (extended_int(&left), extended_int(&right)) {
        return Some(left == right);
    }
    match (left, right) {
        (
            Value::Refined {
                constraint: left_constraint,
                value: left,
                ..
            },
            Value::Refined {
                constraint: right_constraint,
                value: right,
                ..
            },
        ) if left_constraint == right_constraint => values_equal(*left, *right, trace),
        (Value::Refined { value, .. }, right) => values_equal(*value, right, trace),
        (left, Value::Refined { value, .. }) => values_equal(left, *value, trace),
        (Value::Type(left), Value::Type(right)) | (Value::String(left), Value::String(right)) => {
            Some(left == right)
        }
        (Value::Effects(left), Value::Effects(right)) => Some(left == right),
        (Value::Boolean(left), Value::Boolean(right)) => Some(left == right),
        (Value::Rational(left), Value::Rational(right)) => Some(left == right),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left) == right)
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left == BigRational::from_integer(right))
        }
        (
            Value::Modular {
                type_name: left_type,
                value: left,
                ..
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => Some(left == right),
        (
            Value::List {
                element_classifier: left_classifier,
                entries: left,
            },
            Value::List {
                element_classifier: right_classifier,
                entries: right,
            },
        ) if left_classifier == right_classifier => {
            trace.record(TraceEvent {
                event: "equality.list",
                rule: "TOPAL-TYPE-LIST-EQUALITY-001",
                detail: &left_classifier,
            });
            if left.len() != right.len() {
                return Some(false);
            }
            left.into_iter()
                .zip(right)
                .try_fold(true, |equal, (left, right)| {
                    values_equal(left, right, trace).map(|entry_equal| equal && entry_equal)
                })
        }
        (
            Value::Enum {
                type_name: left_type,
                alternative: left,
            },
            Value::Enum {
                type_name: right_type,
                alternative: right,
            },
        ) if left_type == right_type => Some(left == right),
        (Value::Union(left), Value::Union(right))
            if left.type_name == right.type_name
                && left.supports_equality
                && right.supports_equality =>
        {
            trace.record(TraceEvent {
                event: "equality.sum",
                rule: "TOPAL-TYPE-SUM-EQUALITY-001",
                detail: &left.type_name,
            });
            if left.alternative != right.alternative {
                return Some(false);
            }
            if left.payload_classifier != right.payload_classifier {
                return None;
            }
            match (left.payload, right.payload) {
                (None, None) => Some(true),
                (Some(left), Some(right)) => values_equal(*left, *right, trace),
                _ => Some(false),
            }
        }
        (
            Value::Optional {
                payload_classifier: left_classifier,
                payload: left,
            },
            Value::Optional {
                payload_classifier: right_classifier,
                payload: right,
            },
        ) if left_classifier == right_classifier => {
            trace.record(TraceEvent {
                event: "equality.optional",
                rule: "TOPAL-TYPE-OPTIONAL-EQUALITY-001",
                detail: &left_classifier,
            });
            match (left, right) {
                (None, None) => Some(true),
                (Some(left), Some(right)) => values_equal(*left, *right, trace),
                _ => Some(false),
            }
        }
        (Value::Completed, Value::Completed) | (Value::Unit, Value::Unit) => Some(true),
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => left
            .into_iter()
            .zip(right)
            .try_fold(true, |equal, (left, right)| {
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            }),
        (Value::Record(left), Value::Record(right)) if left.len() == right.len() => {
            left.into_iter().try_fold(true, |equal, (label, left)| {
                let right = right
                    .iter()
                    .find(|(right_label, _)| right_label == &label)
                    .map(|(_, value)| value.clone())?;
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            })
        }
        _ => None,
    }
}

fn trace_conversion(trace: &mut impl TraceSink, detail: &'static str) {
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: "TOPAL-TYPE-CONVERT-001",
        detail,
    });
}

fn apply_int_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Compare
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Range
        | CallableKind::RangeOpen
        | CallableKind::RangeInclusive
        | CallableKind::RangeOpenInclusive => {
            unreachable!("range is dispatched before numeric operations")
        }
        CallableKind::Plus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.+(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.add",
                rule: "TOPAL-NUM-ADD-001",
                detail: "Int",
            });
            Ok(Value::Int(left + right))
        }
        CallableKind::Minus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.subtract",
                rule: "TOPAL-NUM-SUB-001",
                detail: "Int",
            });
            Ok(Value::Int(left - right))
        }
        CallableKind::Multiply => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.*(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.multiply",
                rule: "TOPAL-NUM-MUL-001",
                detail: "Int",
            });
            Ok(Value::Int(left * right))
        }
        CallableKind::Divide => apply_divide(source, left, right, right_span, trace),
        CallableKind::QuotientModulo => {
            apply_quotient_modulo(source, left, right, right_span, trace)
        }
        CallableKind::Modulo => apply_modulo(source, left, &right, right_span, trace),
        CallableKind::Power => apply_power(source, left, right, right_span, trace),
    }
}

fn apply_modulo(
    source: &SourceText,
    left: BigInt,
    right: &BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == &BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        if parse_integer(source.slice(right_span)).is_none() {
            let position = source.position(right_span.start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: "root.%(Int,Int);division-by-zero",
            });
            return Ok(Value::Error {
                domain: "root.%(Int,Int)".to_owned(),
                code: "division-by-zero".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident modulo by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.%(Int,Int)",
    });
    let remainder = euclidean_remainder(left, right);
    trace.record(TraceEvent {
        event: "evaluation.modulo",
        rule: "TOPAL-NUM-INT-MODULO-001",
        detail: "Euclidean",
    });
    Ok(Value::Int(remainder))
}
