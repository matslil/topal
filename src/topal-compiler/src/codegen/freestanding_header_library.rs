/// Convert a caller-proved immutable byte extent to semantic values. The copy
/// reads backwards only to construct the ordinary immutable List in file order.
fn emit_owned_header_bridge(unit:&crate::systems_numeric_library::CheckedNumericLibrary) -> String {
    let mut bridge=format!("define i64 @{}(ptr %bytes, i64 %length, i64 %machine, i64 %maximum) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{\n",unit.policy.native_export());
    bridge.push_str(r#"entry:
  %address = ptrtoint ptr %bytes to i64
  %end = add i64 %address, 64
  %nonwrapping = icmp ugt i64 %end, %address
  %nonnull = icmp ne ptr %bytes, null
  %length.valid = icmp eq i64 %length, 64
  %machine.low = icmp uge i64 %machine, 1
  %machine.high = icmp ule i64 %machine, 65535
  %maximum.low = icmp uge i64 %maximum, 1
  %maximum.high = icmp ule i64 %maximum, 65535
  %extent.valid = and i1 %nonnull, %length.valid
  %extent.bounded = and i1 %extent.valid, %nonwrapping
  %machine.valid = and i1 %machine.low, %machine.high
  %maximum.valid = and i1 %maximum.low, %maximum.high
  %facts.valid = and i1 %machine.valid, %maximum.valid
  %valid = and i1 %extent.bounded, %facts.valid
  br i1 %valid, label %invoke, label %invalid
invoke:
  %machine.value = call ptr @topal.runtime.int.from.u64(i64 %machine)
  %maximum.value = call ptr @topal.runtime.int.from.u64(i64 %maximum)
  br label %copy
copy:
  %remaining = phi i64 [64, %invoke], [%index, %copy]
  %tail = phi ptr [null, %invoke], [%node, %copy]
  %index = sub i64 %remaining, 1
  %input = getelementptr i8, ptr %bytes, i64 %index
  %byte = load i8, ptr %input, align 1
  %unsigned = zext i8 %byte to i64
  %integer = call ptr @topal.runtime.int.from.u64(i64 %unsigned)
  %node = call ptr @topal.platform.allocate(i64 16)
  store ptr %integer, ptr %node, align 8
  %next = getelementptr i8, ptr %node, i64 8
  store ptr %tail, ptr %next, align 8
  %done = icmp eq i64 %index, 0
  br i1 %done, label %policy, label %copy
policy:
  %array = call ptr @topal.runtime.container.array.int.collect(ptr %node)
"#);
    let _=writeln!(bridge,"  %accepted = call fastcc i1 @{}{}(ptr %array, ptr %machine.value, ptr %maximum.value)",unit.policy.function_prefix(),unit.entry_symbol);
    bridge.push_str("  %result = zext i1 %accepted to i64\n  ret i64 %result\ninvalid:\n  call void @topal_kernel_numeric_abort_v1()\n  unreachable\n}\n\n");
    bridge
}
