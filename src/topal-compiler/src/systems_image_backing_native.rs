//! Whole-owner checks reuse the existing validator body unchanged, with private
//! helper names. The existing USER validator is neither replaced nor widened.
use super::*;

pub(crate) fn llvm(plan: &Plan, bindings: &Bindings) -> Result<String, CompileError> {
    bindings.validate(plan)?;
    if plan.is_tables() {
        return super::tables_native::llvm(plan, bindings);
    }
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

pub(crate) fn inspect_instructions_selected(text: &str, boot: bool) -> Result<(), CompileError> {
    let mut function = "";
    let mut reads = std::collections::BTreeMap::<&str, u64>::new();
    let mut ordinary = String::new();
    for line in text.lines() {
        if let Some((address, body)) = line.split_once(':')
            && !address.trim().is_empty()
            && address.trim().chars().all(|c| c.is_ascii_hexdigit())
        {
            let fields = body.split_whitespace().collect::<Vec<_>>();
            if fields.first().is_some_and(|m| {
                [
                    "lgdt", "lgdtq", "lidt", "lidtq", "ltr", "lldt", "invlpg", "invlpga", "invd",
                    "wbinvd", "clac", "stac", "clts", "lmsw", "xsetbv",
                ]
                .contains(m)
            }) || fields
                .last()
                .is_some_and(|m| ["%cs", "%ds", "%es", "%ss", "%fs", "%gs"].contains(m))
            {
                return Err(CompileError::Tool(
                    "image preparation must not write architecture control state".into(),
                ));
            }
        }
        if line.ends_with(">:")
            && let Some((_, name)) = line.split_once(" <")
        {
            function = name.trim_end_matches(">:");
        }
        if let Some((address, body)) = line.split_once(':')
            && !address.trim().is_empty()
            && address.trim().chars().all(|c| c.is_ascii_hexdigit())
            && (body.contains("%cr")
                || body.split_whitespace().any(|word| word == "%cs,")
                || ["cpuid", "rdmsr", "sgdtq", "sidtq", "sgdt", "sidt"]
                    .contains(&body.split_whitespace().next().unwrap_or("")))
        {
            let fields = body.split_whitespace().collect::<Vec<_>>();
            let key = match (function, fields.as_slice()) {
                ("k.image.active_root", ["movq", "%cr3,", "%rax"]) => "cr3",
                ("k.boot.cr0", ["movq", "%cr0,", "%rax"]) if boot => "cr0",
                ("k.boot.cr4", ["movq", "%cr4,", "%rax"]) if boot => "cr4",
                ("k.boot.cs", ["movw", "%cs,", "%ax"]) if boot => "cs",
                ("k.boot.cpuid", ["cpuid"]) if boot => "cpuid",
                ("k.boot.msr", ["rdmsr"]) if boot => "rdmsr",
                ("k.boot.descriptors", [op, operand])
                    if boot
                        && ["sgdtq", "sgdt", "sidtq", "sidt"].contains(op)
                        && operand.starts_with("(%")
                        && operand.ends_with(')') =>
                {
                    if op.starts_with('s') && op.contains("gdt") {
                        "sgdt"
                    } else {
                        "sidt"
                    }
                }
                _ => {
                    return Err(CompileError::Tool(
                        "unexpected image-preparation architectural instruction".into(),
                    ));
                }
            };
            *reads.entry(key).or_default() += 1;
        } else {
            ordinary.push_str(line);
            ordinary.push('\n');
        }
    }
    let expected = if boot {
        vec!["cr3", "cr0", "cr4", "cs", "cpuid", "rdmsr", "sgdt", "sidt"]
    } else {
        vec!["cr3"]
    };
    if reads.len() != expected.len() || expected.iter().any(|key| reads.get(key) != Some(&1)) {
        return Err(CompileError::Tool(
            "image-backing component requires exactly one active-root observation site".into(),
        ));
    }
    crate::systems_library::inspect_instructions(&ordinary)
}
