#!/usr/bin/env python3
"""Static contract tests for the pinned x86-64 reference VM."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import unittest


LAB = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("topal_x86_vm", LAB / "vm.py")
assert SPEC is not None and SPEC.loader is not None
VM = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VM)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class ReferenceVmContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads((LAB / "manifest.json").read_text(encoding="utf-8"))

    def test_manifest_pins_immutable_inputs_and_artifacts(self) -> None:
        inputs = self.manifest["inputs"]
        self.assertRegex(inputs["debian_cloud_image"]["sha512"], r"^[0-9a-f]{128}$")
        self.assertRegex(inputs["linux"]["sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual(
            inputs["linux"]["configuration_sha256"],
            sha256(LAB / "reference-kernel.config"),
        )
        self.assertRegex(inputs["linux"]["bzimage_sha256"], r"^[0-9a-f]{64}$")
        self.assertNotIn("latest", inputs["debian_cloud_image"]["url"])

    def test_machine_command_has_no_unversioned_board_defaults(self) -> None:
        command = VM.qemu_command("distro", 2222, "file")
        joined = " ".join(command)
        self.assertIn("pc-q35-10.2", joined)
        self.assertIn("qemu64-v1", joined)
        self.assertIn("tcg,thread=multi", joined)
        self.assertIn("-nodefaults", command)
        self.assertIn("virtio-blk-pci-non-transitional", joined)
        self.assertIn("virtio-net-pci-non-transitional", joined)
        self.assertIn("virtio-rng-pci-non-transitional", joined)
        self.assertIn("addr=03.0", joined)
        self.assertIn("addr=04.0", joined)
        self.assertIn("addr=05.0", joined)
        self.assertIn("addr=06.0", joined)

    def test_reference_configuration_covers_the_step3_runtime(self) -> None:
        lines = set((LAB / "reference-kernel.config").read_text(encoding="utf-8").splitlines())
        required = {
            "CONFIG_64BIT=y",
            "CONFIG_CGROUPS=y",
            "CONFIG_MEMCG=y",
            "CONFIG_USER_NS=y",
            "CONFIG_NET_NS=y",
            "CONFIG_SECCOMP_FILTER=y",
            "CONFIG_OVERLAY_FS=y",
            "CONFIG_BRIDGE=y",
            "CONFIG_VETH=y",
            "CONFIG_NF_TABLES=y",
            "CONFIG_VIRTIO_BLK=y",
            "CONFIG_VIRTIO_NET=y",
            "CONFIG_HW_RANDOM_VIRTIO=y",
            "CONFIG_EXT4_FS=y",
            "CONFIG_SERIAL_8250_CONSOLE=y",
            "# CONFIG_IA32_EMULATION is not set",
            "# CONFIG_X86_X32_ABI is not set",
        }
        self.assertEqual(required - lines, set())

    def test_evidence_uses_one_board_and_passes_both_boots(self) -> None:
        evidence = json.loads(
            (LAB / "results" / "verification.json").read_text(encoding="utf-8")
        )
        self.assertEqual(evidence["manifest_sha256"], sha256(LAB / "manifest.json"))
        self.assertEqual(
            evidence["reference_config_sha256"], sha256(LAB / "reference-kernel.config")
        )
        results = {result["boot"]: result for result in evidence["results"]}
        self.assertEqual(set(results), {"distro", "reference"})
        self.assertEqual(results["reference"]["facts"]["kernel_release"], "7.2.9")
        for result in results.values():
            self.assertIn("docker:ok", result["smoke"])
            self.assertIn("podman:ok", result["smoke"])
            self.assertIn("STEP3_SMOKE_OK", result["smoke"])
        self.assertEqual(
            results["distro"]["facts"]["pci"], results["reference"]["facts"]["pci"]
        )
        self.assertEqual(
            results["distro"]["facts"]["acpi_tables"],
            results["reference"]["facts"]["acpi_tables"],
        )


if __name__ == "__main__":
    unittest.main()
