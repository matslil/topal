//! Bind actual inactive table bytes to the same complete live owners.
use super::*;

pub(super) fn llvm(plan: &Plan, bindings: &Bindings) -> Result<String, CompileError> {
    let low = bindings
        .low_user_leaf
        .ok_or_else(|| CompileError::Tool("missing separate low leaf".into()))?;
    let original = include_str!("runtime/kernel_user_extent_target.ll");
    let start = original.find("define internal i64 @u.owner(").unwrap();
    let end = start
        + original[start..]
            .find("define internal i64 @u.static_owner(")
            .unwrap();
    let owner = original[start..end]
        .replace("@u.owner", "@k.image.owner")
        .replace("@u.require", "@k.image.require");
    let common = include_str!("runtime/kernel_image_backing.ll");
    let end = common
        .find("define void @topal_x86_image_backing_observe_v1")
        .unwrap();
    let common = common[..end]
        .replace(
            "@k.image.query(ptr %token)",
            "@k.image.query(ptr %token,i64 %expected)",
        )
        .replace("%count,{{PAGES}}", "%count,%expected")
        .replace(
            "@k.image.query(ptr %bt)",
            "@k.image.query(ptr %bt,i64 {{PAGES}})",
        )
        .replace(
            "@k.image.query(ptr %pt)",
            "@k.image.query(ptr %pt,i64 {{PAGES}})",
        )
        .replace(
            "@k.image.zero(i64 %base)",
            "@k.image.zero(i64 %base,i64 %words)",
        )
        .replace(
            "@k.image.seed(i64 %base)",
            "@k.image.seed(i64 %base,i64 %words)",
        )
        .replace(
            "@k.image.verify_zero(i64 %base)",
            "@k.image.verify_zero(i64 %base,i64 %words)",
        )
        .replace("{{WORDS}}", "%words");
    let runtime = include_str!("runtime/kernel_image_tables.ll");
    let text = (common + runtime)
        .replace(
            "@topal_x86_image_backing_observation_v1",
            "@topal_x86_image_tables_observation_v1",
        )
        .replace("{{PAGES}}", &plan.pages().to_string())
        .replace("{{WORDS}}", &(plan.pages() * 512).to_string())
        .replace("{{BYTES}}", &(plan.pages() * 4096).to_string())
        .replace("{{BOOT_SLOT}}", &(bindings.banks[0].slot * 24).to_string())
        .replace(
            "{{PROCESS_SLOT}}",
            &(bindings.banks[1].slot * 24).to_string(),
        )
        .replace("{{LOW_SLOT}}", &(low.slot * 24).to_string());
    if text.contains("{{") {
        return Err(CompileError::Tool(
            "unbound inactive table component".into(),
        ));
    }
    let template = plan.table_plan()?.llvm();
    let template = template
        .lines()
        .skip(2)
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "define i1 @topal_x86_image_table_",
            "define internal i1 @topal_x86_image_table_",
        );
    let boot = crate::systems_boot_image::image_contract::llvm()?;
    Ok(format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{text}\n{owner}\n{template}\n{boot}\n",
        crate::X86_SYSTEMS_DATA_LAYOUT
    ))
}
