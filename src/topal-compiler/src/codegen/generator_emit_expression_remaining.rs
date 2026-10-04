impl Generator<'_> {
    #[allow(clippy::if_not_else, clippy::too_many_lines)] // Collection and decision lowering remain exhaustive and ordered.
    fn emit_expression_remaining(
        &mut self,
        expression: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        match &expression.kind {
            CompilerExpressionKind::ListConcat { left, right } => {
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked List concatenation retains its List result")
                };
                let int_pair = compiler_integer_pair(element.as_ref());
                let three_pointer = compiler_three_pointer_tuple(element.as_ref());
                let int_string_pair = compiler_int_string_pair(element.as_ref());
                let int_list_pair = compiler_int_list_pair(element.as_ref());
                let four_pointer = compiler_int_int_string_int_tuple(element.as_ref());
                let string = element.as_ref() == &CompilerType::String;
                if int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else if three_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntTriple);
                } else if int_string_pair || int_list_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntStringPair);
                } else if four_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntStringIntTuple);
                } else if string {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::String);
                } else {
                    self.list_int_runtime_fragments
                        .insert(ListIntRuntimeFragment::Core);
                }
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.{}.concat(ptr {}, ptr {})",
                            if four_pointer {
                                "int-int-string-int"
                            } else if int_string_pair || int_list_pair {
                                "int-string"
                            } else if int_pair {
                                "int.pair"
                            } else if three_pointer {
                                "int.triple"
                            } else if string {
                                "string"
                            } else {
                                "int"
                            },
                            left.list_pointer(),
                            right.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListReverse(value) => {
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked List reversal retains its List result")
                };
                let helper = if compiler_int_string_pair(element.as_ref()) {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntStringPair);
                    "int-string"
                } else {
                    self.list_int_runtime_fragments
                        .insert(ListIntRuntimeFragment::Core);
                    "int"
                };
                let value = self.emit_expression(value, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.{helper}.reverse(ptr {})",
                            value.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListEntryCount(value) => {
                let unit = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Unit
                );
                let completed = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Completed
                );
                let boolean = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Boolean
                );
                let effect = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Effect
                );
                let type_value = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Type
                );
                let enumeration = matches!(
                    &value.value_type,
                    CompilerType::List(element) if matches!(element.as_ref(), CompilerType::Enum(_))
                );
                let modular = matches!(
                    &value.value_type,
                    CompilerType::List(element) if matches!(element.as_ref(), CompilerType::Modular(_))
                );
                let optional_int = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if matches!(element.as_ref(), CompilerType::Optional(payload)
                            if payload.as_ref() == &CompilerType::Int)
                );
                let optional_rational = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if matches!(element.as_ref(), CompilerType::Optional(payload)
                            if payload.as_ref() == &CompilerType::Rational)
                );
                let optional_string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if matches!(element.as_ref(), CompilerType::Optional(payload)
                            if payload.as_ref() == &CompilerType::String)
                );
                let int_pair = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_integer_pair(element.as_ref())
                            || compiler_list_integer_pair(element.as_ref())
                            || compiler_list_string_rational_pair(element.as_ref())
                            || compiler_rational_nat_pair(element.as_ref())
                            || compiler_rational_pair(element.as_ref())
                );
                let int_triple = matches!(
                    &value.value_type,
                    CompilerType::List(element) if compiler_three_pointer_tuple(element.as_ref())
                );
                let int_int_boolean_pair = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_int_int_boolean_pair(element.as_ref())
                );
                let four_pointer = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_int_int_string_int_tuple(element.as_ref())
                );
                let int_range = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if matches!(element.as_ref(), CompilerType::Range(endpoint)
                            if endpoint.as_ref() == &CompilerType::Int)
                );
                let int_string_pair = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_int_string_pair(element.as_ref())
                            || compiler_int_list_pair(element.as_ref())
                );
                let string_int_pair = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_string_int_pair(element.as_ref())
                );
                let string_pair = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_string_pair(element.as_ref())
                );
                let comparison = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Comparison
                );
                let error_code = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::ErrorCode
                );
                let string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if matches!(element.as_ref(), CompilerType::Character | CompilerType::String)
                );
                let rational = matches!(
                    &value.value_type,
                    CompilerType::List(element) if element.as_ref() == &CompilerType::Rational
                );
                let nested_int_string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_nested_int_string_list_element(element)
                );
                let nested_int = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_nested_int_list_element(element)
                );
                if unit {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Unit);
                } else if completed {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Completed);
                } else if effect {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Effect);
                } else if type_value {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Type);
                } else if enumeration {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Enum);
                } else if modular {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Modular);
                } else if optional_int {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::OptionalInt);
                } else if optional_rational {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::OptionalRational);
                } else if optional_string {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::OptionalString);
                } else if int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else if int_triple {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntTriple);
                } else if int_int_boolean_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntBooleanPair);
                } else if four_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntStringIntTuple);
                } else if int_range {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntRange);
                } else if int_string_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntStringPair);
                } else if string_int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::StringIntPair);
                } else if string_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::StringPair);
                } else if boolean {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Boolean);
                } else if comparison {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Comparison);
                } else if error_code {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::ErrorCode);
                } else if rational {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Rational);
                } else if string {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::String);
                } else {
                    self.list_int_runtime_fragments
                        .insert(if nested_int_string {
                            ListIntRuntimeFragment::NestedIntStringCore
                        } else if nested_int {
                            ListIntRuntimeFragment::NestedIntCore
                        } else {
                            ListIntRuntimeFragment::Core
                        });
                }
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.{}.entry.count(ptr {})",
                        if unit {
                            "unit"
                        } else if completed {
                            "completed"
                        } else if effect {
                            "effect"
                        } else if type_value {
                            "type"
                        } else if enumeration {
                            "enum"
                        } else if modular {
                            "modular"
                        } else if optional_int {
                            "optional.int"
                        } else if optional_rational {
                            "optional.rational"
                        } else if optional_string {
                            "optional.string"
                        } else if int_pair {
                            "int.pair"
                        } else if int_triple {
                            "int.triple"
                        } else if int_int_boolean_pair {
                            "int-int-boolean-pair"
                        } else if four_pointer {
                            "int-int-string-int"
                        } else if int_range {
                            "int.range"
                        } else if int_string_pair {
                            "int-string"
                        } else if string_int_pair {
                            "string-int"
                        } else if string_pair {
                            "string-pair"
                        } else if boolean {
                            "boolean"
                        } else if comparison {
                            "comparison"
                        } else if error_code {
                            "error.code"
                        } else if rational {
                            "rational"
                        } else if string {
                            "string"
                        } else if nested_int_string {
                            "nested.int-string"
                        } else if nested_int {
                            "nested.int"
                        } else {
                            "int"
                        },
                        value.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListEmptyPredicate(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!("icmp eq ptr {}, null", value.list_pointer()),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListFirst(value)
            | CompilerExpressionKind::ListRest(value)
            | CompilerExpressionKind::ListUncons(value) => {
                let int_pair = matches!(&value.value_type, CompilerType::List(element)
                    if compiler_integer_pair(element.as_ref())
                        || compiler_list_integer_pair(element.as_ref())
                        || compiler_list_string_rational_pair(element.as_ref()));
                let int_pointer_pair = matches!(&value.value_type, CompilerType::List(element)
                    if compiler_int_string_pair(element.as_ref())
                        || compiler_int_list_pair(element.as_ref()));
                let int_triple = matches!(&value.value_type, CompilerType::List(element)
                    if compiler_three_pointer_tuple(element.as_ref()));
                let four_pointer = matches!(&value.value_type, CompilerType::List(element)
                    if compiler_int_int_string_int_tuple(element.as_ref()));
                let boolean = matches!(&value.value_type, CompilerType::List(element)
                    if element.as_ref() == &CompilerType::Boolean);
                let nested_int_string = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_nested_int_string_list_element(element)
                );
                let nested_int = matches!(
                    &value.value_type,
                    CompilerType::List(element)
                        if compiler_nested_int_list_element(element)
                );
                if int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else if int_triple {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntTriple);
                } else if int_pointer_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntStringPair);
                } else if four_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntStringIntTuple);
                } else if boolean {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::Boolean);
                } else {
                    self.list_int_runtime_fragments
                        .insert(if nested_int_string {
                            ListIntRuntimeFragment::NestedIntStringCore
                        } else if nested_int {
                            ListIntRuntimeFragment::NestedIntCore
                        } else {
                            ListIntRuntimeFragment::Core
                        });
                }
                let operation = match &expression.kind {
                    CompilerExpressionKind::ListFirst(_) => "first",
                    CompilerExpressionKind::ListRest(_) => "rest",
                    CompilerExpressionKind::ListUncons(_) => "uncons",
                    _ => unreachable!(),
                };
                let value = self.emit_expression(value, body, environment);
                let CompilerType::Optional(payload) = &expression.value_type else {
                    unreachable!("checked List projection returns Optional")
                };
                LlValue::Optional {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.{}.{operation}(ptr {})",
                            if int_pointer_pair {
                                "int-string"
                            } else if four_pointer {
                                "int-int-string-int"
                            } else if boolean {
                                "boolean"
                            } else if int_pair {
                                "int.pair"
                            } else if int_triple {
                                "int.triple"
                            } else if nested_int_string {
                                "nested.int-string"
                            } else if nested_int {
                                "nested.int"
                            } else {
                                "int"
                            },
                            value.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: payload.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListMap {
                list,
                parameters,
                body: action,
            } => self.emit_list_map(list, parameters, action, body, environment, expression.span),
            CompilerExpressionKind::ListSelect {
                list,
                parameters,
                body: predicate,
            } => self.emit_list_select(
                list,
                parameters,
                predicate,
                body,
                environment,
                expression.span,
                false,
                false,
            ),
            CompilerExpressionKind::ListRangeSelect {
                list,
                range,
                indexes,
            } => {
                let int_pair = matches!(&list.value_type, CompilerType::List(element)
                    if compiler_integer_pair(element.as_ref())
                        || compiler_list_integer_pair(element.as_ref())
                        || compiler_list_string_rational_pair(element.as_ref()));
                let int_pointer_pair = matches!(&list.value_type, CompilerType::List(element)
                    if compiler_int_string_pair(element.as_ref())
                        || compiler_int_list_pair(element.as_ref()));
                let int_triple = matches!(&list.value_type, CompilerType::List(element)
                    if compiler_three_pointer_tuple(element.as_ref()));
                let four_pointer = matches!(&list.value_type, CompilerType::List(element)
                    if compiler_int_int_string_int_tuple(element.as_ref()));
                if int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else if int_triple {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntTriple);
                } else if int_pointer_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntStringPair);
                } else if four_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntStringIntTuple);
                } else {
                    self.list_int_runtime_fragments
                        .insert(ListIntRuntimeFragment::RangeSelection);
                }
                let list = self.emit_expression(list, body, environment);
                let range = self.emit_expression(range, body, environment);
                let operation = if *indexes { "index" } else { "value" };
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked List range selection retains its List result")
                };
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.{}.select.{operation}.range(ptr {}, ptr {})",
                            if four_pointer {
                                "int-int-string-int"
                            } else if int_pointer_pair {
                                "int-string"
                            } else if int_pair {
                                "int.pair"
                            } else if int_triple {
                                "int.triple"
                            } else {
                                "int"
                            },
                            list.list_pointer(),
                            range.range().0
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListForeach {
                list,
                parameter,
                body: action,
            } => {
                self.emit_list_foreach(list, parameter, action, body, environment, expression.span)
            }
            CompilerExpressionKind::ListInsertAt {
                list,
                boundary,
                inserted,
                inserts_list,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                let inserted = self.emit_expression(inserted, body, environment);
                let inserted = if *inserts_list {
                    inserted.list_pointer().to_owned()
                } else {
                    let singleton = body.instruction(
                        "call ptr @topal.platform.allocate(i64 16)",
                        expression.span,
                        &mut self.debug,
                    );
                    body.effect(
                        &format!("store ptr {}, ptr {singleton}, align 8", inserted.integer()),
                        expression.span,
                        &mut self.debug,
                    );
                    let next = body.instruction(
                        &format!("getelementptr i8, ptr {singleton}, i64 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    body.effect(
                        &format!("store ptr null, ptr {next}, align 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    singleton
                };
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.insert.at(ptr {}, i64 {boundary}, ptr {inserted})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListInsertEverywhere { list, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::NestedIntCore);
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.nested.int.insert.everywhere(ptr {}, ptr {})",
                            list.list_pointer(),
                            value.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::List(Box::new(CompilerType::Int)),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListCartesianStringInt { left, right } => {
                self.scalar_list_runtime_fragments
                    .insert(ScalarListRuntimeFragment::String);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.scalar_list_runtime_fragments
                    .insert(ScalarListRuntimeFragment::StringIntPair);
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.string-int.cartesian(ptr {}, ptr {})",
                            left.list_pointer(),
                            right.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Tuple(vec![CompilerType::String, CompilerType::Int]),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListIndexOperation {
                list,
                index,
                operation,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                let operation_name = match operation {
                    CompilerListIndexOperation::Split => "split.at",
                    CompilerListIndexOperation::Take => "take",
                    CompilerListIndexOperation::Drop => "drop",
                    CompilerListIndexOperation::Remove => "remove.index",
                };
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.int.{operation_name}(ptr {}, i64 {index})",
                        list.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                if *operation == CompilerListIndexOperation::Split {
                    let suffix_address = body.instruction(
                        &format!("getelementptr i8, ptr {value}, i64 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    LlValue::Tuple(vec![
                        LlValue::List {
                            value: body.instruction(
                                &format!("load ptr, ptr {value}, align 8"),
                                expression.span,
                                &mut self.debug,
                            ),
                            element: CompilerType::Int,
                            function_captures: Vec::new(),
                        },
                        LlValue::List {
                            value: body.instruction(
                                &format!("load ptr, ptr {suffix_address}, align 8"),
                                expression.span,
                                &mut self.debug,
                            ),
                            element: CompilerType::Int,
                            function_captures: Vec::new(),
                        },
                    ])
                } else {
                    LlValue::List {
                        value,
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    }
                }
            }
            CompilerExpressionKind::ListRemoveIndexRange { list, start, end } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.remove.index.range(ptr {}, i64 {start}, i64 {end})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Int,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListReject {
                list,
                parameters,
                predicate,
                indexes,
            } => self.emit_list_select(
                list,
                parameters,
                predicate,
                body,
                environment,
                expression.span,
                true,
                *indexes,
            ),
            CompilerExpressionKind::ListZip {
                left,
                right,
                operation,
                left_default,
                right_default,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Core);
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let left = self.emit_expression(left, body, environment);
                let left_default = left_default
                    .as_deref()
                    .map(|value| self.emit_expression(value, body, environment));
                let right = self.emit_expression(right, body, environment);
                let right_default = right_default
                    .as_deref()
                    .map(|value| self.emit_expression(value, body, environment));
                let pair = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]);
                match operation {
                    CompilerListZipOperation::Exact => {
                        let domain = self.emit_string_value(
                            "root.zip-exact(List,List)",
                            body,
                            expression.span,
                        );
                        let source =
                            self.emit_string_value(self.source_name, body, expression.span);
                        let position = self.program.source.position(expression.span.start);
                        LlValue::Result {
                            value: body.instruction(
                                &format!(
                                    "call ptr @topal.runtime.list.int.zip.exact(ptr {}, ptr {}, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                                    left.list_pointer(),
                                    right.list_pointer(),
                                    position.line,
                                    position.column
                                ),
                                expression.span,
                                &mut self.debug,
                            ),
                            success: CompilerType::List(Box::new(pair)),
                            function_captures: Vec::new(),
                        }
                    }
                    CompilerListZipOperation::Shortest => LlValue::List {
                        value: body.instruction(
                            &format!(
                                "call ptr @topal.runtime.list.int.zip.shortest(ptr {}, ptr {})",
                                left.list_pointer(),
                                right.list_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: pair,
                        function_captures: Vec::new(),
                    },
                    CompilerListZipOperation::Longest => LlValue::List {
                        value: body.instruction(
                            &format!(
                                "call ptr @topal.runtime.list.int.zip.longest(ptr {}, ptr {}, ptr {}, ptr {})",
                                left.list_pointer(),
                                left_default
                                    .as_ref()
                                    .expect("checked longest zip has left default")
                                    .integer(),
                                right.list_pointer(),
                                right_default
                                    .as_ref()
                                    .expect("checked longest zip has right default")
                                    .integer()
                            ),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: pair,
                        function_captures: Vec::new(),
                    },
                }
            }
            CompilerExpressionKind::ListUnzip(pairs) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let pairs = self.emit_expression(pairs, body, environment);
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.int.unzip(ptr {})",
                        pairs.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                let right_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![
                    LlValue::List {
                        value: body.instruction(
                            &format!("load ptr, ptr {value}, align 8"),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    },
                    LlValue::List {
                        value: body.instruction(
                            &format!("load ptr, ptr {right_address}, align 8"),
                            expression.span,
                            &mut self.debug,
                        ),
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    },
                ])
            }
            CompilerExpressionKind::ListEntries(list) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.entries(ptr {})",
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Record(vec![
                        ("index".into(), CompilerType::Int),
                        ("value".into(), CompilerType::Int),
                    ]),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListCollectString(list) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Sequence);
                let list = self.emit_expression(list, body, environment);
                let empty = self.emit_string_value("", body, expression.span);
                LlValue::String(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.list.string.collect(ptr {}, ptr {empty})",
                        list.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ContainerCollect {
                source,
                kind,
                map_policy,
                map_keys,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let source = self.emit_expression(source, body, environment);
                let (function_captures, function_capture_keys) =
                    match (&expression.value_type, &source) {
                        (
                            CompilerType::Array { element, .. },
                            LlValue::List {
                                function_captures, ..
                            },
                        ) if element.as_ref() == &CompilerType::Function => {
                            (function_captures.clone(), Vec::new())
                        }
                        (
                            CompilerType::Map { key, value },
                            LlValue::List {
                                function_captures, ..
                            },
                        ) if key.as_ref() == &CompilerType::String
                            && value.as_ref() == &CompilerType::Function =>
                        {
                            let keys = map_keys
                                .as_ref()
                                .expect("checked Map Function collection retains exact keys");
                            debug_assert_eq!(keys.len(), function_captures.len());
                            let mut positions: BTreeMap<String, usize> = BTreeMap::new();
                            let mut unique_keys = Vec::new();
                            let mut unique_captures: Vec<Vec<(String, LlValue)>> = Vec::new();
                            for (key, captures) in keys.iter().zip(function_captures) {
                                if let Some(index) = positions.get(key).copied() {
                                    if matches!(
                                        map_policy,
                                        Some(CompilerMapCollisionPolicy::KeepLast)
                                    ) {
                                        unique_captures[index].clone_from(captures);
                                    }
                                } else {
                                    positions.insert(key.clone(), unique_keys.len());
                                    unique_keys.push(key.clone());
                                    unique_captures.push(captures.clone());
                                }
                            }
                            (unique_captures, unique_keys)
                        }
                        _ => (Vec::new(), Vec::new()),
                    };
                let call = match kind {
                    CompilerContainerKind::Array
                        if matches!(
                            &expression.value_type,
                            CompilerType::Array { element, .. }
                                if element.as_ref() == &CompilerType::Function
                        ) =>
                    {
                        format!(
                            "call ptr @topal.runtime.container.array.function.collect(ptr {})",
                            source.list_pointer()
                        )
                    }
                    CompilerContainerKind::Array => {
                        format!(
                            "call ptr @topal.runtime.container.array.int.collect(ptr {})",
                            source.list_pointer()
                        )
                    }
                    CompilerContainerKind::Set => format!(
                        "call ptr @topal.runtime.container.set.int.collect(ptr {})",
                        source.list_pointer()
                    ),
                    CompilerContainerKind::Bag => format!(
                        "call ptr @topal.runtime.container.bag.int.collect(ptr {})",
                        source.list_pointer()
                    ),
                    CompilerContainerKind::Map => {
                        let keep_last =
                            matches!(map_policy, Some(CompilerMapCollisionPolicy::KeepLast));
                        let helper = if matches!(
                            &expression.value_type,
                            CompilerType::Map { key, value }
                                if key.as_ref() == &CompilerType::String
                                    && value.as_ref() == &CompilerType::Function
                        ) {
                            "topal.runtime.container.map.string-function.collect"
                        } else {
                            "topal.runtime.container.map.string-int.collect"
                        };
                        format!(
                            "call ptr @{helper}(ptr {}, i1 {keep_last})",
                            source.list_pointer()
                        )
                    }
                };
                LlValue::Container {
                    value: body.instruction(&call, expression.span, &mut self.debug),
                    value_type: expression.value_type.clone(),
                    function_captures,
                    function_capture_keys,
                }
            }
            CompilerExpressionKind::ContainerEntryCount(container) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let container = self.emit_expression(container, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.container.entry.count(ptr {})",
                        container.container_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ContainerEmpty(container) => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let container = self.emit_expression(container, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.container.empty(ptr {})",
                        container.container_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ArrayAt { array, index } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let array = self.emit_expression(array, body, environment);
                let LlValue::Container {
                    value: array,
                    value_type,
                    function_captures,
                    ..
                } = array
                else {
                    unreachable!("checked Array access retains its container value")
                };
                let CompilerType::Array { element, .. } = value_type else {
                    unreachable!("checked Array access retains its Array classifier")
                };
                let helper = match element.as_ref() {
                    CompilerType::Int => "topal.runtime.container.array.int.at",
                    CompilerType::Function => "topal.runtime.container.array.function.at",
                    _ => unreachable!("checked Array access has an admitted element classifier"),
                };
                let captures = if element.as_ref() == &CompilerType::Function {
                    function_captures.get(*index).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                };
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @{helper}(ptr {array}, i64 {index})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: element.as_ref().clone(),
                    function_captures: captures,
                }
            }
            CompilerExpressionKind::SetContains { set, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let set = self.emit_expression(set, body, environment);
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.container.set.int.contains(ptr {}, ptr {})",
                        set.container_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::BagMultiplicity { bag, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let bag = self.emit_expression(bag, body, environment);
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.container.bag.int.multiplicity(ptr {}, ptr {})",
                        bag.container_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::MapLookup {
                mapping,
                key,
                exact_key,
            } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::FundamentalContainers);
                let mapping = self.emit_expression(mapping, body, environment);
                let key = self.emit_expression(key, body, environment);
                let LlValue::Container {
                    value: mapping,
                    value_type,
                    function_captures,
                    function_capture_keys,
                } = mapping
                else {
                    unreachable!("checked Map lookup retains its container value")
                };
                let CompilerType::Map {
                    key: map_key,
                    value: map_value,
                } = value_type
                else {
                    unreachable!("checked Map lookup retains its Map classifier")
                };
                debug_assert_eq!(map_key.as_ref(), &CompilerType::String);
                let helper = match map_value.as_ref() {
                    CompilerType::Int => "topal.runtime.container.map.string-int.lookup",
                    CompilerType::Function => "topal.runtime.container.map.string-function.lookup",
                    _ => unreachable!("checked Map lookup has an admitted value classifier"),
                };
                let captures = if map_value.as_ref() == &CompilerType::Function {
                    let exact_key = exact_key
                        .as_ref()
                        .expect("checked Map Function lookup retains an exact key");
                    function_capture_keys
                        .iter()
                        .position(|candidate| candidate == exact_key)
                        .and_then(|index| function_captures.get(index).cloned())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @{helper}(ptr {mapping}, ptr {})", key.string()),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: map_value.as_ref().clone(),
                    function_captures: captures,
                }
            }
            CompilerExpressionKind::ListFold {
                list,
                initial,
                parameters,
                body: action,
            } => self.emit_list_fold(
                list,
                initial,
                parameters,
                action,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ErrorField { error, field } => {
                let error_span = error.span;
                let error = self.emit_expression(error, body, environment);
                let error = match error {
                    LlValue::Error(error) => error,
                    LlValue::Result { value, .. } => {
                        let is_error = body.instruction(
                            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
                            error_span,
                            &mut self.debug,
                        );
                        let valid = body.label("error.field.valid");
                        let invalid = body.label("error.field.invalid");
                        let location = self.debug.location(error_span, body.subprogram);
                        body.terminator(
                            &format!("br i1 {is_error}, label %{valid}, label %{invalid}"),
                            location,
                        );
                        body.start_block(&invalid);
                        body.effect(
                            "call void @topal.platform.exit(i64 70)",
                            error_span,
                            &mut self.debug,
                        );
                        body.terminator("unreachable", location);
                        body.start_block(&valid);
                        body.instruction(
                            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
                            error_span,
                            &mut self.debug,
                        )
                    }
                    _ => unreachable!("checked field subject contains an Error"),
                };
                match field {
                    CompilerErrorField::Code => LlValue::ErrorCode(body.instruction(
                        &format!("call i32 @topal.runtime.error.code(ptr {error})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    CompilerErrorField::Domain => LlValue::ErrorDomain(body.instruction(
                        &format!("call ptr @topal.runtime.error.domain(ptr {error})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    CompilerErrorField::Detail => LlValue::Optional {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.error.detail(ptr {error})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::String,
                        function_captures: Vec::new(),
                    },
                    CompilerErrorField::Cause => LlValue::Optional {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.error.cause(ptr {error})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::Error,
                        function_captures: Vec::new(),
                    },
                    CompilerErrorField::Source => LlValue::Optional {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.error.source(ptr {error})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::SourceLocation,
                        function_captures: Vec::new(),
                    },
                }
            }
            CompilerExpressionKind::Validate {
                operation,
                value,
                error_span,
            } => self.emit_validation(
                *operation,
                value,
                *error_span,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::Fallible {
                operation,
                left,
                right,
                error_span,
            } => self.emit_fallible(
                *operation,
                left,
                right,
                *error_span,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::RangeLower(operand)
            | CompilerExpressionKind::RangeUpper(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let operation = if matches!(&expression.kind, CompilerExpressionKind::RangeLower(_))
                {
                    "lower"
                } else {
                    "upper"
                };
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.range.{operation}(ptr {})",
                        emitted.range().0
                    ),
                    expression.span,
                    &mut self.debug,
                );
                match expression.value_type {
                    CompilerType::Int | CompilerType::InfiniteInt | CompilerType::InfiniteNat => {
                        LlValue::Int(value)
                    }
                    CompilerType::Rational | CompilerType::InfiniteRational => {
                        LlValue::Rational(value)
                    }
                    _ => unreachable!("checked Range bound retains its endpoint type"),
                }
            }
            CompilerExpressionKind::RangeLowerInclusive(operand)
            | CompilerExpressionKind::RangeUpperInclusive(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let operation = if matches!(
                    &expression.kind,
                    CompilerExpressionKind::RangeLowerInclusive(_)
                ) {
                    "lower.inclusive"
                } else {
                    "upper.inclusive"
                };
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{operation}(ptr {})",
                        emitted.range().0
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::RangeEmpty(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let (value, endpoint) = emitted.range();
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{}.empty(ptr {value})",
                        numeric_domain(endpoint)
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::Not(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                let operand = emitted.boolean();
                LlValue::Boolean(body.instruction(
                    &format!("xor i1 {operand}, true"),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::Binary {
                operation,
                left,
                right,
            } => {
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                self.emit_binary(*operation, &left, &right, body, expression.span)
            }
            CompilerExpressionKind::Call { symbol, arguments } => {
                let mut argument_environment = environment.clone();
                let values = arguments
                    .iter()
                    .map(|argument| {
                        let value = self.emit_expression(argument, body, &argument_environment);
                        extend_function_capture_environment(&mut argument_environment, &value);
                        value
                    })
                    .collect::<Vec<_>>();
                let target = self
                    .program
                    .functions
                    .iter()
                    .find(|function| function.symbol == *symbol)
                    .expect("checked call target has a generated function");
                let parameter_types = target
                    .parameters
                    .iter()
                    .map(|parameter| parameter.value_type.clone())
                    .collect::<Vec<_>>();
                let result_captures = target.result_captures.clone();
                let function_return_type = function_llvm_return_type(target);
                debug_assert_eq!(values.len(), parameter_types.len());
                let mut machine_arguments = Vec::with_capacity(values.len());
                for (value, value_type) in values.iter().zip(&parameter_types) {
                    machine_arguments.push(self.emit_machine_operand(
                        value,
                        value_type,
                        body,
                        expression.span,
                    ));
                }
                let arguments = machine_arguments.join(", ");
                if !result_captures.is_empty() {
                    let aggregate = body.instruction(
                        &format!("call fastcc {function_return_type} @{symbol}({arguments})"),
                        expression.span,
                        &mut self.debug,
                    );
                    let source_value = body.instruction(
                        &format!("extractvalue {function_return_type} {aggregate}, 0"),
                        expression.span,
                        &mut self.debug,
                    );
                    let mut result = self.emit_extracted_machine_value(
                        &source_value,
                        &expression.value_type,
                        body,
                        expression.span,
                    );
                    for (index, capture) in result_captures.iter().enumerate() {
                        let value = body.instruction(
                            &format!(
                                "extractvalue {function_return_type} {aggregate}, {}",
                                index + 1
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                        let value = self.emit_extracted_machine_value(
                            &value,
                            &capture.value_type,
                            body,
                            expression.span,
                        );
                        attach_function_capture(
                            &mut result,
                            &capture.path,
                            compiler_function_result_capture_storage(
                                symbol,
                                expression.span,
                                index,
                            ),
                            value,
                        );
                    }
                    result
                } else {
                    match expression.value_type {
                        CompilerType::Unit => {
                            body.effect(
                                &format!("call fastcc void @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            LlValue::Unit
                        }
                        CompilerType::Completed => LlValue::Completed(body.instruction(
                            &format!("call fastcc i8 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Effect => LlValue::Effect(body.instruction(
                            &format!("call fastcc i8 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Type => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: fundamental_type_enumeration(),
                        },
                        CompilerType::Scope => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: scope_enumeration(),
                        },
                        CompilerType::Function => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: function_value_enumeration(self.program),
                        },
                        CompilerType::Identity
                        | CompilerType::TypeView
                        | CompilerType::FunctionView
                        | CompilerType::LanguageContext
                        | CompilerType::Capability
                        | CompilerType::NativeSerializer(_)
                        | CompilerType::ExternalMetadata
                        | CompilerType::SerializationStream(_)
                        | CompilerType::Constraint
                        | CompilerType::Refined { .. }
                        | CompilerType::TraversalControl(_)
                        | CompilerType::TaskResponse(_)
                        | CompilerType::Task(_)
                        | CompilerType::ExternalLocation(_) => {
                            unreachable!("checked functions do not return this static object kind")
                        }
                        CompilerType::Generator(ref generator) => LlValue::Generator {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            generator: generator.clone(),
                            captured_initial: None,
                            captured_additional_initials: Vec::new(),
                        },
                        CompilerType::Version => {
                            unreachable!("Version function results are not admitted")
                        }
                        CompilerType::Boolean => LlValue::Boolean(body.instruction(
                            &format!("call fastcc i1 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Int
                        | CompilerType::Nat
                        | CompilerType::InfiniteInt
                        | CompilerType::InfiniteNat => LlValue::Int(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Modular(ref modular) => LlValue::Modular {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            modular: modular.clone(),
                        },
                        CompilerType::Rational | CompilerType::InfiniteRational => {
                            LlValue::Rational(body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ))
                        }
                        CompilerType::Comparison => LlValue::Comparison(body.instruction(
                            &format!("call fastcc i32 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::ErrorCode => LlValue::ErrorCode(body.instruction(
                            &format!("call fastcc i32 @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Enum(ref enumeration) => LlValue::Enum {
                            value: body.instruction(
                                &format!("call fastcc i32 @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            enumeration: enumeration.clone(),
                        },
                        CompilerType::Error => LlValue::Error(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::ErrorDomain => LlValue::ErrorDomain(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::SourceLocation => LlValue::SourceLocation(body.instruction(
                            &format!("call fastcc ptr @{symbol}({arguments})"),
                            expression.span,
                            &mut self.debug,
                        )),
                        CompilerType::Character | CompilerType::String => {
                            LlValue::String(body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ))
                        }
                        CompilerType::Range(ref endpoint) => LlValue::Range {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            endpoint: endpoint.as_ref().clone(),
                        },
                        CompilerType::Result(ref success) => LlValue::Result {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            success: success.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        CompilerType::Optional(ref payload) => LlValue::Optional {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            payload: payload.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        CompilerType::List(ref element) => LlValue::List {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            element: element.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        CompilerType::Array { .. }
                        | CompilerType::Set(_)
                        | CompilerType::Bag(_)
                        | CompilerType::Map { .. } => LlValue::Container {
                            value: body.instruction(
                                &format!("call fastcc ptr @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            ),
                            value_type: expression.value_type.clone(),
                            function_captures: Vec::new(),
                            function_capture_keys: Vec::new(),
                        },
                        CompilerType::Tuple(ref field_types) => {
                            let aggregate_type = llvm_value_type(&expression.value_type);
                            let aggregate = body.instruction(
                                &format!("call fastcc {aggregate_type} @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            self.emit_tuple_extract(&aggregate, field_types, body, expression.span)
                        }
                        CompilerType::Record(ref field_types) => {
                            let aggregate_type = llvm_value_type(&expression.value_type);
                            let aggregate = body.instruction(
                                &format!("call fastcc {aggregate_type} @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            self.emit_record_extract(&aggregate, field_types, body, expression.span)
                        }
                        CompilerType::Sum(ref sum) => {
                            let aggregate_type = llvm_value_type(&expression.value_type);
                            let aggregate = body.instruction(
                                &format!("call fastcc {aggregate_type} @{symbol}({arguments})"),
                                expression.span,
                                &mut self.debug,
                            );
                            self.emit_sum_extract(&aggregate, sum, body, expression.span)
                        }
                    }
                }
            }
            CompilerExpressionKind::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => self.emit_boolean_decision(
                subject,
                when_true,
                when_false,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::OrderedComparisonDecision {
                subject,
                rules,
                otherwise,
            } => self.emit_ordered_comparison_decision(
                subject,
                rules,
                otherwise,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ComparisonValueDecision {
                subject,
                when_less,
                when_equal,
                when_greater,
            } => self.emit_comparison_value_decision(
                subject,
                when_less,
                when_equal,
                when_greater,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::EnumDecision {
                subject,
                rules,
                otherwise,
            } => self.emit_enum_decision(
                subject,
                rules,
                otherwise.as_deref(),
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::SumDecision {
                subject,
                rules,
                otherwise,
            } => self.emit_sum_decision(
                subject,
                rules,
                otherwise.as_deref(),
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ResultDecision {
                subject,
                ok_binding,
                ok_binding_span,
                ok_action,
                error_codes,
                error_fallback,
            } => self.emit_result_decision(
                subject,
                ok_binding,
                *ok_binding_span,
                ok_action,
                error_codes,
                error_fallback.as_ref(),
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::OptionalDecision {
                subject,
                some_binding,
                some_action,
                none_action,
            } => self.emit_optional_decision(
                subject,
                some_binding.as_ref(),
                some_action,
                none_action,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::ListDecision {
                subject,
                entry_bindings,
                entry_action,
                empty_action,
            } => self.emit_list_decision(
                subject,
                entry_bindings.as_ref(),
                entry_action,
                empty_action,
                body,
                environment,
                expression.span,
            ),
            _ => unreachable!("expression handled by primary emitter"),
        }
    }
}
