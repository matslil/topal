//! Whole-owner checks reuse the existing validator body unchanged, with private
//! helper names. The existing USER validator is neither replaced nor widened.
use super::*;

pub(crate) fn llvm(plan: &Plan, bindings: &Bindings) -> Result<String, CompileError> {
    bindings.validate(plan)?;
    let original = include_str!("runtime/kernel_user_extent_target.ll");
    let start = original.find("define internal i64 @u.owner(").unwrap();
    let end = original[start..]
        .find("define internal i64 @u.static_owner(")
        .unwrap()
        + start;
    let owner = original[start..end]
        .replace("@u.owner", "@k.image.owner")
        .replace("@u.require", "@k.image.require");
    let body = include_str!("runtime/kernel_image_backing.ll")
        .replace("{{PAGES}}", &plan.pages().to_string())
        .replace("{{WORDS}}", &(plan.pages() * 512).to_string())
        .replace("{{BYTES}}", &(plan.pages() * 4096).to_string())
        .replace("{{BOOT_SLOT}}", &(bindings.banks[0].slot * 24).to_string())
        .replace(
            "{{PROCESS_SLOT}}",
            &(bindings.banks[1].slot * 24).to_string(),
        );
    if body.contains("{{") {
        return Err(CompileError::Tool(
            "unresolved image-backing binding".into(),
        ));
    }
    Ok(format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{body}\n{owner}",
        crate::X86_SYSTEMS_DATA_LAYOUT
    ))
}

pub(crate) fn inspect_instructions(text: &str) -> Result<(), CompileError> {
    let mut function = "";
    let mut reads = 0;
    let mut ordinary = String::new();
    for line in text.lines() {
        if line.ends_with(">:")
            && let Some((_, name)) = line.split_once(" <")
        {
            function = name.trim_end_matches(">:");
        }
        if let Some((address, body)) = line.split_once(':')
            && !address.trim().is_empty()
            && address.trim().chars().all(|c| c.is_ascii_hexdigit())
            && body.contains("%cr")
        {
            if function != "k.image.active_root"
                || body.split_whitespace().collect::<Vec<_>>() != ["movq", "%cr3,", "%rax"]
            {
                return Err(CompileError::Tool(
                    "unexpected image-backing privileged instruction".into(),
                ));
            }
            reads += 1;
        } else {
            ordinary.push_str(line);
            ordinary.push('\n');
        }
    }
    if reads != 1 {
        return Err(CompileError::Tool(
            "image-backing component requires exactly one active-root observation site".into(),
        ));
    }
    crate::systems_library::inspect_instructions(&ordinary)
}
