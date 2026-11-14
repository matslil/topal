//! Native ownership controller around the ordinary frame-selection policy.
//! Target admission/physical access and complete kernel caller binding are
//! required separately; emitting this module grants no publication authority.
use crate::CompileError;

#[cfg(test)]
#[path = "systems_frame_controller_tests.rs"]
mod tests;

pub(crate) fn llvm() -> Result<String, CompileError> {
    let mut text = format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n",
        crate::X86_SYSTEMS_DATA_LAYOUT
    );
    text.push_str(include_str!("runtime/kernel_frame_controller.ll"));
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
