//! TOPAL-SYSTEMS-USER-EXTENT-001: preserve complete private zero checks.
//! Only the owned-extent target changes; the predecessor target stays bytewise.
use crate::CompileError;

const ORIGINAL: &str = r#"define internal i1 @t.clean(ptr %p,i64 %bytes) nounwind noinline noredzone noimplicitfloat "disable-tail-calls"="true" {
entry:
  br label %loop
loop:
  %i = phi i64 [0,%entry],[%next,%body]
  %more = icmp ult i64 %i,%bytes
  br i1 %more,label %body,label %end
body:
  %a = getelementptr i8,ptr %p,i64 %i
  %v = load volatile i8,ptr %a,align 1
  %zero = icmp eq i8 %v,0
  %next = add i64 %i,1
  br i1 %zero,label %loop,label %bad
end:
  ret i1 true
bad:
  ret i1 false
}"#;
const WIDE: &str = include_str!("runtime/kernel_user_extent_clean.ll");

pub(super) fn adapt(text: String) -> Result<String, CompileError> {
    let invalid =
        || CompileError::Tool("E-SYSTEMS-USER-EXTENT: private check binding changed".into());
    // These are complete, stable, private ordinary-RAM buffers. Width changes
    // are not valid for MMIO, racing aliases or a partial ownership window.
    if text.matches(ORIGINAL).count() != 1 || text.matches("call i1 @t.clean(").count() != 4 {
        return Err(invalid());
    }
    for (name, bytes) in [
        ("private", 65528),
        ("stack", 4096),
        ("facts", 24),
        ("output", 56),
    ] {
        let binding = format!("call i1 @t.clean(ptr @topal_x86_frame_{name}_v1,i64 {bytes})");
        if text.matches(&binding).count() != 1 {
            return Err(invalid());
        }
    }
    for declaration in [
        "@topal_x86_frame_private_v1 = global [65536 x i8] zeroinitializer, align 16",
        "@topal_x86_frame_stack_v1 = global [4096 x i8] zeroinitializer, align 16",
        "@topal_x86_frame_facts_v1 = global [3 x i64] zeroinitializer, align 16",
        "@topal_x86_frame_output_v1 = global [7 x i64] zeroinitializer, align 16",
    ] {
        if text.matches(declaration).count() != 1 {
            return Err(invalid());
        }
    }
    Ok(text.replace(ORIGINAL, WIDE.trim_end()))
}

#[cfg(test)]
#[path = "systems_user_extent_clean_tests.rs"]
mod tests;
