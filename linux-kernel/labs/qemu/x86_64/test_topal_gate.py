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

    def test_command_uses_the_pinned_board_with_only_the_selected_entropy_device(self) -> None:
        command = GATE.qemu_command(
            Path("/tmp/bzImage"),
            Path("/tmp/initramfs.cpio"),
            Path("/tmp/serial.log"),
            "/dev/pts/9",
        )
        joined = " ".join(map(str, command))
        self.assertIn("pc-q35-10.2", joined)
        self.assertIn("qemu64-v1", joined)
        self.assertIn("tcg,thread=multi", joined)
        self.assertIn("hpet=on", joined)
        self.assertIn("-nodefaults", command)
        self.assertIn("-kernel", command)
        self.assertIn("-initrd", command)
        self.assertIn("isa-serial", joined)
        self.assertIn("serial,id=serial0,path=/dev/pts/9", joined)
        self.assertIn("rng-random,id=entropy0,filename=/dev/urandom", joined)
        self.assertIn("virtio-rng-pci-non-transitional", joined)
        self.assertIn("rng=entropy0,bus=pcie.0,addr=05.0", joined)
        self.assertIn("max-bytes=1024,period=1000", joined)
        self.assertNotIn("-drive", command)
        self.assertNotIn("-netdev", command)

    def test_static_user_fixture_has_the_approved_clean_room_shape(self) -> None:
        fixture = GATE.static_user_fixture()
        self.assertEqual(len(fixture), 2_024_544)
        self.assertEqual(fixture[:4], b"\x7fELF")
        self.assertEqual(int.from_bytes(fixture[24:32], "little"), 0x004A_6580)
        self.assertEqual(int.from_bytes(fixture[56:58], "little"), 11)
        self.assertEqual(fixture[0x1000], 0xA5)

    def test_evidence_binds_inputs_artifacts_and_runtime_observations(self) -> None:
        evidence = self.evidence
        self.assertEqual(evidence["schema"], "topal-kernel-toolchain-gate-qemu/121")
        self.assertEqual(evidence["publication"], "topalc-target-interface/1")
        self.assertEqual(evidence["provider"], "topal.provider.x86_64-qemu-pc-q35/110")
        self.assertEqual(
            evidence["artifact_schema"],
            "topal.systems-artifact.x86_64-qemu-pc-q35/119",
        )
        self.assertEqual(
            evidence["boot_schema"],
            "topal.boot-adapter.linux-x86-protocol-2.15-q35/69",
        )
        self.assertEqual(
            evidence["boot_resource_schema"],
            "topal.boot-resource.x86_64-initial-newc/2",
        )
        self.assertEqual(evidence["boot_resource_member"], "init")
        self.assertEqual(evidence["static_boot_resource_member"], "static-user.elf")
        self.assertEqual(evidence["boot_resource_bytes"], 2_037_200)
        self.assertEqual(evidence["initial_user_image_bytes"], 12_288)
        self.assertEqual(evidence["initial_user_image_input"], "supplied")
        self.assertEqual(evidence["static_user_image_bytes"], 2_024_544)
        self.assertEqual(
            evidence["static_user_image_input"], "supplied-clean-room-fixture"
        )
        self.assertEqual(evidence["serial_input_utf8"], "read")
        self.assertEqual(
            evidence["serial_input_sha256"], hashlib.sha256(b"read").hexdigest()
        )
        self.assertNotEqual(
            evidence["initial_user_image_sha256"],
            evidence["generated_initial_user_image_sha256"],
        )
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
            "boot_resource_sha256",
            "initial_user_image_sha256",
            "generated_initial_user_image_sha256",
            "boot_provenance_sha256",
            "static_user_image_sha256",
        ):
            self.assertRegex(evidence[field], r"^[0-9a-f]{64}$")

        observations = evidence["observations"]
        self.assertEqual(
            observations["markers_in_order"],
            [
                "TOPAL_KERNEL_FRAME_ALLOCATED",
                "TOPAL_KERNEL_FRAME_MAPPED",
                "TOPAL_KERNEL_TRANSLATION_ACTIVE",
                "TOPAL_KERNEL_TRANSLATION_EDITED",
                "TOPAL_KERNEL_FRAME_EXTENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_FRAME_EXTENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK",
                "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK",
                "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK",
                "TOPAL_KERNEL_INTERRUPTS_MASKED",
                "TOPAL_KERNEL_MEMORY_DESCRIBED",
                "TOPAL_KERNEL_PCI_VIRTIO_RNG_OK",
                "TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK",
                "TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED",
                "TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED",
                "TOPAL_KERNEL_PCI_BUS_MASTER_OK",
                "TOPAL_KERNEL_DMA_BUFFERS_OK",
                "TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK",
                "TOPAL_KERNEL_DMA_DEVICE_OWNED",
                "TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK",
                "TOPAL_KERNEL_BOOT",
                "TOPAL_KERNEL_FAULT_RESUMED",
                "TOPAL_KERNEL_ATOMIC_OK",
                "TOPAL_KERNEL_MEMORY_OK",
                "TOPAL_KERNEL_INTERRUPT_OK",
                "TOPAL_KERNEL_TIME_OK",
                "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED",
                "TOPAL_KERNEL_DEADLINE_PREEMPTED",
                "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED",
                "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED",
                "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED",
                "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED",
                "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED",
                "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED",
                "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED",
                "TOPAL_KERNEL_USER_IMAGE_READY",
                "TOPAL_KERNEL_BOOT_RESOURCE_OK",
                "TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE",
                "TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED",
                "TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL",
                "TOPAL_KERNEL_VIRTIO_USED_RING_OK",
                "TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED",
                "TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED",
                "TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED",
                "TOPAL_KERNEL_ENTROPY_POOL_CREATED",
                "TOPAL_KERNEL_ENTROPY_POOL_MIXED",
                "TOPAL_KERNEL_ENTROPY_POOL_CREDITED",
                "TOPAL_KERNEL_ENTROPY_GENERATOR_READY",
                "TOPAL_KERNEL_ENTROPY_OUTPUT_READY",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK",
                "TOPAL_KERNEL_USER_WRITE_OK",
                "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK",
                "TOPAL_KERNEL_USER_GETRANDOM_OK",
                "TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK",
                "TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK",
                "TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK",
                "TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK",
                "TOPAL_KERNEL_USER_READ_OK",
                "TOPAL_KERNEL_USER_CLOSE_OK",
                "TOPAL_KERNEL_USER_DUP_OK",
                "TOPAL_KERNEL_USER_FCNTL_OK",
                "TOPAL_KERNEL_USER_FSTAT_OK",
                "TOPAL_KERNEL_USER_TCGETS_OK",
                "TOPAL_KERNEL_USER_WINSIZE_OK",
                "TOPAL_KERNEL_USER_LSEEK_OK",
                "TOPAL_KERNEL_USER_COPY_TO_OK",
                "TOPAL_KERNEL_USER_COPY_FROM_OK",
                "TOPAL_KERNEL_USER_SYSCALL_OK",
                "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK",
                "TOPAL_KERNEL_USER_IMAGE_STACK_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK",
                "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK",
                "TOPAL_KERNEL_USER_IMAGE_ELF_OK",
                "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK",
                "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK",
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK",
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK",
                "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK",
                "TOPAL_KERNEL_USER_IMAGE_BSS_OK",
                "TOPAL_KERNEL_USER_IMAGE_DATA_OK",
                "TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK",
                "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK",
                "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK",
                "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK",
                "TOPAL_KERNEL_USER_IMAGE_RETAINED",
                "TOPAL_KERNEL_USER_MAPPING_OK",
                "TOPAL_KERNEL_FRAME_DISJOINT_OK",
                "TOPAL_KERNEL_USER_BACKING_OK",
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
