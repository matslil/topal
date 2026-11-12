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
