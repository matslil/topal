use topal_language::compiler::CompilerKernelRegionProfile;

pub(crate) fn region_adapter_sha256() -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(REGION_BRIDGE_HEAD.as_bytes());
    hash.update(REGION_BRIDGE_TAIL.as_bytes());
    format!("{:x}", hash.finalize())
}

/// Ordinary function lowering with explicitly supplied region/runtime authority.
pub(crate) fn emit_region_kernel_library(
    unit: &crate::systems_region_library::CheckedRegionLibrary,
) -> Result<String, crate::CompileError> {
    let mut generator = Generator::new(&unit.program, &unit.source_name);
    generator.kernel_library_mode = true;
    generator.kernel_parameter_debug_storage = true;
    generator.kernel_function_prefix = unit.prefix();
    generator
        .debug
        .install_source_modules(&unit.program.dependencies);
    for function in &unit.program.functions {
        generator.emit_function(function);
    }
    if !generator.foreign_declarations.is_empty()
        || !generator.scalar_list_runtime_fragments.is_empty()
        || generator
            .list_int_runtime_fragments
            .iter()
            .any(|fragment| !matches!(fragment, ListIntRuntimeFragment::FundamentalContainers))
        || generator.needs_infinity_result_runtime
    {
        return Err(crate::CompileError::Tool(
            "E-SYSTEMS-REGION-LIBRARY: unqualified runtime fragment".into(),
        ));
    }
    let mut module = format!(
        "; checked ordinary Topal region source object\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n\n",
        llvm_string(generator.debug.filename()),
        crate::X86_SYSTEMS_DATA_LAYOUT
    );
    module.push_str(
        "@llvm.used = appending global [1 x ptr] [ptr @memset], section \"llvm.metadata\"\n",
    );
    for runtime in [
        VALUE_RUNTIME,
        include_str!("../runtime/list_int_layout.ll"),
        FUNDAMENTAL_CONTAINERS_RUNTIME,
        include_str!("../runtime/kernel_region_platform.ll"),
        include_str!("../runtime/kernel_regions.ll"),
    ] {
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
        module.push('\n');
    }
    module.push_str(&region_bridge_head(unit.profile));
    match unit.profile {
        CompilerKernelRegionProfile::Unit => {
            let _ = writeln!(
                module,
                "  call fastcc void @{}{}(ptr %view)",
                unit.prefix(),
                unit.entry_symbol
            );
        }
        CompilerKernelRegionProfile::Action => {
            module.push_str(REGION_ACTION_INPUTS);
            let _ = writeln!(
                module,
                "  %accepted = call fastcc i1 @{}{}(ptr %view, ptr %first.value, ptr %second.value)",
                unit.prefix(),
                unit.entry_symbol
            );
        }
    }
    module.push_str(&region_bridge_tail(unit.profile));
    module.push_str(&generator.debug.finish());
    let module = crate::systems_numeric_library::namespace_llvm(&module, unit.prefix());
    Ok(match unit.profile {
        CompilerKernelRegionProfile::Unit => module,
        CompilerKernelRegionProfile::Action => {
            // Named local constants permit exact object-to-linked data binding.
            // They remain immutable and module-owned; no extra global export.
            let module = module
                .lines()
                .map(|line| {
                    if line.starts_with('@') {
                        line.replacen(" = private ", " = internal ", 1)
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            crate::systems_numeric_library::namespace_llvm_symbols(&module, |name| {
                use crate::systems_region_library as region;
                match name {
                    region::EXPORT => region::ACTION_EXPORT.into(),
                    region::ALLOCATE => region::ACTION_ALLOCATE.into(),
                    region::ABORT => region::ACTION_ABORT.into(),
                    _ => match name.strip_prefix("topal.kernel.region.") {
                        Some(suffix) => format!("{}primitive.{suffix}", region::ACTION_PREFIX),
                        None => match name.strip_prefix(".topal.") {
                            Some(suffix) => format!("{}literal.{suffix}", region::ACTION_PREFIX),
                            None => name.into(),
                        },
                    },
                }
            })
        }
    })
}

// The caller's region/control/temporary extents are live, mapped and disjoint.
// Expected owner comes from that caller's owning resource, never from source.
const REGION_BRIDGE_HEAD: &str = r#"
define void @topal_kernel_region_apply_v1(ptr %control, i64 %expected.owner, i64 %next.epoch) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  %address = ptrtoint ptr %control to i64
  %nonnull = icmp ne i64 %address, 0
  %alignment = and i64 %address, 7
  %aligned = icmp eq i64 %alignment, 0
  %control.end = add i64 %address, 64
  %control.extent.valid = icmp uge i64 %control.end, %address
  %pointer.valid = and i1 %nonnull, %aligned
  %control.valid = and i1 %pointer.valid, %control.extent.valid
  br i1 %control.valid, label %fields, label %invalid
fields:
  %code = load ptr, ptr %control, align 8
  %owner.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 1
  %epoch.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 2
  %active.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 3
  %base.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 4
  %count.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 5
  %rights.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 6
  %issued.pointer = getelementptr %topal.KernelRegionControl, ptr %control, i32 0, i32 7
  %owner = load i64, ptr %owner.pointer, align 8
  %epoch = load i64, ptr %epoch.pointer, align 8
  %active = load i64, ptr %active.pointer, align 8
  %base = load ptr, ptr %base.pointer, align 8
  %count = load i64, ptr %count.pointer, align 8
  %rights = load i64, ptr %rights.pointer, align 8
  %issued = load i64, ptr %issued.pointer, align 8
  %code.valid = icmp eq ptr %code, @topal_kernel_region_apply_v1
  %owner.nonzero = icmp ne i64 %owner, 0
  %owner.matches = icmp eq i64 %owner, %expected.owner
  %owner.valid = and i1 %owner.nonzero, %owner.matches
  %epoch.bounded = icmp ne i64 %epoch, 18446744073709551615
  %successor = add i64 %epoch, 1
  %epoch.matches = icmp eq i64 %successor, %next.epoch
  %epoch.valid = and i1 %epoch.bounded, %epoch.matches
  %active.valid = icmp eq i64 %active, 0
  %issued.valid = icmp eq i64 %issued, 0
  %count.valid = icmp ule i64 %count, 65536
  %rights.valid = icmp ule i64 %rights, 1
  %base.address = ptrtoint ptr %base to i64
  %base.valid = icmp ne i64 %base.address, 0
  %end = add i64 %base.address, %count
  %end.valid = icmp uge i64 %end, %base.address
  %below.control = icmp ule i64 %end, %address
  %above.control = icmp uge i64 %base.address, %control.end
  %disjoint = or i1 %below.control, %above.control
  %a = and i1 %code.valid, %owner.valid
  %b = and i1 %epoch.valid, %active.valid
  %c = and i1 %issued.valid, %count.valid
  %d = and i1 %rights.valid, %base.valid
  %e = and i1 %end.valid, %disjoint
  %ab = and i1 %a, %b
  %cd = and i1 %c, %d
  %abcd = and i1 %ab, %cd
  %valid = and i1 %abcd, %e
  br i1 %valid, label %prepare, label %invalid
prepare:
  store i64 %next.epoch, ptr %epoch.pointer, align 8
  store i64 1, ptr %active.pointer, align 8
  store i64 1, ptr %issued.pointer, align 8
  %view = call ptr @topal.platform.allocate(i64 64)
  %v0 = insertvalue %topal.KernelRegionView zeroinitializer, ptr %control, 0
  %v1 = insertvalue %topal.KernelRegionView %v0, i64 %count, 2
  %v2 = insertvalue %topal.KernelRegionView %v1, i64 %rights, 3
  %v3 = insertvalue %topal.KernelRegionView %v2, i64 %next.epoch, 4
  %v4 = insertvalue %topal.KernelRegionView %v3, i64 %owner, 5
  %v5 = insertvalue %topal.KernelRegionView %v4, ptr @topal_kernel_region_apply_v1, 6
  store %topal.KernelRegionView %v5, ptr %view, align 8
"#;

const REGION_BRIDGE_TAIL: &str = r#"
  %final.view = call ptr @topal.kernel.region.guard(ptr %view)
  store ptr null, ptr %control, align 8
  store i64 0, ptr %owner.pointer, align 8
  store i64 0, ptr %active.pointer, align 8
  store ptr null, ptr %base.pointer, align 8
  store i64 0, ptr %count.pointer, align 8
  store i64 0, ptr %rights.pointer, align 8
  store i64 0, ptr %issued.pointer, align 8
  ret void
invalid:
  call void @topal.kernel.region.abort()
  unreachable
}
"#;

// Only explicit, compiler-owned adapter syntax is adapted here. All source and
// debug text is preserved; global renaming uses the LLVM identifier scanner.
fn region_bridge_head(profile: CompilerKernelRegionProfile) -> String {
    match profile {
        CompilerKernelRegionProfile::Unit => REGION_BRIDGE_HEAD.into(),
        CompilerKernelRegionProfile::Action => REGION_BRIDGE_HEAD.replacen(
            "define void @topal_kernel_region_apply_v1(ptr %control, i64 %expected.owner, i64 %next.epoch)",
            "define i64 @topal_kernel_region_apply_v1(ptr %control, i64 %expected.owner, i64 %next.epoch, i64 %first, i64 %second)", 1),
    }
}
fn region_bridge_tail(profile: CompilerKernelRegionProfile) -> String {
    match profile {
        CompilerKernelRegionProfile::Unit => REGION_BRIDGE_TAIL.into(),
        CompilerKernelRegionProfile::Action => REGION_BRIDGE_TAIL.replacen(
            "  ret void",
            "  %result = zext i1 %accepted to i64\n  ret i64 %result",
            1,
        ),
    }
}
const REGION_ACTION_INPUTS: &str = r#"
  %first.value = call ptr @topal.runtime.int.from.u64(i64 %first)
  %second.value = call ptr @topal.runtime.int.from.u64(i64 %second)
"#;

pub(crate) fn region_action_adapter_sha256() -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(region_bridge_head(CompilerKernelRegionProfile::Action).as_bytes());
    hash.update(REGION_ACTION_INPUTS.as_bytes());
    hash.update(region_bridge_tail(CompilerKernelRegionProfile::Action).as_bytes());
    format!("{:x}", hash.finalize())
}
