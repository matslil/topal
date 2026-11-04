#!/usr/bin/env python3
"""Rebuild TK277 primary VM and compare every artifact with an independent baseline."""

import argparse
import hashlib
import json
from pathlib import Path

import topal_gate
import modular_provenance

lab = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--baseline-artifact", type=Path, required=True)
args = parser.parse_args()
baseline = json.loads((lab / "results/topal-toolchain-gate.json").read_text())
comparison = None
original_record = topal_gate.evidence_record


def compare_record(*values, **keywords):
    global comparison
    comparison = modular_provenance.compare(args.baseline_artifact.resolve(), values[4].parent)
    comparison["controlled_refusals"] = modular_provenance.refusal_controls(
        args.baseline_artifact.resolve(), values[4].parent)
    return original_record(*values, **keywords)


topal_gate.evidence_record = compare_record
fresh = topal_gate.verify(120, False)
assert {key: value for key, value in fresh.items() if key != "artifact_provenance_sha256"} == {
    key: value for key, value in baseline.items() if key != "artifact_provenance_sha256"}
assert comparison["fresh_provenance_sha256"] == fresh["artifact_provenance_sha256"]
assert fresh["linked_kernel_sha256"] == "397e07bfa0c0528e73389ea781e112923b2637a5aa1ee587e4a795cc2cbc7bb0"
record = {
    "schema": "topal.modular-systems-qualification.v1",
    "preservation_baseline": "73e17ab5a50d34561b71db6fb835ed0aae31217f",
    "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    "provenance_comparison_sha256": hashlib.sha256(
        Path(modular_provenance.__file__).read_bytes()).hexdigest(),
    "baseline_builder_sha256": hashlib.sha256(
        (topal_gate.LINUX_KERNEL / "tools/systems-source-audit/build_baseline.py").read_bytes()).hexdigest(),
    "preservation_audits": {
        name: hashlib.sha256((topal_gate.LINUX_KERNEL / "tools/systems-source-audit" / name).read_bytes()).hexdigest()
        for name in ("checkers.py", "backends.py", "rust_items.py")
    },
    "fresh_vm_observations": 1,
    "fresh_primary": fresh,
    "all_primary_fields_except_per_build_provenance_hash_identical": True,
    "verified_provenance_equivalence": comparison,
    "retained_preceding_vm_observations": 137,
    "retained_evidence": [
        {"path": str(path.relative_to(lab)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        for path in sorted((lab / "results").glob("topal-*.json"))
        if path.name != "topal-modular-systems-gate.json"
    ],
}
target = lab / "results/topal-modular-systems-gate.json"
target.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
print("PASS fresh primary VM; native artifacts and verified semantic provenance match baseline")
print("Kernel", fresh["linked_kernel_sha256"])
print("wrote", target)
