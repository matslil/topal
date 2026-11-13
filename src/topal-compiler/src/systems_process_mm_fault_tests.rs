//! Source-built diagnostic producers, never a canonical allocator selection.
//! TOPAL-SYSTEMS-PROCESS-MM-001 / FRAME-POLICY-BINDING-001.
use super::{CallerFocus, KernelProfile, execute_profiled};
use std::{fs, path::Path};

#[derive(Clone, Copy)]
pub(super) enum Failure {
    Allocate,
    Release,
}

impl Failure {
    fn name(self) -> &'static str {
        match self {
            Self::Allocate => "allocate-metadata",
            Self::Release => "release-metadata",
        }
    }

    pub(super) fn prepare(self, input: &mut crate::SystemsSourceLibraryInput) {
        // The ordinary program forks a 67-page heap, then the child grows to
        // 101 pages. Only the diagnostic source producer changes. Every
        // controller, admission, delta and fatal guard remains enabled.
        let (count, condition) = match self {
            Self::Allocate => (101, "identity > 0"),
            Self::Release => (67, "identity = 0"),
        };
        let original = "  write (region, cell cursor, identity)\n";
        assert_eq!(input.text.matches(original).count(), 1);
        input.text = input.text.replace(
            original,
            "  write (region, cell cursor, diagnostic-identity (region, identity))\n",
        );
        input.text.push_str(&format!(
            "\n# Diagnostic producer only: the controller must reject this delta.\n\
diagnostic-identity is fn ((region : KernelByteRegion, identity : Nat)) -> Nat\n  \
selected is both ((word (region, 8)) = {count}, {condition})\n  \
selected\n    true then identity + 1\n    false then identity\n"
        ));
        input.source_name = format!("diagnostic/{}/frame-metadata.t", self.name());
    }

    pub(super) fn record(self, work: &Path, input: &crate::SystemsSourceLibraryInput) {
        fs::write(work.join("diagnostic-policy.t"), &input.text).unwrap();
        fs::write(
            work.join("process-mm-policy-failure.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": "topal-process-mm-source-policy-failure-v1",
                "kind": self.name(),
                "source_sha256": crate::artifact::sha256(input.text.as_bytes()),
                "original_source_sha256": crate::artifact::sha256(include_bytes!(
                    "../../../linux-kernel/kernel/memory/frame-metadata.t")),
                "source_name": input.source_name,
                "scope": "Compiled diagnostic source producer and ordinary unmodified syscall fixture; no canonical allocator qualification or guest register/data/instruction injection",
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

fn execute(failure: Failure) {
    execute_profiled(
        KernelProfile {
            memory: true,
            user_extents: true,
            owned_callers: true,
            process_memory: true,
            policy_failure: Some(failure),
        },
        Some(CallerFocus::SourceProcessOwners),
    );
}

#[test]
#[ignore = "requires the bounded QEMU/GDB source-built process-MM failure cohort"]
fn allocation_source_failure_retains_real_process_owners_and_performed_effects() {
    execute(Failure::Allocate);
}

#[test]
#[ignore = "requires the bounded QEMU/GDB source-built process-MM failure cohort"]
fn release_source_failure_retains_published_candidate_and_performed_effects() {
    execute(Failure::Release);
}
