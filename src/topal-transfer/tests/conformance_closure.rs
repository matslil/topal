use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Closure {
    phases: Vec<Phase>,
    platforms: Vec<Platform>,
}
#[derive(Deserialize)]
struct Phase {
    #[serde(rename = "phase")]
    number: u8,
    disposition: String,
    evidence: Vec<String>,
}
#[derive(Deserialize)]
struct Platform {
    #[serde(rename = "platform")]
    target: String,
    disposition: String,
    evidence: Vec<String>,
}

#[test]
fn closure_has_exact_terminal_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let closure: Closure = serde_json::from_str(
        &fs::read_to_string(root.join("se/data-transfer-conformance.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(closure.phases.len(), 18);
    let phases = closure
        .phases
        .iter()
        .map(|phase| phase.number)
        .collect::<BTreeSet<_>>();
    assert_eq!(phases, (1..=18).collect());
    for phase in &closure.phases {
        assert!(matches!(
            phase.disposition.as_str(),
            "implemented" | "platform-specific" | "deferred"
        ));
        assert!(!phase.evidence.is_empty());
        for evidence in &phase.evidence {
            assert!(root.join(evidence).exists(), "missing evidence {evidence}");
        }
    }
    let expected = ["linux", "windows", "macos", "android", "ios"];
    assert_eq!(
        closure
            .platforms
            .iter()
            .map(|platform| platform.target.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    for platform in &closure.platforms {
        assert!(matches!(
            platform.disposition.as_str(),
            "implemented" | "platform-specific" | "deferred"
        ));
        assert!(!platform.evidence.is_empty());
    }
}
