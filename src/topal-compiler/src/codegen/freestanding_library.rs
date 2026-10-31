/// Reuse ordinary function lowering, excluding the hosted root and runtime.
/// The private checked wrapper admits no values which need those facilities.
pub(crate) fn emit_finite_kernel_library(unit: &crate::systems_library::CheckedFiniteLibrary)
    -> Result<String,crate::CompileError> {
    let mut generator=Generator::new(unit.program(),unit.source_name());
    generator.kernel_library_mode=true;
    for function in &unit.program().functions { generator.emit_function(function); }
    if !generator.globals.is_empty() || !generator.foreign_declarations.is_empty()
        || !generator.scalar_list_runtime_fragments.is_empty()
        || !generator.list_int_runtime_fragments.is_empty() || generator.needs_infinity_result_runtime {
        return Err(crate::CompileError::Tool("E-SYSTEMS-LIBRARY: finite source lowering requested runtime/global support".into()));
    }
    let mut module=format!("; qualified Topal kernel source library\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n\n",
        llvm_string(generator.debug.filename()),crate::X86_SYSTEMS_DATA_LAYOUT);
    for function in &generator.functions { module.push_str(function); module.push('\n'); }
    let _=writeln!(module,"define i64 @{}(i64 %facts) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{",crate::systems_library::EXPORT_SYMBOL);
    module.push_str("entry:\n  %valid = icmp ult i64 %facts, 8\n  br i1 %valid, label %invoke, label %invalid\ninvoke:\n  %directory.bits = lshr i64 %facts, 1\n  %required.bits = lshr i64 %facts, 2\n  %write = trunc i64 %facts to i1\n  %directory = trunc i64 %directory.bits to i1\n  %required = trunc i64 %required.bits to i1\n");
    let _=writeln!(module,"  %decision = call fastcc i32 @{}(i1 %write, i1 %directory, i1 %required)",unit.entry_symbol());
    module.push_str("  switch i32 %decision, label %invalid [\n");
    for (canonical,source) in unit.source_tags().iter().enumerate() { let _=writeln!(module,"    i32 {source}, label %choice.{canonical}"); }
    module.push_str("  ]\n");
    for canonical in 0..4 { let _=writeln!(module,"choice.{canonical}:\n  ret i64 {canonical}"); }
    module.push_str("invalid:\n  ret i64 -1\n}\n\n");
    module.push_str(&generator.debug.finish());
    Ok(module)
}
