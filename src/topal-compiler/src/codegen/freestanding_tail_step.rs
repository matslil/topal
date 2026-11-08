// TOPAL-SYSTEMS-TAIL-STEP-001. Ordinary source lowering into private step outcomes.
const TAIL_PREFIX: &str = "topal.tail.component.";
pub(crate) struct TailStepModule {
    pub(crate) llvm: String,
    pub(crate) readonly_roots: Vec<crate::systems_tail_component::SystemsTailReadonlyRoot>,
    pub(crate) region_runtime_sha256: String,
}

pub(crate) fn emit_tail_step_module(
    program: &CompilerProgram,
    source_name: &str,
    entry: &str,
    unsigned: bool,
) -> Result<TailStepModule, crate::CompileError> {
    use sha2::{Digest, Sha256};
    let mut generator = Generator::new(program, source_name);
    generator.kernel_library_mode = true;
    generator.kernel_parameter_debug_storage = true;
    generator.kernel_function_prefix = TAIL_PREFIX;
    generator.kernel_tail_component = true;
    generator
        .debug
        .install_source_modules(&program.dependencies);
    for function in &program.functions {
        if function.symbol != entry {
            generator.emit_function(function);
        }
    }
    let function = program
        .functions
        .iter()
        .find(|f| f.symbol == entry)
        .expect("rebuilt tail plan");
    generator.emit_tail_step(function);
    if !generator.foreign_declarations.is_empty()
        || !generator.scalar_list_runtime_fragments.is_empty()
        || generator
            .list_int_runtime_fragments
            .iter()
            .any(|f| !matches!(f, ListIntRuntimeFragment::FundamentalContainers))
        || generator.needs_infinity_result_runtime
    {
        return Err(crate::CompileError::Tool(
            "E-SYSTEMS-TAIL-STEP: unqualified runtime fragment".into(),
        ));
    }
    let region = tail_region_component_runtime()?;
    let region_runtime_sha256 = format!("{:x}", Sha256::digest(region.as_bytes()));
    let mut module = format!(
        "; unqualified ordinary source tail step component\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n\n",
        llvm_string(generator.debug.filename()),
        crate::X86_SYSTEMS_DATA_LAYOUT
    );
    for runtime in [
        VALUE_RUNTIME,
        include_str!("../runtime/list_int_layout.ll"),
        FUNDAMENTAL_CONTAINERS_RUNTIME,
        TAIL_COMPONENT_PLATFORM,
        region.as_str(),
    ]
    .into_iter()
    .chain(unsigned.then_some(include_str!("../runtime/kernel_region_unsigned.ll")))
    {
        for line in runtime.lines() {
            if line.starts_with("define ") && line.ends_with(" {") {
                let _ = writeln!(
                    module,
                    "{} noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{",
                    line.strip_suffix(" {").unwrap()
                );
            } else {
                let _ = writeln!(module, "{line}");
            }
        }
    }
    for global in &generator.globals {
        let _ = writeln!(module, "{global}");
    }
    for function in &generator.functions {
        module.push_str(function);
    }
    let mut roots = generator.tail_readonly_integers.clone();
    roots.insert("topal.runtime.int.zero".into(), 16);
    roots.insert("topal.runtime.int.one".into(), 20);
    module.push_str(&tail_readonly_query(&roots));
    module.push_str(&generator.debug.finish());
    let rename = |name: &str| {
        if let Some(suffix) = name.strip_prefix("topal.kernel.region.") {
            format!("{TAIL_PREFIX}primitive.{suffix}")
        } else if let Some(suffix) = name.strip_prefix(".topal.") {
            format!("{TAIL_PREFIX}literal.{suffix}")
        } else {
            name.into()
        }
    };
    let readonly_roots = roots
        .into_iter()
        .map(
            |(name, bytes)| crate::systems_tail_component::SystemsTailReadonlyRoot {
                symbol: rename(&if let Some(suffix) = name.strip_prefix("topal.runtime.") {
                    format!("{TAIL_PREFIX}runtime.{suffix}")
                } else {
                    name
                }),
                bytes,
            },
        )
        .collect();
    let module = crate::systems_numeric_library::namespace_llvm(&module, TAIL_PREFIX);
    let llvm = crate::systems_numeric_library::namespace_llvm_symbols(&module, rename);
    Ok(TailStepModule {
        llvm,
        readonly_roots,
        region_runtime_sha256,
    })
}

fn tail_region_component_runtime() -> Result<String, crate::CompileError> {
    let original = include_str!("../runtime/kernel_regions.ll");
    let cap = "  %count.valid = icmp ule i64 %count, 65536";
    if original.matches(cap).count() != 1 {
        return Err(crate::CompileError::Tool(
            "E-SYSTEMS-TAIL-STEP: tail extent guard review is stale".into(),
        ));
    }
    // Explicitly distinct family: the owning bridge must bind the full mapped
    // extent and scoped view. No primitive adds allocations on this path.
    Ok(original
        .replacen(
            cap,
            "  %count.valid = icmp ule i64 %count, 18446744073709551615",
            1,
        )
        .replace(
            "@topal_kernel_region_apply_v1",
            "@topal_tail_step_component_v1",
        )
        .replace("@topal_kernel_region_abort_v1", "@topal_tail_abort_v1"))
}

fn tail_readonly_query(roots: &BTreeMap<String, u64>) -> String {
    let mut result = String::from(
        "define i64 @topal_tail_readonly_bytes_v1(ptr %root) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {\nentry:\n",
    );
    let mut prior = "0".to_string();
    for (index, (name, bytes)) in roots.iter().enumerate() {
        let _ = writeln!(
            result,
            "  %match{index} = icmp eq ptr %root, @{name}\n  %size{index} = select i1 %match{index}, i64 {bytes}, i64 {prior}"
        );
        prior = format!("%size{index}");
    }
    let _ = writeln!(result, "  ret i64 {prior}\n}}\n");
    result
}

const TAIL_COMPONENT_PLATFORM: &str = r#"
declare ptr @topal_tail_allocate_v1(i64)
declare void @topal_tail_abort_v1() noreturn
declare void @topal_tail_step_begin_v1(ptr)
declare ptr @topal_tail_input_number_v1(ptr, i64)
declare i64 @topal_tail_input_boolean_v1(ptr, i64)
declare ptr @topal_tail_input_view_v1(ptr)
declare ptr @topal_tail_helper_enter_v1(ptr, i64, ptr)
declare void @topal_tail_helper_leave_v1(ptr, i64)
declare void @topal_tail_validate_view_v1(ptr, ptr)
declare void @topal_tail_stage_number_v1(ptr, i64, ptr)
declare void @topal_tail_stage_boolean_v1(ptr, i64, i64)
declare void @topal_tail_result_number_v1(ptr, ptr)
declare void @topal_tail_result_boolean_v1(ptr, i64)
declare void @topal_tail_result_unit_v1(ptr)
define internal ptr @topal.platform.allocate(i64 %length) nounwind noinline {
entry:
  %pointer = call ptr @topal_tail_allocate_v1(i64 %length)
  ret ptr %pointer
}
define internal void @topal.platform.exit(i64 %status) noreturn nounwind noinline {
entry:
  call void @topal_tail_abort_v1()
  unreachable
}
define internal i64 @topal.platform.write(i64 %fd, ptr %buffer, i64 %length) nounwind noinline {
entry:
  call void @topal_tail_abort_v1()
  unreachable
}
"#;
