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
    let mut runtime = include_str!("runtime/kernel_image_tables.ll").to_owned()
        + include_str!("runtime/kernel_boot_coverage.ll");
    if plan.is_retained() {
        let from = runtime
            .find("define i64 @topal_x86_image_tables_discard_v1()")
            .unwrap();
        let until = from
            + runtime[from..]
                .find("  call void @k.tables.erase(i64 %b,i64 %p,i64 %low)")
                .unwrap();
        let retain = runtime[from..until].replace(
            "@topal_x86_image_tables_discard_v1()",
            "@topal_x86_image_tables_retain_v1()",
        );
        runtime.push_str(&retain);
        runtime.push_str("  %retained = call {i64,i64,i64} @r.verify(i1 true)\n  store volatile i64 1,ptr @topal_x86_retained_tables_v1,align 8\n  call void @k.image.store(i64 0,i64 3)\n  call void @topal_x86_image_tables_observe_v1()\n  ret i64 1\n}\n");
        runtime.push_str(include_str!("runtime/kernel_retained_tables.ll"));
        runtime=runtime.replace("%expected = select i1 %save,i64 0,i64 1", "%one = icmp eq i64 %phase,1\n  %two = icmp eq i64 %phase,2\n  %repeat = or i1 %one,%two\n  %expected_repeat = select i1 %repeat,i64 %phase,i64 1\n  %expected = select i1 %save,i64 0,i64 %expected_repeat")
            .replace("%completed = select i1 %save,i64 1,i64 2", "%advanced = add i64 %phase,1\n  %completed = select i1 %save,i64 1,i64 %advanced");
    }
    if plan.has_startup_storage() {
        if !plan.is_retained() {
            return Err(CompileError::Tool(
                "startup storage requires its retained boot profile".into(),
            ));
        }
        runtime=runtime.replace("  %retained = call {i64,i64,i64} @r.verify(i1 true)",
            "  %retained = call {i64,i64,i64} @r.verify(i1 true)\n  %startup_b = extractvalue {i64,i64,i64} %retained,0\n  %startup_p = extractvalue {i64,i64,i64} %retained,1\n  call void @s.startup.claim(i64 %startup_b,i64 %startup_p)");
        let start = runtime
            .find("define i64 @topal_x86_retained_tables_discard_v1()")
            .unwrap();
        let needle = "  call void @k.tables.verify(i64 %b,i64 %p,i64 %low,i1 false)\n";
        let entry = start + runtime[start..].find(needle).unwrap() + needle.len();
        runtime.insert_str(entry, "  call void @s.startup.verify(i64 %b,i64 %p)\n");
        runtime.push_str(&crate::systems_boot_image::startup::llvm());
    }
    let text = (common + &runtime)
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
