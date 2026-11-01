/// Extract the actual ordinary result Record by semantic name. This private
/// two-word ABI is separately qualified; no source object escapes invocation.
fn emit_owned_identity_bridge(unit:&crate::systems_numeric_library::CheckedNumericLibrary)->String {
    let mut bridge=format!("define {{ i64, i64 }} @{}(i64 %cursor) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{\nentry:\n  %zero = icmp eq i64 %cursor, 0\n  %minimum = icmp uge i64 %cursor, 2\n  %maximum = icmp ule i64 %cursor, 2147483647\n  %range = and i1 %minimum, %maximum\n  %valid = or i1 %zero, %range\n  br i1 %valid, label %invoke, label %invalid\ninvoke:\n  %value = call ptr @topal.runtime.int.from.u64(i64 %cursor)\n  %state.value = insertvalue {{ ptr, i32 }} undef, ptr %value, 0\n  %state = insertvalue {{ ptr, i32 }} %state.value, i32 0, 1\n",unit.policy.native_export());
    bridge.push_str(&emit_identity_value_adaptation(unit));
    bridge.push_str(identity_scalar_return());
    bridge
}

fn emit_identity_value_adaptation(unit:&crate::systems_numeric_library::CheckedNumericLibrary)->String {
    let mut bridge=String::new();
    let _=writeln!(bridge,"  %result = call fastcc {{ ptr, ptr, i32, i32 }} @{}{}({{ ptr, i32 }} %state)",unit.policy.function_prefix(),unit.entry_symbol);
    bridge.push_str("  %order.first = extractvalue { ptr, ptr, i32, i32 } %result, 2\n  %order.second = extractvalue { ptr, ptr, i32, i32 } %result, 3\n  %order.first.valid = icmp ult i32 %order.first, 2\n  %order.second.valid = icmp ult i32 %order.second, 2\n  %order.distinct = icmp ne i32 %order.first, %order.second\n  %order.range = and i1 %order.first.valid, %order.second.valid\n  %order.valid = and i1 %order.range, %order.distinct\n  br i1 %order.valid, label %result.valid, label %invalid\nresult.valid:\n");
    for name in ["allocated","next"] {
        let index=unit.result_field_order.iter().position(|field|topal_source::canonically_equal(field,name)).unwrap();
        let _=writeln!(bridge,"  %{name}.object = extractvalue {{ ptr, ptr, i32, i32 }} %result, {index}\n  %{name}.sign.pointer = getelementptr %topal.IntStorage, ptr %{name}.object, i32 0, i32 0\n  %{name}.length.pointer = getelementptr %topal.IntStorage, ptr %{name}.object, i32 0, i32 1\n  %{name}.sign = load i64, ptr %{name}.sign.pointer, align 8\n  %{name}.length = load i64, ptr %{name}.length.pointer, align 8\n  %{name}.positive = icmp eq i64 %{name}.sign, 0\n  %{name}.single = icmp eq i64 %{name}.length, 1\n  %{name}.shape = and i1 %{name}.positive, %{name}.single\n  br i1 %{name}.shape, label %{name}.extract, label %invalid\n{name}.extract:\n  %{name}.word.pointer = getelementptr %topal.IntStorage, ptr %{name}.object, i32 0, i32 2, i64 0\n  %{name}.word = load i32, ptr %{name}.word.pointer, align 4\n  %{name}.value = zext i32 %{name}.word to i64\n  %{name}.minimum = icmp uge i64 %{name}.value, 2\n  %{name}.maximum = icmp ule i64 %{name}.value, 2147483647\n  %{name}.range = and i1 %{name}.minimum, %{name}.maximum\n  br i1 %{name}.range, label %{name}.ready, label %invalid\n{name}.ready:");
    }
    bridge
}

fn identity_scalar_return()-> &'static str {
    "  %first = insertvalue { i64, i64 } undef, i64 %allocated.value, 0\n  %both = insertvalue { i64, i64 } %first, i64 %next.value, 1\n  ret { i64, i64 } %both\ninvalid:\n  call void @topal_kernel_numeric_abort_v1()\n  unreachable\n}\n\n"
}
