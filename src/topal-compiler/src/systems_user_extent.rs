//! Private x86 owned-extent component. Target callbacks must be reconstructed
//! from the actual root/entry owners; this module grants no publication route.
use crate::CompileError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Owner {
    pub slot: u64,
    pub count: u64,
    pub pins: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Owners {
    pub token_bytes: u64,
    // Static hierarchy, independent low leaf, resident image, retained stack.
    pub roles: [Owner; 4],
}

impl Owners {
    pub(crate) fn validate(&self) -> Result<(), CompileError> {
        let invalid =
            || CompileError::Tool("E-SYSTEMS-USER-EXTENT: incomplete checked root owners".into());
        if self.token_bytes == 0 || self.token_bytes % 24 != 0 {
            return Err(invalid());
        }
        let mut slots = std::collections::BTreeSet::new();
        for (owner, (count, pins)) in self.roles.iter().zip([(5, 1), (1, 1), (503, 1), (2, 2)]) {
            if owner.count != count
                || owner.pins != pins
                || owner.slot >= self.token_bytes / 24
                || !slots.insert(owner.slot)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

pub(crate) fn llvm() -> String {
    format!(
        "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n{}",
        crate::X86_SYSTEMS_DATA_LAYOUT,
        include_str!("runtime/kernel_user_extent.ll")
    )
}

pub(crate) fn target_llvm(
    component: &crate::SystemsTailOwnerComponent,
    owners: &Owners,
) -> Result<String, CompileError> {
    owners.validate()?;
    let mut text = crate::systems_frame_target::memory_llvm(component, owners.token_bytes)?;
    let split = "  %boot = icmp ult i64 %op,5\n";
    if text.matches(split).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-USER-EXTENT: missing exact memory-context adapter".into(),
        ));
    }
    text = text.replace(split, &format!(
        "  %user_extent = icmp eq i64 %op,8\n  br i1 %user_extent,label %user_observation,label %original_context\nuser_observation:\n  %user_owned = call i1 @u.syscall_context(i64 %saved,i64 %identity)\n  ret i1 %user_owned\noriginal_context:\n{split}"));
    let mut target = include_str!("runtime/kernel_user_extent_target.ll").to_owned();
    for (placeholder, role) in [
        "HIERARCHY_OFFSET",
        "LOW_OFFSET",
        "IMAGE_OFFSET",
        "STACK_OFFSET",
    ]
    .into_iter()
    .zip(owners.roles)
    {
        let offset = role
            .slot
            .checked_mul(24)
            .ok_or_else(|| CompileError::Tool("user owner offset overflow".into()))?;
        target = target.replace(&format!("{{{{{placeholder}}}}}"), &offset.to_string());
    }
    target = target.replace(
        "{{ROOT_TOKEN_SLOTS}}",
        &(owners.token_bytes / 24).to_string(),
    );
    if target.contains("{{") {
        return Err(CompileError::Tool(
            "E-SYSTEMS-USER-EXTENT: unresolved target binding".into(),
        ));
    }
    text.push_str(&target);
    Ok(text)
}

pub(crate) fn dispatch_llvm() -> Result<String, CompileError> {
    let mut text = crate::systems_frame_target::memory_dispatch();
    let entry = "define i64 @topal_frame_kernel_dispatch_v1(i64 %op,i64 %a,i64 %b,ptr %token,ptr %facts) nounwind noinline noredzone {\n";
    if text.matches(entry).count() != 1 {
        return Err(CompileError::Tool(
            "E-SYSTEMS-USER-EXTENT: missing exact dispatch boundary".into(),
        ));
    }
    text = text.replace(entry, &format!("{entry}entry:\n  %user_extent = icmp eq i64 %op,8\n  br i1 %user_extent,label %extent,label %previous\nextent:\n  %direction = ptrtoint ptr %token to i64\n  %known_direction = icmp ule i64 %direction,1\n  %no_facts = icmp eq ptr %facts,null\n  %format = and i1 %known_direction,%no_facts\n  call void @m.require(i1 %format)\n  %checked = call i64 @topal_x86_owned_user_extent_v1(i64 %a,i64 %b,i64 %direction)\n  ret i64 %checked\nprevious:\n"));
    text.push_str("\ndeclare i64 @topal_x86_owned_user_extent_v1(i64,i64,i64)\n");
    Ok(text)
}

/// The kernel target has a separate instruction profile from an ordinary
/// source library. Admit only its exact register-observation sites, then apply
/// the unchanged ordinary instruction check to everything else. This is not
/// a substitute for actual context, control-flow or caller/retirement proof.
pub(crate) fn inspect_target_instructions(text: &str) -> Result<(), CompileError> {
    use std::collections::BTreeMap;
    let invalid =
        || CompileError::Tool("E-SYSTEMS-USER-EXTENT: unexpected privileged observation".into());
    let expected = BTreeMap::from([
        (("g.memory_context", "cr3"), 1),
        (("u.syscall_context", "cr3"), 1),
        (("topal_x86_user_extent_begin_v1", "cr0"), 1),
        (("topal_x86_user_extent_begin_v1", "cr4"), 1),
        (("g.memory_context", "rdmsr"), 1),
        (("u.msr", "rdmsr"), 1),
    ]);
    let mut observed = BTreeMap::new();
    let mut function = "";
    let mut ordinary = String::new();
    for line in text.lines() {
        if line.ends_with(">:")
            && let Some((_, name)) = line.split_once(" <")
        {
            function = name.trim_end_matches(">:");
        }
        let Some((address, body)) = line.split_once(':') else {
            ordinary.push_str(line);
            ordinary.push('\n');
            continue;
        };
        if address.trim().is_empty() || !address.trim().chars().all(|c| c.is_ascii_hexdigit()) {
            ordinary.push_str(line);
            ordinary.push('\n');
            continue;
        }
        let mut words = body.split_whitespace();
        let instruction = words.next().unwrap_or("");
        let register = if instruction == "rdmsr" {
            Some("rdmsr")
        } else if body.contains("%cr") {
            let source = words.next().ok_or_else(invalid)?;
            let destination = words.next().ok_or_else(invalid)?;
            if instruction != "movq"
                || words.next().is_some()
                || ![
                    "%rax", "%rbx", "%rcx", "%rdx", "%rsi", "%rdi", "%r8", "%r9", "%r10", "%r11",
                    "%r12", "%r13", "%r14", "%r15",
                ]
                .contains(&destination)
            {
                return Err(invalid());
            }
            Some(match source {
                "%cr0," => "cr0",
                "%cr3," => "cr3",
                "%cr4," => "cr4",
                _ => return Err(invalid()),
            })
        } else {
            None
        };
        if let Some(register) = register {
            let key = (function, register);
            if !expected.contains_key(&key) {
                return Err(invalid());
            }
            *observed.entry(key).or_insert(0) += 1;
        } else {
            ordinary.push_str(line);
            ordinary.push('\n');
        }
    }
    if observed != expected {
        return Err(invalid());
    }
    crate::systems_library::inspect_instructions(&ordinary)
}

#[cfg(test)]
#[path = "systems_user_extent_tests.rs"]
mod tests;
