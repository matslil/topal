//! TK-DEC-284: unpublished native heap preparation/cancellation component.
//! Real allocator/entry binding; no process switching or public activation.
use crate::{CompileError, SystemsTailOwnerComponent};

pub(crate) fn target_llvm(
    component: &SystemsTailOwnerComponent,
    owners: &crate::systems_user_extent::Owners,
) -> Result<String, CompileError> {
    let mut text = crate::systems_user_extent::caller_target_llvm(component, owners)?;
    let domain = "  %mm = and i1 %owned,%either\n";
    if text.matches(domain).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing exact token domain".into(),
        ));
    }
    text = text.replace(domain, "  %prepared = icmp eq ptr %token,@topal_x86_process_mm_prepared_token_v1\n  %mmEither = or i1 %either,%prepared\n  %mm = and i1 %owned,%mmEither\n");
    text.push_str("\n@topal_x86_process_mm_prepared_token_v1 = external global [3 x i64]\n");
    Ok(text)
}

pub(crate) fn dispatch_llvm() -> Result<String, CompileError> {
    let mut text = crate::systems_user_extent::dispatch_llvm()?;
    let site = "  %brk = icmp eq i64 %operation,5\n";
    if text.matches(site).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing exact admitted memory site".into(),
        ));
    }
    text = text.replace(site, &format!("  %pmBrk = icmp eq i64 %operation,5\n  br i1 %pmBrk,label %pmPreparation,label %pmContinue\npmPreparation:\n  %preparedHeap = call i1 @pm.prepare()\n  call void @pm.cancel(i1 %preparedHeap)\n  br label %pmContinue\npmContinue:\n{site}"));
    text.push_str(include_str!("runtime/kernel_process_mm.ll"));
    Ok(text)
}

pub(crate) fn process_target_llvm(
    component: &SystemsTailOwnerComponent,
    owners: &crate::systems_user_extent::Owners,
) -> Result<String, CompileError> {
    let mut text = target_llvm(component, owners)?;
    let domain = "  %mmEither = or i1 %either,%prepared\n";
    let known = "  %known = icmp ule i64 %op,7\n";
    let abi = "  %abi = select i1 %cleanup,i1 %exitOk,i1 %requestOk\n";
    let site = "  %returnSite = select i1 %cleanup,i64 %cleanupReturn,i64 %syscallReturn\n";
    for expected in [domain, known, abi, site] {
        if text.matches(expected).count() != 1 {
            return Err(CompileError::Tool(
                "E-SYSTEMS-PROCESS-MM: incomplete exact context adapter".into(),
            ));
        }
    }
    text = text.replace(domain, "  %inactive = icmp eq ptr %token,@topal_x86_process_mm_inactive_token_v1\n  %extra = or i1 %prepared,%inactive\n  %mmEither = or i1 %either,%extra\n");
    text = text.replace(known, "  %oldKnown = icmp ule i64 %op,7\n  %processLow = icmp uge i64 %op,9\n  %processHigh = icmp ule i64 %op,13\n  %process = and i1 %processLow,%processHigh\n  %known = or i1 %oldKnown,%process\n");
    text = text.replace(abi, "  %oldAbi = select i1 %cleanup,i1 %exitOk,i1 %requestOk\n  %processAbi = call i1 @p.abi(i64 %op,i64 %number)\n  %abi = select i1 %process,i1 %processAbi,i1 %oldAbi\n");
    text = text.replace(site, "  %oldReturn = select i1 %cleanup,i64 %cleanupReturn,i64 %syscallReturn\n  %processReturn = call i64 @p.return_site(i64 %op)\n  %returnSite = select i1 %process,i64 %processReturn,i64 %oldReturn\n");
    text.push_str(include_str!("runtime/kernel_process_mm_context.ll"));
    Ok(text)
}

pub(crate) fn process_dispatch_llvm() -> Result<String, CompileError> {
    let mut text = crate::systems_user_extent::dispatch_llvm()?;
    let entry = "define i64 @topal_frame_kernel_dispatch_v1(i64 %op,i64 %a,i64 %b,ptr %token,ptr %facts) nounwind noinline noredzone {\nentry:\n";
    if text.matches(entry).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing exact dispatch entry".into(),
        ));
    }
    text = text.replace(entry, &format!("{entry}  %pLow = icmp uge i64 %op,9\n  %pHigh = icmp ule i64 %op,13\n  %pKnown = and i1 %pLow,%pHigh\n  br i1 %pKnown,label %process,label %oldEntry\nprocess:\n  %pNoB = icmp eq i64 %b,0\n  %pNoToken = icmp eq ptr %token,null\n  %pNoFacts = icmp eq ptr %facts,null\n  %pA = and i1 %pNoB,%pNoToken\n  %pOk = and i1 %pA,%pNoFacts\n  call void @m.require(i1 %pOk)\n  %pResult = call i64 @p.request(i64 %op,i64 %a)\n  ret i64 %pResult\noldEntry:\n"));
    let validated = "  %brk = icmp eq i64 %operation,5\n";
    if text.matches(validated).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing validated active heap".into(),
        ));
    }
    text = text.replace(
        validated,
        &format!("  call void @p.memory_entry(i64 %operation)\n{validated}"),
    );
    let initialized = "  call void @m.put(ptr @topal_x86_frame_mm_state_v1,i64 1,i64 6254592)\n";
    if text.matches(initialized).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing initial memory owner".into(),
        ));
    }
    text = text.replace(
        initialized,
        &format!("{initialized}  call void @p.initial_parent()\n"),
    );
    let result =
        "  %result = call i64 @topal_frame_memory_request_v1(i64 %op,i64 %a)\n  ret i64 %result\n";
    if text.matches(result).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing memory completion".into(),
        ));
    }
    text = text.replace(result, "  %result = call i64 @topal_frame_memory_request_v1(i64 %op,i64 %a)\n  call void @p.memory_return(i64 %op)\n  ret i64 %result\n");
    let primitive = include_str!("runtime/kernel_process_mm.ll");
    let old = "  %brk = icmp eq i64 %op,5\n";
    if primitive.matches(old).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-PROCESS-MM: missing preparation operation guard".into(),
        ));
    }
    text.push_str(&primitive.replace(old, "  %brk = icmp eq i64 %op,9\n"));
    text.push_str(
        &include_str!("runtime/kernel_process_mm_owners.ll").replace(
            "{{TABLE_BYTES}}",
            &crate::systems_provider_object::PROCESS_MM_TABLE_BYTES.to_string(),
        ),
    );
    Ok(text)
}
