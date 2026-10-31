/// Ordinary checked functions and the shared numeric algorithms, with the
/// platform dependency explicitly supplied by the owning kernel invocation.
pub(crate) fn emit_numeric_kernel_library(unit:&crate::systems_numeric_library::CheckedNumericLibrary)
    ->Result<String,crate::CompileError> {
    let mut generator=Generator::new(&unit.program,&unit.source_name);
    generator.kernel_library_mode=true;
    generator.kernel_function_prefix=crate::systems_numeric_library::FUNCTION_PREFIX;
    for function in &unit.program.functions { generator.emit_function(function); }
    if !generator.foreign_declarations.is_empty() || !generator.scalar_list_runtime_fragments.is_empty()
        || !generator.list_int_runtime_fragments.is_empty() || generator.needs_infinity_result_runtime {
        return Err(crate::CompileError::Tool("E-SYSTEMS-NUMERIC-LIBRARY: unqualified runtime fragment".into()));
    }
    let mut module=format!("; checked Topal numeric kernel library\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n\n",
        llvm_string(generator.debug.filename()),crate::X86_SYSTEMS_DATA_LAYOUT);
    module.push_str("@llvm.used = appending global [1 x ptr] [ptr @memset], section \"llvm.metadata\"\n");
    for runtime in [VALUE_RUNTIME,include_str!("../runtime/kernel_numeric_platform.ll")] {
        for line in runtime.lines() {
            if line.starts_with("define ") && line.ends_with(" {") {
                let _=writeln!(module,"{} noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{",line.strip_suffix(" {").unwrap());
            } else { let _=writeln!(module,"{line}"); }
        }
    }
    for global in &generator.globals { let _=writeln!(module,"{global}"); }
    for function in &generator.functions { module.push_str(function);module.push('\n'); }
    let _=writeln!(module,"define i64 @{}(i64 %seconds, i64 %nanoseconds, i64 %ticks, i64 %wrapped) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{",crate::systems_numeric_library::EXPORT_SYMBOL);
    module.push_str("entry:\n  %seconds.valid = icmp ule i64 %seconds, 9223372036854775807\n  %nanos.valid = icmp ult i64 %nanoseconds, 1000000000\n  %wrapped.valid = icmp ule i64 %wrapped, 1\n  %valid.interval = and i1 %seconds.valid, %nanos.valid\n  %valid = and i1 %valid.interval, %wrapped.valid\n  br i1 %valid, label %invoke, label %invalid\ninvoke:\n  %s = call ptr @topal.runtime.int.from.u64(i64 %seconds)\n  %n = call ptr @topal.runtime.int.from.u64(i64 %nanoseconds)\n  %t = call ptr @topal.runtime.int.from.u64(i64 %ticks)\n  %w = trunc i64 %wrapped to i1\n");
    let _=writeln!(module,"  %result = call fastcc ptr @{}(ptr %s, ptr %n, ptr %t, i1 %w)",format!("{}{}",crate::systems_numeric_library::FUNCTION_PREFIX,unit.entry_symbol));
    module.push_str("  %sign.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 0\n  %length.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 1\n  %sign = load i64, ptr %sign.pointer, align 8\n  %length = load i64, ptr %length.pointer, align 8\n  %nonnegative = icmp eq i64 %sign, 0\n  %bounded.length = icmp ule i64 %length, 2\n  %valid.result = and i1 %nonnegative, %bounded.length\n  br i1 %valid.result, label %extract, label %invalid\nextract:\n  %zero = icmp eq i64 %length, 0\n  br i1 %zero, label %return.zero, label %low\nreturn.zero:\n  ret i64 0\nlow:\n  %low.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 0\n  %low.word = load i32, ptr %low.pointer, align 4\n  %low.value = zext i32 %low.word to i64\n  %single = icmp eq i64 %length, 1\n  br i1 %single, label %check, label %high\nhigh:\n  %high.pointer = getelementptr %topal.IntStorage, ptr %result, i32 0, i32 2, i64 1\n  %high.word = load i32, ptr %high.pointer, align 4\n  %high.value = zext i32 %high.word to i64\n  %shifted = shl i64 %high.value, 32\n  %combined = or i64 %low.value, %shifted\n  br label %check\ncheck:\n  %value = phi i64 [%low.value, %low], [%combined, %high]\n  %in.range = icmp ule i64 %value, 922337203685477581\n  br i1 %in.range, label %return, label %invalid\nreturn:\n  ret i64 %value\ninvalid:\n  call void @topal_kernel_numeric_abort_v1()\n  unreachable\n}\n\n");
    module.push_str(&generator.debug.finish());
    Ok(module)
}
