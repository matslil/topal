//! Experimental bootstrap physical owner. Public source-bearing publication
//! remains refused until the complete owning gateway and root are reconstructed.
use crate::CompileError;

#[path = "systems_frame_inspection.rs"]
pub(crate) mod inspection;

#[cfg(test)]
#[path = "systems_frame_memory_tests.rs"]
pub(crate) mod memory_tests;
#[cfg(test)]
#[path = "systems_frame_root_tests.rs"]
pub(crate) mod root_tests;
#[cfg(test)]
#[path = "systems_frame_target_tests.rs"]
mod tests;

pub(crate) fn llvm(component: &crate::SystemsTailOwnerComponent) -> Result<String, CompileError> {
    let storage = &component.runtime.source_step.storage;
    let cost = |text: &str| -> Result<u64, CompileError> {
        let value = text
            .parse::<u64>()
            .ok()
            .filter(|v| *v != 0)
            .ok_or_else(|| {
                CompileError::Tool(
                    "E-SYSTEMS-FRAME-POLICY-BINDING: invalid source identity cost".into(),
                )
            })?;
        // Maximum input countdown plus its terminal source step and virgin IDs.
        value.checked_mul(2 * 262144 + 4).and_then(|n| n.checked_add(2)).ok_or_else(||
            CompileError::Tool("E-SYSTEMS-FRAME-POLICY-BINDING: source identity budget overflows bootstrap geometry".into()))?;
        Ok(value)
    };
    let frames = cost(&storage.source_frames_per_step)?;
    let loans = cost(&storage.source_region_loans_per_step)?;
    let mut text = format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n",
        crate::X86_SYSTEMS_DATA_LAYOUT
    );
    text.push_str(include_str!("runtime/kernel_frame_target.ll"));
    text = text
        .replace("{{FRAMES_PER_STEP}}", &frames.to_string())
        .replace("{{LOANS_PER_STEP}}", &loans.to_string());
    Ok(text
        .lines()
        .map(|line| {
            if line.starts_with("define ") && line.ends_with(" {") {
                format!(
                    "{} noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{",
                    line.strip_suffix(" {").unwrap()
                )
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n")
}

pub(crate) fn gateway_llvm(
    component: &crate::SystemsTailOwnerComponent,
) -> Result<String, CompileError> {
    let mut text = llvm(component)?;
    let start = text.find("define internal i1 @t.bootstrap()").unwrap();
    let end = start + text[start..].find("\n}\n").unwrap() + 3;
    let old = text[start..end].to_owned();
    let new = old.replace(
        "  ret i1 %ok",
        "  %lease = call i1 @g.lease()\n  %owned = and i1 %ok,%lease\n  ret i1 %owned",
    );
    text.replace_range(start..end, &new);
    let old = "  call void asm sideeffect \"cli; hlt\", \"~{dirflag},~{fpsr},~{flags},~{memory}\"()\n  br label %halt\n}";
    assert_eq!(text.matches(old).count(), 1);
    text = text.replace(
        old,
        "  call void @topal_x86_frame_gateway_abort_v1()\n  unreachable\n}",
    );
    text.push_str(include_str!("runtime/kernel_frame_gateway_lease.ll"));
    text = text.replace(
        "{{CALLER_RETIRED}}",
        &crate::systems_provider_object::tail_boundary::frame_gateway::RETIRED.to_string(),
    );
    Ok(text)
}

pub(crate) fn root_llvm(
    component: &crate::SystemsTailOwnerComponent,
    token_bytes: u64,
) -> Result<String, CompileError> {
    if token_bytes == 0 || token_bytes % 24 != 0 || token_bytes > 1024 * 1024 * 24 {
        return Err(CompileError::Tool(
            "invalid checked-root affine token storage".into(),
        ));
    }
    let mut text = gateway_llvm(component)?;
    let name = "define i1 @topal_frame_kernel_token_domain_v1";
    assert_eq!(text.matches(name).count(), 1);
    text = text.replace(name, "define internal i1 @r.pool_token_domain");
    text.push_str(&format!(
        "\n@topal_x86_frame_root_tokens_v1 = external global [{token_bytes} x i8]\n"
    ));
    text.push_str(
        &include_str!("runtime/kernel_frame_root_domain.ll")
            .replace("{{ROOT_TOKEN_BYTES}}", &token_bytes.to_string()),
    );
    Ok(text)
}

pub(crate) fn memory_llvm(
    component: &crate::SystemsTailOwnerComponent,
    token_bytes: u64,
) -> Result<String, CompileError> {
    let mut text = root_llvm(component, token_bytes)?;
    let domain = "define i1 @topal_frame_kernel_token_domain_v1";
    assert_eq!(text.matches(domain).count(), 1);
    text = text.replace(domain, "define internal i1 @mm.previous_token_domain");
    let lease = "  ret i1 %owned\n}";
    let start = text.find("define internal i1 @g.lease()").unwrap();
    let end = start + text[start..].find("\n}\n").unwrap() + 3;
    let current = text[start..end].to_owned();
    assert_eq!(current.matches(lease).count(), 1);
    text.replace_range(start..end,&current.replace(lease,"  %memoryContext = call i1 @g.memory_context()\n  %combined = and i1 %owned,%memoryContext\n  ret i1 %combined\n}"));
    let fatal = "define void @topal_frame_kernel_fatal_v1(ptr %ctx)";
    let at = text.find(fatal).unwrap();
    let brace = at + text[at..].find("entry:\n").unwrap() + 7;
    text.insert_str(
        brace,
        "  store volatile i64 3,ptr @topal_x86_frame_mm_state_v1,align 8\n",
    );
    text.push_str(include_str!("runtime/kernel_frame_memory_lease.ll"));
    Ok(text)
}

pub(crate) fn memory_dispatch() -> String {
    let old = include_str!("runtime/kernel_frame_dispatch.ll").replace(
        "define i64 @topal_frame_kernel_dispatch_v1",
        "define internal i64 @topal_frame_kernel_boot_dispatch_v1",
    );
    format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{old}\n{}",
        crate::X86_SYSTEMS_DATA_LAYOUT,
        include_str!("runtime/kernel_frame_memory.ll")
    )
}
