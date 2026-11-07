// Target-private transport only; source sees an ordinary region and Nat values.
fn region_frame_head(scalar_count: u8) -> String {
    let mut head = REGION_BRIDGE_HEAD.replacen(
        "define void @topal_kernel_region_apply_v1(ptr %control, i64 %expected.owner, i64 %next.epoch)",
        "define i64 @topal_kernel_region_apply_v1(ptr %control, i64 %expected.owner, i64 %next.epoch, ptr %frame.words, i64 %frame.word.count)", 1);
    head = head.replacen("entry:\n", &format!(
        "entry:\n  %frame.arity.valid = icmp eq i64 %frame.word.count, {scalar_count}\n  br i1 %frame.arity.valid, label %frame.control, label %invalid\nframe.control:\n"), 1);
    head = head.replacen(
        "  br i1 %valid, label %prepare, label %invalid\nprepare:\n",
        &format!(
            "  br i1 %valid, label %frame.bounds, label %invalid\n{}prepare:\n",
            region_frame_bounds(scalar_count)
        ),
        1,
    );
    head
}

fn region_frame_bounds(scalar_count: u8) -> String {
    if scalar_count == 0 {
        return "frame.bounds:\n  %frame.empty = icmp eq ptr %frame.words, null\n  br i1 %frame.empty, label %prepare, label %invalid\n".into();
    }
    format!(
        r#"frame.bounds:
  %frame.address = ptrtoint ptr %frame.words to i64
  %frame.nonnull = icmp ne i64 %frame.address, 0
  %frame.alignment = and i64 %frame.address, 7
  %frame.aligned = icmp eq i64 %frame.alignment, 0
  %frame.end = add i64 %frame.address, {}
  %frame.extent.valid = icmp uge i64 %frame.end, %frame.address
  %frame.below.control = icmp ule i64 %frame.end, %address
  %frame.above.control = icmp uge i64 %frame.address, %control.end
  %frame.control.disjoint = or i1 %frame.below.control, %frame.above.control
  %frame.below.owner = icmp ule i64 %frame.end, %base.address
  %frame.above.owner = icmp uge i64 %frame.address, %end
  %frame.owner.disjoint = or i1 %frame.below.owner, %frame.above.owner
  %frame.pointer.valid = and i1 %frame.nonnull, %frame.aligned
  %frame.storage.valid = and i1 %frame.extent.valid, %frame.control.disjoint
  %frame.nonoverlapping = and i1 %frame.storage.valid, %frame.owner.disjoint
  %frame.valid = and i1 %frame.pointer.valid, %frame.nonoverlapping
  br i1 %frame.valid, label %prepare, label %invalid
"#,
        u64::from(scalar_count) * 8
    )
}

fn region_frame_inputs(scalar_count: u8) -> String {
    let mut result = String::new();
    for index in 0..scalar_count {
        let _ = writeln!(
            result,
            "  %frame.word.pointer.{index} = getelementptr i64, ptr %frame.words, i64 {index}\n  %frame.word.{index} = load i64, ptr %frame.word.pointer.{index}, align 8\n  %frame.value.{index} = call ptr @topal.runtime.int.from.u64(i64 %frame.word.{index})"
        );
    }
    result
}

fn region_frame_call(scalar_count: u8, entry: &str) -> String {
    let mut arguments = "ptr %view".to_owned();
    for index in 0..scalar_count {
        let _ = write!(arguments, ", ptr %frame.value.{index}");
    }
    format!("  %accepted = call fastcc i1 @{entry}({arguments})\n")
}

pub(crate) fn region_frame_adapter_sha256(scalar_count: u8) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(region_frame_head(scalar_count).as_bytes());
    hash.update(region_frame_inputs(scalar_count).as_bytes());
    // Callee identity is bound independently by the selected source/object;
    // this rendering also binds the exact single dispatch and argument order.
    hash.update(region_frame_call(scalar_count, "<qualified-entry>").as_bytes());
    hash.update(
        region_bridge_tail(CompilerKernelRegionProfile::FramedAction { scalar_count }).as_bytes(),
    );
    format!("{:x}", hash.finalize())
}

pub(crate) fn region_frame_adapter_family_sha256() -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    for scalar_count in 0..=topal_language::compiler::MAX_KERNEL_REGION_FRAME_SCALARS {
        hash.update(region_frame_adapter_sha256(scalar_count).as_bytes());
    }
    format!("{:x}", hash.finalize())
}
