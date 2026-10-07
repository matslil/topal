#!/usr/bin/env python3
"""Contract tests for the Topal x86-64 toolchain-gate run."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import unittest


LAB = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("topal_gate", LAB / "topal_gate.py")
assert SPEC is not None and SPEC.loader is not None
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class TopalToolchainGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads((LAB / "manifest.json").read_text(encoding="utf-8"))
        self.evidence = json.loads(
            (LAB / "results" / "topal-toolchain-gate.json").read_text(encoding="utf-8")
        )

    def test_command_uses_the_pinned_board_without_guest_devices(self) -> None:
        command = GATE.qemu_command(Path("/tmp/bzImage"), Path("/tmp/serial.log"))
        joined = " ".join(map(str, command))
        self.assertIn("pc-q35-10.2", joined)
        self.assertIn("qemu64-v1", joined)
        self.assertIn("tcg,thread=multi", joined)
        self.assertIn("-nodefaults", command)
        self.assertIn("-kernel", command)
        self.assertIn("isa-serial", joined)
        self.assertNotIn("-drive", command)
        self.assertNotIn("-netdev", command)

    def test_evidence_binds_inputs_artifacts_and_runtime_observations(self) -> None:
        evidence = self.evidence
        self.assertEqual(evidence["schema"], "topal-kernel-toolchain-gate-qemu/3")
        self.assertEqual(evidence["publication"], "topalc-target-interface/1")
        self.assertEqual(evidence["manifest_sha256"], sha256(LAB / "manifest.json"))
        self.assertEqual(
            evidence["source_sha256"],
            sha256(GATE.SOURCE_PATH),
        )
        self.assertEqual(evidence["machine"], self.manifest["emulator"]["machine"])
        self.assertEqual(evidence["cpu"], self.manifest["emulator"]["cpu"])
        self.assertEqual(
            evidence["firmware_sha256"], self.manifest["firmware"]["sha256"]
        )
        for field in (
            "linked_kernel_sha256",
            "artifact_provenance_sha256",
            "boot_image_sha256",
            "boot_provenance_sha256",
        ):
            self.assertRegex(evidence[field], r"^[0-9a-f]{64}$")

        observations = evidence["observations"]
        self.assertEqual(
            observations["markers_in_order"],
            [
                "TOPAL_KERNEL_MEMORY_DESCRIBED",
                "TOPAL_KERNEL_BOOT",
                "TOPAL_KERNEL_FAULT_RESUMED",
                "TOPAL_KERNEL_MEMORY_OK",
            ],
        )
        self.assertEqual(observations["result"], "pass")
        self.assertTrue(observations["qemu_running_after_markers"])
        self.assertTrue(observations["serial_quiescent_after_fatal_disposition"])
        self.assertEqual(
            observations["serial_utf8"], "".join(observations["markers_in_order"])
        )
        self.assertEqual(
            observations["serial_sha256"],
            hashlib.sha256(observations["serial_utf8"].encode()).hexdigest(),
        )


if __name__ == "__main__":
    unittest.main()
