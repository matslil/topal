#!/usr/bin/env python3
"""Contract tests for the Topal x86-64 toolchain-gate run."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
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
        self.external_evidence = json.loads(
            (LAB / "results" / "topal-pinned-static-busybox-gate.json").read_text(
                encoding="utf-8"
            )
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

    def test_external_static_image_requires_the_exact_pinned_digest(self) -> None:
        clean, source = GATE.checked_static_user_image(None)
        self.assertEqual(clean, GATE.static_user_fixture())
        self.assertEqual(source, "supplied-clean-room-fixture")
        with tempfile.TemporaryDirectory() as directory:
            candidate = Path(directory) / "busybox"
            candidate.write_bytes(b"not-the-pinned-executable")
            with self.assertRaisesRegex(GATE.GateError, "pinned SHA-256"):
                GATE.checked_static_user_image(candidate)

    def test_static_user_fixture_has_the_approved_clean_room_shape(self) -> None:
        fixture = GATE.static_user_fixture()
        self.assertEqual(len(fixture), 2_024_544)
        self.assertEqual(fixture[:4], b"\x7fELF")
        self.assertEqual(int.from_bytes(fixture[24:32], "little"), 0x004A_6580)
        self.assertEqual(int.from_bytes(fixture[56:58], "little"), 11)
        self.assertEqual(fixture[0x1000], 0xA5)
        self.assertEqual(
            fixture[0xA_6580 : 0xA_6580 + 882],
            bytes(
                [
                    0xB8, 0x0C, 0, 0, 0,
                    0x31, 0xFF,
                    0x0F, 0x05,
                    0x48, 0x3D, 0x00, 0x70, 0x5F, 0x00,
                    0x75, 0xFE,
                    0xBF, 0x80, 0x7D, 0x5F, 0x00,
                    0xB8, 0x0C, 0, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x3D, 0x80, 0x7D, 0x5F, 0x00,
                    0x75, 0xFE,
                    0xC6, 0x04, 0x25, 0x00, 0x70, 0x5F, 0x00, 0x5A,
                    0x80, 0x3C, 0x25, 0x00, 0x70, 0x5F, 0x00, 0x5A,
                    0x75, 0xFE,
                    0xC6, 0x04, 0x25, 0x00, 0x74, 0x5F, 0x00, 0x6B,
                    0xBF, 0x02, 0x10, 0x00, 0x00,
                    0xBE, 0x00, 0x74, 0x5F, 0x00,
                    0xB8, 0x9E, 0, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x85, 0xC0,
                    0x75, 0xFE,
                    0x64, 0x8A, 0x04, 0x25, 0, 0, 0, 0,
                    0x3C, 0x6B,
                    0x75, 0xFE,
                    0xC7, 0x04, 0x25, 0xD0, 0x76, 0x5F, 0x00, 1, 0, 0, 0,
                    0xBF, 0xD0, 0x76, 0x5F, 0x00,
                    0xB8, 0xDA, 0, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x83, 0xF8, 1,
                    0x75, 0xFE,
                    0x48, 0xC7, 0x04, 0x25, 0xE0, 0x76, 0x5F, 0x00, 0xE0, 0x76, 0x5F, 0x00,
                    0x48, 0xC7, 0x04, 0x25, 0xE8, 0x76, 0x5F, 0x00, 0xE0, 0xFF, 0xFF, 0xFF,
                    0x48, 0xC7, 0x04, 0x25, 0xF0, 0x76, 0x5F, 0x00, 0, 0, 0, 0,
                    0xBF, 0xE0, 0x76, 0x5F, 0x00,
                    0xBE, 0x18, 0, 0, 0,
                    0xB8, 0x11, 0x01, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x85, 0xC0,
                    0x75, 0xFE,
                    0xBF, 0x00, 0x73, 0x5F, 0x00,
                    0xBE, 0x21, 0, 0, 0,
                    0x31, 0xD2,
                    0x41, 0xBA, 0x53, 0x30, 0x05, 0x53,
                    0xB8, 0x4E, 0x01, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x83, 0xF8, 0xDA,
                    0x75, 0xFE,
                    0x48, 0x81, 0xEC, 0x40, 0x01, 0, 0,
                    0x31, 0xFF,
                    0xBE, 0x03, 0, 0, 0,
                    0x31, 0xD2,
                    0x49, 0x89, 0xE2,
                    0xB8, 0x2E, 0x01, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x85, 0xC0,
                    0x75, 0xFE,
                    0x48, 0x81, 0x3C, 0x24, 0x00, 0x00, 0x80, 0x00,
                    0x75, 0xFE,
                    0x48, 0x83, 0x7C, 0x24, 0x08, 0xFF,
                    0x75, 0xFE,
                    0x48, 0x89, 0x04, 0x24,
                    0x48, 0x89, 0x44, 0x24, 0x08,
                    0x48, 0x81, 0xC4, 0x40, 0x01, 0, 0,
                    0xBF, 0x9C, 0xFF, 0xFF, 0xFF,
                    0xBE, 0xDA, 0xFD, 0x5B, 0x00,
                    0xBA, 0x00, 0xE0, 0x5F, 0x00,
                    0x41, 0xBA, 0x00, 0x10, 0, 0,
                    0xB8, 0x0B, 0x01, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x83, 0xF8, 0xDA,
                    0x75, 0xFE,
                    0xBF, 0x68, 0x5E, 0x5F, 0x00,
                    0xBE, 0x08, 0, 0, 0,
                    0xBA, 0x01, 0, 0, 0,
                    0xB8, 0x3E, 0x01, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x83, 0xF8, 0x08,
                    0x75, 0xFE,
                    0x4C, 0x8B, 0x14, 0x25, 0x68, 0x5E, 0x5F, 0x00,
                    0x48, 0xC7, 0x04, 0x25, 0x68, 0x5E, 0x5F, 0x00, 0, 0, 0, 0,
                    0xBF, 0x70, 0x5E, 0x5F, 0x00,
                    0xBE, 0x08, 0, 0, 0,
                    0xBA, 0x01, 0, 0, 0,
                    0xB8, 0x3E, 0x01, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x83, 0xF8, 0x08,
                    0x75, 0xFE,
                    0x4C, 0x8B, 0x14, 0x25, 0x70, 0x5E, 0x5F, 0x00,
                    0x48, 0xC7, 0x04, 0x25, 0x70, 0x5E, 0x5F, 0x00, 0, 0, 0, 0,
                    0xB8, 0x0C, 0, 0, 0,
                    0x31, 0xFF,
                    0x0F, 0x05,
                    0x48, 0x3D, 0x80, 0x7D, 0x5F, 0x00,
                    0x75, 0xFE,
                    0xBF, 0x80, 0x8D, 0x61, 0x00,
                    0xB8, 0x0C, 0, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x3D, 0x80, 0x8D, 0x61, 0x00,
                    0x75, 0xFE,
                    0xC6, 0x04, 0x25, 0x7F, 0x8D, 0x61, 0x00, 0x6D,
                    0x80, 0x3C, 0x25, 0x7F, 0x8D, 0x61, 0x00, 0x6D,
                    0x75, 0xFE,
                    0xBF, 0x00, 0x90, 0x61, 0x00,
                    0xB8, 0x0C, 0, 0, 0,
                    0x0F, 0x05,
                    0x48, 0x3D, 0x00, 0x90, 0x61, 0x00,
                    0x75, 0xFE,
                    0xC6, 0x04, 0x25, 0xFF, 0x8F, 0x61, 0x00, 0x71,
                    0x80, 0x3C, 0x25, 0xFF, 0x8F, 0x61, 0x00, 0x71,
                    0x75, 0xFE,
                    0xBF, 0x00, 0x50, 0x5E, 0x00,
                    0xBE, 0x00, 0x80, 0x00, 0x00,
                    0xBA, 0x01, 0x00, 0x00, 0x00,
                    0xB8, 0x0A, 0x00, 0x00, 0x00,
                    0x0F, 0x05,
                    0x48, 0x85, 0xC0,
                    0x75, 0xFE,
                    0x48, 0x81, 0xEC, 0x40, 0x01, 0x00, 0x00,
                    0xBF, 0x10, 0x00, 0x00, 0x00,
                    0x48, 0x89, 0xE6,
                    0xB8, 0x9D, 0x00, 0x00, 0x00,
                    0x0F, 0x05,
                    0x48, 0x85, 0xC0,
                    0x75, 0xFE,
                    0x48, 0xB8, 0x62, 0x75, 0x73, 0x79, 0x62, 0x6F, 0x78, 0x00,
                    0x48, 0x39, 0x04, 0x24,
                    0x75, 0xFE,
                    0x48, 0x83, 0x7C, 0x24, 0x08, 0x00,
                    0x75, 0xFE,
                    0x48, 0xC7, 0x04, 0x24, 0, 0, 0, 0,
                    0x48, 0xC7, 0x44, 0x24, 0x08, 0, 0, 0, 0,
                    0x48, 0x81, 0xC4, 0x40, 0x01, 0x00, 0x00,
                    0xB8, 0x66, 0x00, 0x00, 0x00,
                    0x0F, 0x05,
                    0x48, 0x85, 0xC0,
                    0x75, 0xFE,
                    0x48, 0x81, 0xEC, 0x20, 0x02, 0x00, 0x00,
                    0xBF, 0x9C, 0xFF, 0xFF, 0xFF,
                    0xBE, 0x00, 0x69, 0x4A, 0x00,
                    0x48, 0x89, 0xE2,
                    0x45, 0x31, 0xD2,
                    0xB8, 0x06, 0x01, 0x00, 0x00,
                    0x0F, 0x05,
                    0x48, 0x83, 0xF8, 0xFE,
                    0x75, 0xFE,
                    0x48, 0x81, 0xC4, 0x20, 0x02, 0x00, 0x00,
                    0xB8, 0x68, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
                    0x31, 0xFF,
                    0xB8, 0x6A, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
                    0x31, 0xFF,
                    0xB8, 0x69, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
                    0x48, 0x81, 0xEC, 0x40, 0x02, 0x00, 0x00,
                    0xC6, 0x04, 0x24, 0x0A,
                    0xBF, 0x01, 0x00, 0x00, 0x00,
                    0x48, 0x89, 0xE6,
                    0xBA, 0x01, 0x00, 0x00, 0x00,
                    0xB8, 0x01, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x83, 0xF8, 0x01, 0x75, 0xFE,
                    0xC6, 0x04, 0x24, 0x00,
                    0x48, 0x81, 0xC4, 0x40, 0x02, 0x00, 0x00,
                    0xB8, 0x27, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x83, 0xF8, 0x01, 0x75, 0xFE,
                    0x48, 0x81, 0xEC, 0x40, 0x02, 0x00, 0x00,
                    0x48, 0xC7, 0x04, 0x24, 0x80, 0x65, 0x4A, 0x00,
                    0x48, 0xC7, 0x44, 0x24, 0x08, 0x00, 0x00, 0x00, 0x04,
                    0x48, 0xC7, 0x44, 0x24, 0x10, 0x80, 0x65, 0x4A, 0x00,
                    0x48, 0xB8, 0xFF, 0xFF, 0xFF, 0x7F, 0xFE, 0xFF, 0xFF, 0xFF,
                    0x48, 0x89, 0x44, 0x24, 0x18,
                    0xBF, 0x11, 0x00, 0x00, 0x00,
                    0x48, 0x89, 0xE6,
                    0x31, 0xD2,
                    0x41, 0xBA, 0x08, 0x00, 0x00, 0x00,
                    0xB8, 0x0D, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
                    0x31, 0xC0,
                    0x48, 0x89, 0x04, 0x24,
                    0x48, 0x89, 0x44, 0x24, 0x08,
                    0x48, 0x89, 0x44, 0x24, 0x10,
                    0x48, 0x89, 0x44, 0x24, 0x18,
                    0x48, 0x81, 0xC4, 0x40, 0x02, 0x00, 0x00,
                    0xB8, 0x6E, 0x00, 0x00, 0x00,
                    0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
                    0xE9, 0x0E, 0x07, 0x00, 0x00,
                ]
            ),
        )
        continuation = fixture[0xA_7000 : 0xA_7800]
        self.assertEqual(len(continuation), 2_048)
        self.assertIn(bytes.fromhex("b83f0000000f05"), continuation)
        self.assertIn(bytes.fromhex("bf60945f00be00100000b84f0000000f05"), continuation)
        self.assertIn(bytes.fromhex("31ffbe608f5f00baff03000031c00f05"), continuation)
        self.assertIn(bytes.fromhex("bf010000004889e6b8050000000f05"), continuation)
        self.assertIn(bytes.fromhex("48817c24188021000075fe"), continuation)
        self.assertIn(bytes.fromhex("48817c24280044000075fe"), continuation)
        self.assertIn(bytes.fromhex("48817c24380010000075fe"), continuation)
        self.assertIn(bytes.fromhex("31c0b9120000004889e7fcf348ab"), continuation)
        self.assertIn(bytes.fromhex("31c0b9120000004889e7fcf348af75fe"), continuation)
        self.assertIn(bytes.fromhex("4883f81075fe"), continuation)
        self.assertIn(
            bytes.fromhex(
                "bf9cffffffbe12694a00ba020000004531d2"
                "b8010100000f054883f8fe75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
                "bf9cffffffbe20694a004889e24531d2b8060100000f054883f8fe75fe"
                "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348af75fe"
                "31c0b9120000004889e7fcf348ab"
                "31c0b9120000004889e7fcf348af75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b80a0a42757379426f48890424"
                "48b8782076312e33372e4889442408"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b864732e0a0a0000004889442468"
                "bf010000004889e6ba6d000000b8010000000f054883f86d75fe"
                "31c0b90e0000004889e7fcf348ab"
                "31c0b90e0000004889e7fcf348af75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "c604246d31ffbe015400004889e2b8100000000f054883f8e775fe"
                "803c246d75fec6042400803c240075fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "66c7042560945f00a5a5bf60945f00be00100000b84f0000000f05"
                "4883f80275fe66813c2560945f002f0075fe66c7042560945f000000"
                "66833c2560945f000075fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("b86b0000000f054885c075fe"), continuation)
        self.assertIn(
            bytes.fromhex(
                "bf9cffffffbe38694a0031d24531d2"
                "b8010100000f054883f80375fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
                "bf030000004889e6b8050000000f054885c075fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48833c240075fe48837c24080275fe48837c24100175fe"
                "48817c24182481000075fe48837c24200075fe48837c24280075fe"
                "48837c24300075fe48817c24380010000075fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b8a5a5a5a5a5a5a5a54889e7b910000000fcf348ab"
                "bf030000004889e6ba8000000031c00f054885c075fe"
                "48b8a5a5a5a5a5a5a5a54889e7b910000000fcf348af75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "c604246d"
                "bf02000000be015400004889e2b8100000000f054883f8e775fe803c246d75fe"
                "bf01000000be015400004889e2b8100000000f054883f8e775fe803c246d75fe"
                "31ffbe015400004889e2b8100000000f054883f8e775fe803c246d75fe"
                "c6042400803c240075fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "c7042473683a20bf020000004889e6ba04000000b8010000000f05"
                "4883f80475fec7042400000000833c240075fe"
                "48b863616e277420616348890424"
                "48b863657373207474794889442408"
                "48b83b206a6f6220636f4889442410"
                "48b86e74726f6c2074754889442418"
                "48b8726e6564206f66664889442420"
                "bf020000004889e6ba28000000b8010000000f054883f82875fe"
                "31c0b9050000004889e7fcf348ab"
                "31c0b9050000004889e7fcf348af75fe"
                "c604240abf020000004889e6ba01000000b8010000000f05"
                "4883f80175fec6042400803c240075fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("48b86563686f20746f70"), continuation)
        self.assertIn(bytes.fromhex("48b8616c0a657869740a"), continuation)
        self.assertIn(bytes.fromhex("bf010000004889e6ba06000000b8010000000f05"), continuation)
        self.assertIn(bytes.fromhex("48c70425608f5f0000000000"), continuation)
        self.assertIn(bytes.fromhex("48c70425688f5f0000000000"), continuation)
        self.assertTrue(continuation.endswith(bytes.fromhex("31ffb8e70000000f050f0b")))
        self.assertEqual(fixture[0xA_6900 : 0xA_6912], b"/etc/busybox.conf\0")
        self.assertEqual(fixture[0xA_6912 : 0xA_691B], b"/dev/tty\0")
        self.assertEqual(fixture[0xA_6920 : 0xA_6935], b"/var/spool/mail/root\0")
        self.assertEqual(fixture[0xA_6938 : 0xA_6945], b"/etc/profile\0")

    def test_evidence_binds_inputs_artifacts_and_runtime_observations(self) -> None:
        evidence = self.evidence
        self.assertEqual(evidence["schema"], "topal-kernel-toolchain-gate-qemu/198")
        self.assertEqual(evidence["publication"], "topalc-target-interface/1")
        self.assertEqual(evidence["provider"], "topal.provider.x86_64-qemu-pc-q35/186")
        self.assertEqual(
            evidence["artifact_schema"],
            "topal.systems-artifact.x86_64-qemu-pc-q35/196",
        )
        self.assertEqual(
            evidence["boot_schema"],
            "topal.boot-adapter.linux-x86-protocol-2.15-q35/96",
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
                "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK",
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK",
                "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_OK",
                "TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_OK",
                "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_OK",
                "TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_OK",
                "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PROCESS_ENTRY_RESTORED_OK",
                "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK",
                "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK",
                "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK",
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
        self.assertEqual(observations["static_applet_newline_count"], 8)
        self.assertEqual(
            observations["static_shell_getpid_profile"], "clean-pid-one-witness"
        )
        self.assertEqual(
            observations["static_shell_sigchld_profile"],
            "clean-installed-cleared-witness",
        )
        self.assertEqual(
            observations["static_shell_getppid_profile"], "clean-no-parent-witness"
        )
        self.assertEqual(
            observations["static_shell_echo_command_profile"],
            "clean-sealed-echo-command-witness",
        )
        self.assertEqual(
            observations["static_shell_console_fstat_profile"],
            "clean-complete-console-fstat-witness",
        )
        self.assertEqual(
            observations["static_shell_dev_tty_open_profile"],
            "clean-dev-tty-open-enoent-witness",
        )
        self.assertEqual(
            observations["static_shell_standard_terminal_probe_profile"],
            "clean-stderr-stdout-stdin-enotty-witness",
        )
        self.assertEqual(
            observations["static_shell_terminal_warning_profile"],
            "clean-three-segment-stderr-warning-witness",
        )
        self.assertEqual(
            observations["static_shell_mailpath_stat_profile"],
            "clean-root-mailpath-enoent-witness",
        )
        self.assertEqual(
            observations["static_shell_banner_profile"],
            "clean-exact-109-byte-shell-banner-witness",
        )
        self.assertEqual(
            observations["static_shell_post_banner_terminal_probe_profile"],
            "clean-post-banner-stdin-enotty-witness",
        )
        self.assertEqual(
            observations["static_shell_post_banner_getcwd_profile"],
            "clean-post-banner-sealed-root-witness",
        )
        self.assertEqual(
            observations["static_shell_post_banner_geteuid_profile"],
            "clean-post-banner-effective-root-witness",
        )
        self.assertEqual(
            observations["static_shell_profile_open_profile"],
            "clean-read-only-system-profile-descriptor-three-witness",
        )
        self.assertEqual(
            observations["static_shell_profile_fstat_profile"],
            "clean-complete-system-profile-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_profile_read_profile"],
            "clean-empty-system-profile-eof-witness",
        )
        newline_position = observations["markers_in_order"].index(
            "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK"
        )
        expected_serial = (
            "".join(observations["markers_in_order"][:newline_position])
            + "\n"
            + "sh: can't access tty; job control turned off\n"
            + "\n\nBusyBox v1.37.0 (Debian 1:1.37.0-6+b9) built-in shell (ash)\n"
            + "Enter 'help' for a list of built-in commands.\n\n"
            + "topal\n"
            + "".join(observations["markers_in_order"][newline_position:])
        )
        self.assertEqual(observations["serial_utf8"], expected_serial)
        self.assertEqual(
            observations["serial_sha256"],
            hashlib.sha256(observations["serial_utf8"].encode()).hexdigest(),
        )

    def test_external_evidence_binds_the_pinned_busybox_without_a_host_path(self) -> None:
        evidence = self.external_evidence
        self.assertEqual(evidence["schema"], "topal-kernel-toolchain-gate-qemu/198")
        self.assertEqual(
            evidence["static_user_image_input"],
            "pinned-external-static-busybox",
        )
        self.assertEqual(
            evidence["static_user_image_sha256"],
            GATE.PINNED_STATIC_BUSYBOX_SHA256,
        )
        self.assertEqual(evidence["static_user_image_bytes"], 2_024_544)
        self.assertNotIn("external_static_user_image", evidence)
        self.assertEqual(evidence["observations"]["result"], "pass")
        observations = evidence["observations"]
        self.assertEqual(observations["static_applet_newline_count"], 1)
        self.assertEqual(
            observations["static_shell_getpid_profile"], "external-shell-echo-command"
        )
        self.assertEqual(
            observations["static_shell_sigchld_profile"], "external-shell-echo-command"
        )
        self.assertEqual(
            observations["static_shell_getppid_profile"], "external-shell-echo-command"
        )
        self.assertEqual(
            observations["static_shell_echo_command_profile"],
            "external-busybox-sh-echo-command-output-terminal",
        )
        self.assertEqual(
            observations["static_shell_console_fstat_profile"],
            "external-optional-console-fstat-absent",
        )
        self.assertEqual(
            observations["static_shell_dev_tty_open_profile"],
            "external-optional-dev-tty-open-absent",
        )
        self.assertEqual(
            observations["static_shell_standard_terminal_probe_profile"],
            "external-optional-standard-terminal-probes-absent",
        )
        self.assertEqual(
            observations["static_shell_terminal_warning_profile"],
            "external-optional-terminal-warning-absent",
        )
        self.assertEqual(
            observations["static_shell_mailpath_stat_profile"],
            "external-optional-mailpath-stat-absent",
        )
        self.assertEqual(
            observations["static_shell_banner_profile"],
            "external-optional-shell-banner-absent",
        )
        self.assertEqual(
            observations["static_shell_post_banner_terminal_probe_profile"],
            "external-optional-post-banner-terminal-probe-absent",
        )
        self.assertEqual(
            observations["static_shell_post_banner_getcwd_profile"],
            "external-optional-post-banner-getcwd-absent",
        )
        self.assertEqual(
            observations["static_shell_post_banner_geteuid_profile"],
            "external-optional-post-banner-geteuid-absent",
        )
        self.assertEqual(
            observations["static_shell_profile_open_profile"],
            "external-optional-system-profile-open-absent",
        )
        self.assertEqual(
            observations["static_shell_profile_fstat_profile"],
            "external-optional-system-profile-metadata-absent",
        )
        self.assertEqual(
            observations["static_shell_profile_read_profile"],
            "external-optional-system-profile-read-absent",
        )
        marker_serial = "".join(observations["markers_in_order"])
        echo_position = marker_serial.index(
            "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK"
        )
        expected_serial = marker_serial[:echo_position] + "topal\n" + marker_serial[echo_position:]
        self.assertEqual(observations["serial_utf8"], expected_serial)


if __name__ == "__main__":
    unittest.main()
