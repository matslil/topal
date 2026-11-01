/// The nominal tag is adapted by semantic name, never by declaration order.
fn emit_owned_dispatch_bridge(unit:&crate::systems_numeric_library::CheckedNumericLibrary)->String {
    let mut bridge=format!("define i64 @{}(i64 %selector) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{\nentry:\n  %value = call ptr @topal.runtime.int.from.u64(i64 %selector)\n",unit.policy.native_export());
    let _=writeln!(bridge,"  %selection = call fastcc i32 @{}{}(ptr %value)",unit.policy.function_prefix(),unit.entry_symbol);
    bridge.push_str("  switch i32 %selection, label %invalid [\n");
    for (index,tag) in unit.source_tags.iter().enumerate() {
        let _=writeln!(bridge,"    i32 {tag}, label %operation.{index}");
    }
    bridge.push_str("  ]\n");
    for (index,(_,code)) in crate::systems_numeric_library::DISPATCH_CHOICES.iter().enumerate() {
        let _=writeln!(bridge,"operation.{index}:\n  ret i64 {code}");
    }
    bridge.push_str("invalid:\n  call void @topal_kernel_numeric_abort_v1()\n  unreachable\n}\n\n");
    bridge
}
