#!/usr/bin/env python3
"""Contract tests for the Topal x86-64 toolchain-gate run."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock


LAB = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("topal_gate", LAB / "topal_gate.py")
assert SPEC is not None and SPEC.loader is not None
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class TopalToolchainGateTests(unittest.TestCase):
    def test_pipe_evidence_binds_actual_bytes_exact_conditions_and_reclamation(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-001,
        # TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-TRANSFER-001,
        # TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-WAIT-001
        proof=json.loads((LAB/"results/topal-static-process-pipes-gate.json").read_text())
        self.assertEqual(proof["schema"],"topal-static-process-pipes-qemu/1")
        self.assertEqual(proof["harness_sha256"],GATE.digest(LAB/"pipes_gate.py"))
        self.assertEqual(proof["fixture_sha256"],GATE.digest(LAB/"static_pipe_checks.py"))
        for name in ("clean","external","termination"):
            case=proof[name]
            self.assertEqual(case["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
            self.assertEqual(case["runtime"]["observations"]["result"],"pass")
            observation=case["observation"]
            self.assertEqual(observation["result"],"pass")
            self.assertEqual(set(observation["created_generations"]),set(observation["reclaimed_generations"]))
            self.assertEqual(len(observation["created_generations"]),len(set(observation["created_generations"])))
            for key in ("complete_ring_contents_and_unused_zeros_verified",
                        "all_description_reference_counts_verified","terminal_storage_erased",
                        "terminal_thread_registrations_erased","terminal_library_bindings_erased"):
                self.assertTrue(observation[key])
            for resumed in observation["resumptions"]:
                for key in ("private_backings_preserved","private_table_preserved",
                            "mapping_permissions_preserved","registers_preserved","tls_preserved",
                            "extended_state_preserved","thread_registrations_preserved","shared_history_preserved"):
                    self.assertTrue(resumed[key])
        clean=proof["clean"]["observation"]
        self.assertEqual(len(clean["created_generations"]),9)
        self.assertEqual(clean["unrelated_condition_preserved"],1)
        self.assertEqual(clean["local_yields_with_ineligible_counterpart"],2)
        self.assertEqual(len(clean["resumptions"]),6)
        for operation,buffered in (("read",0),("write",4096)):
            blocked=next(b for b in clean["blocks"] if b["operation"]==operation)
            self.assertEqual(blocked["native_candidate"],0x619000)
            self.assertEqual(blocked["buffered"],buffered)
        external=proof["external"]["observation"]
        self.assertEqual(len(external["blocks"]),4)
        self.assertTrue(external["entropy_history_advanced_across_exec"])
        for operation in ("read","write"):
            positive=[t for t in external["transfers"] if t["operation"]==operation and t["actual"]>0]
            self.assertEqual(sum(t["actual"] for t in positive),8893)
            self.assertTrue(all(t["ordered_bytes_verified"] for t in positive))
        self.assertTrue(proof["termination"]["observation"]["explicit_blocked_child_parent_exit_injected"])
        counter=proof["counterexample"]
        self.assertEqual(counter["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
        self.assertNotIn("runtime",counter)
        rejected=counter["observation"]
        self.assertEqual(rejected["result"],"expected-fifo-wait-identity-rejection")
        self.assertNotEqual(rejected["actual_generation"],rejected["wrong_generation"])
        self.assertTrue(rejected["exact_invariant_failure_entry"])
        self.assertTrue(rejected["inactive_endpoint_generation_verified"])

    def test_resumable_pair_evidence_covers_handoff_wait_exec_and_live_teardown(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-PAIR-001
        proof=json.loads((LAB/"results/topal-static-process-pair-gate.json").read_text())
        self.assertEqual(proof["schema"],"topal-static-process-pair-qemu/1")
        self.assertEqual(proof["harness_sha256"],GATE.digest(LAB/"pair_gate.py"))
        self.assertEqual(proof["fixture_sha256"],GATE.digest(LAB/"static_pair_checks.py"))
        for name in ("clean","external","termination"):
            case=proof[name]
            self.assertEqual(case["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
            self.assertEqual(case["runtime"]["observations"]["result"],"pass")
            self.assertEqual(case["observation"]["result"],"pass")
            for key in ("terminal_storage_erased","terminal_thread_registrations_erased",
                        "terminal_library_bindings_erased"):
                self.assertTrue(case["observation"][key])
            for returned in case["observation"]["returns"]:
                for key in ("registers_preserved","extended_state_preserved","tls_preserved",
                            "private_backings_preserved","private_table_preserved",
                            "mapping_permissions_preserved","thread_registrations_preserved",
                            "shared_history_preserved","description_references_preserved"):
                    self.assertTrue(returned[key])
        clean=proof["clean"]["observation"]
        self.assertEqual([r["pid"] for r in clean["returns"]],[1,6,1,6,1,6,1,1,7,1,1,8,1])
        self.assertEqual(clean["blocking_waits"],2)
        self.assertEqual(clean["local_yields_while_parent_waits"],2)
        self.assertTrue(clean["second_bank_retained"])
        self.assertEqual(sum(r["retired_snapshot_erased"] for r in clean["returns"]),3)
        external=proof["external"]["observation"]
        self.assertTrue(external["explicit_child_yield_injected"])
        self.assertTrue(external["external_exec_after_handoff"])
        self.assertTrue(external["entropy_history_advanced_across_child_exec"])
        self.assertEqual(external["blocking_waits"],1)
        termination=proof["termination"]["observation"]
        self.assertTrue(termination["explicit_live_parent_exit_injected"])
        self.assertEqual(len(termination["returns"]),11)
        self.assertTrue(termination["second_bank_retained"])
        counter=proof["counterexample"]
        self.assertEqual(counter["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
        self.assertNotIn("runtime",counter)
        self.assertEqual(counter["observation"]["result"],"expected-pair-restoration-rejection")
        self.assertEqual(counter["observation"]["observed"],0x12345679)
        self.assertEqual(counter["observation"]["expected"],0x12345678)
        self.assertTrue(counter["observation"]["exact_user_comparison"])

    def test_child_evidence_binds_private_restoration_shared_history_and_erasure(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-001, TOPAL-SYSTEMS-STATIC-PROCESS-CHILD-WAIT-001
        # TOPAL-SYSTEMS-STATIC-PROCESS-THREAD-REGISTRATION-001
        record = json.loads((LAB / "results/topal-static-process-children-gate.json").read_text())
        self.assertEqual(record["schema"], "topal-static-process-children-qemu/1")
        self.assertEqual(record["harness_sha256"], sha256(LAB / "children_gate.py"))
        self.assertEqual(record["fixture_sha256"], sha256(LAB / "static_child_checks.py"))
        for name, count in (("clean", 4), ("external", 1)):
            case = record[name]
            self.assertEqual(case["linked_kernel_sha256"], self.evidence["linked_kernel_sha256"])
            self.assertEqual(case["runtime"]["linked_kernel_sha256"], case["linked_kernel_sha256"])
            self.assertEqual(case["runtime"]["observations"]["result"], "pass")
            observation = case["observation"]
            self.assertEqual(observation["result"], "pass")
            self.assertEqual(observation["process_backing_bytes"], 0x43000)
            for flag in ("terminal_storage_erased", "terminal_thread_registrations_erased",
                         "terminal_library_bindings_erased"):
                self.assertTrue(observation[flag])
            self.assertEqual(len(observation["calls"]), count)
            for call in observation["calls"]:
                self.assertGreaterEqual(call["pid"], 2)
                for flag in ("child_returned", "parent_returned", "registers_preserved",
                             "extended_state_preserved", "tls_preserved", "private_backings_preserved",
                             "private_process_state_preserved", "shared_description_references_preserved",
                             "snapshot_erased", "thread_registrations_restored", "entropy_history_preserved"):
                    self.assertTrue(call[flag])
            self.assertEqual(case["runtime"]["static_user_image_sha256"],
                             GATE.PINNED_STATIC_BUSYBOX_SHA256 if name == "external" else
                             hashlib.sha256(GATE.static_user_fixture()).hexdigest())
        self.assertEqual([call["pid"] for call in record["clean"]["observation"]["calls"]], [2, 3, 4, 5])
        self.assertEqual([call["status"] for call in record["clean"]["observation"]["calls"]], [7, 0, 0, 0])
        self.assertEqual(record["clean"]["observation"]["calls"][-1]["maximum_live_descriptions"],32)
        external = record["external"]["observation"]["calls"][0]
        self.assertTrue(external["child_exec"])
        self.assertTrue(external["entropy_history_advanced"])
        counterexample = record["counterexample"]
        self.assertNotIn("runtime", counterexample)
        self.assertEqual(counterexample["linked_kernel_sha256"], self.evidence["linked_kernel_sha256"])
        rejected = counterexample["observation"]
        self.assertEqual(rejected["result"], "expected-parent-restoration-rejection")
        self.assertTrue(rejected["snapshot_byte_injected"])
        self.assertTrue(rejected["exact_user_comparison"])
        self.assertEqual(rejected["expected"], 0x12345678)
        self.assertEqual(rejected["observed"], 0x12345679)
        self.assertEqual(rejected["execution"], "stopped before completion after independent rejection")

    def test_observer_cleanup_preserves_a_reused_slave_descriptor(self) -> None:
        # Failure/debugger threads can open a descriptor after the PTY slave
        # has been closed. Cleanup must never close that new owner's handle.
        original_openpty = GATE.pty.openpty
        pair = []
        retained = []
        process = mock.Mock()
        process.poll.return_value = None
        process.wait.return_value = 0

        def opening():
            opened = original_openpty()
            pair.extend(opened)
            return opened

        def failure():
            descriptor = os.open(os.devnull, os.O_RDONLY)
            retained.append(descriptor)
            self.assertEqual(descriptor, pair[1])
            return "controlled observer failure"

        try:
            with tempfile.TemporaryDirectory() as temporary, \
                    mock.patch.object(GATE.pty, "openpty", side_effect=opening), \
                    mock.patch.object(GATE, "qemu_command", return_value=["test-qemu"]), \
                    mock.patch.object(GATE.subprocess, "Popen", return_value=process):
                root = Path(temporary)
                with self.assertRaisesRegex(GATE.GateError, "controlled observer failure"):
                    GATE.observe_gate(root / "image", root / "archive", root / "serial", root / "qemu.log",
                                      1, failure_check=failure)
                os.fstat(retained[0])
                process.terminate.assert_called_once()
        finally:
            for descriptor in retained:
                os.close(descriptor)

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
        # Physical checks cover the dynamic stack and recoverable failures.
        entry = fixture[0xA_6100:0xA_6580]
        self.assertIn(bytes.fromhex("4881fc00f87f00"), entry)
        self.assertEqual(fixture[0xA_B000:0xA_B100],
                         b"/alias/argv0\0true\0\0" + b"x" * 232 + bytes(5))
        terminal = bytes.fromhex("bf00905f00be00935f00ba00746000b83b0000000f050f0b")
        continuation_end = fixture.index(terminal, 0xA_7000) + len(terminal)
        continuation = fixture[0xA_7000 : continuation_end]
        self.assertLessEqual(len(continuation), 0x3000)  # disjoint runtime island
        self.assertIn(bytes.fromhex("b83f0000000f05"), continuation)
        self.assertIn(bytes.fromhex("bf60945f00be00100000b84f0000000f05"), continuation)
        self.assertIn(bytes.fromhex("31ffbe608f5f00baff03000031c00f05"), continuation)
        self.assertIn(bytes.fromhex("bf010000004889e6b8050000000f05"), continuation)
        self.assertIn(bytes.fromhex("48817c24188021000075fe"), continuation)
        self.assertIn(bytes.fromhex("48817c24280044000075fe"), continuation)
        self.assertIn(bytes.fromhex("48817c24380010000075fe"), continuation)
        self.assertIn(bytes.fromhex("31c0b9120000004889e7fcf348ab"), continuation)
        self.assertIn(bytes.fromhex("31c0b9120000004889e7fcf348af75fe"), continuation)
        self.assertIn(
            bytes.fromhex("bf03000000b8030000000f054885c075fe"), continuation
        )
        for offset, expected in enumerate(b"\x7fELF"):
            self.assertIn(
                bytes.fromhex(
                    "bf03000000488d7424"
                    + f"{offset:02x}"
                    + "ba0100000031c00f054883f80175fe807c24"
                    + f"{offset:02x}{expected:02x}"
                    + "75fe"
                ),
                continuation,
            )
        self.assertIn(
            bytes.fromhex(
                "48b862757379626f782d48890424"
                "48b8726561640a0000004889442408"
                "bf010000004889e6ba0d000000b8010000000f054883f80d75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "bf03000000be06040000ba0a000000b8480000000f054883f80a75fe"
                "bf03000000b8030000000f054885c075fe"
                "bf0a000000b8030000000f054885c075fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b862757379626f782d48890424"
                "48b8636c6f73650a00004889442408"
                "bf010000004889e6ba0e000000b8010000000f054883f80e75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48b82f62696e2f6275734889042560945f00"
                "48b879626f78000000004889042568945f00"
                "bf9cffffffbe60945f0031d24531d2b8010100000f054883f80375fe"
                "48c7042560945f000000000048c7042568945f0000000000"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "c704242f202320bf020000004889e6ba04000000b8010000000f05"
                "4883f80475fe48c704240000000048833c240075fe"
            ),
            continuation,
        )
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
        self.assertEqual(
            continuation.count(bytes.fromhex("31ffbe608f5f00ba")),
            5,
        )
        self.assertIn(bytes.fromhex("4883f80b75fe"), continuation)
        self.assertIn(bytes.fromhex("48b86563686f20746f70"), continuation)
        self.assertIn(bytes.fromhex("48b8616c0a0000000000"), continuation)
        self.assertIn(bytes.fromhex("bf010000004889e6ba06000000b8010000000f05"), continuation)
        self.assertEqual(
            continuation.count(
                bytes.fromhex(
                    "c704242f202320bf020000004889e6ba04000000b8010000000f05"
                    "4883f80475fe48c704240000000048833c240075fe"
                )
            ),
            5,
        )
        self.assertIn(bytes.fromhex("4883f80575fe"), continuation)
        self.assertIn(bytes.fromhex("48b86364202f0a000000"), continuation)
        self.assertIn(
            bytes.fromhex(
                "66c7042560945f002f00"
                "bf60945f00b8500000000f054885c075fe"
                "66c7042560945f000000"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("4883f85075fe"), continuation)
        self.assertIn(bytes.fromhex("48b86563686f202f2a3b"), continuation)
        self.assertIn(bytes.fromhex("48b82074657374202d64"), continuation)
        self.assertIn(bytes.fromhex("48b8202f62696e202626"), continuation)
        self.assertIn(bytes.fromhex("48b8206563686f202f62"), continuation)
        self.assertIn(bytes.fromhex("48b8696e2f2a3b207465"), continuation)
        self.assertIn(bytes.fromhex("48b87374202d66202f62"), continuation)
        self.assertIn(bytes.fromhex("48b8696e2f6275737962"), continuation)
        self.assertIn(bytes.fromhex("48b86f78202626206563"), continuation)
        self.assertIn(bytes.fromhex("48b8686f206275737962"), continuation)
        self.assertIn(bytes.fromhex("48b86f782d66696c653b"), continuation)
        self.assertIn(bytes.fromhex("48b82074657374202d78"), continuation)
        self.assertIn(bytes.fromhex("48b8202f62696e2f6275"), continuation)
        self.assertIn(bytes.fromhex("48b87379626f78202626"), continuation)
        self.assertIn(bytes.fromhex("48b8206563686f206275"), continuation)
        self.assertIn(bytes.fromhex("48b87379626f782d6578"), continuation)
        self.assertIn(bytes.fromhex("48b86563757461626c65"), continuation)
        self.assertIn(bytes.fromhex("48b83b2074657374202d"), continuation)
        self.assertIn(bytes.fromhex("48b873202f62696e2f62"), continuation)
        self.assertIn(bytes.fromhex("48b8757379626f782026"), continuation)
        self.assertIn(bytes.fromhex("48b826206563686f2062"), continuation)
        self.assertIn(bytes.fromhex("48b8757379626f782d6e"), continuation)
        self.assertIn(bytes.fromhex("48b86f6e656d7074793b"), continuation)
        self.assertIn(bytes.fromhex("48b8206578656320333c"), continuation)
        self.assertIn(bytes.fromhex("48b82f62696e2f627573"), continuation)
        self.assertIn(bytes.fromhex("48b879626f783b206563"), continuation)
        self.assertIn(bytes.fromhex("48b8686f206275737962"), continuation)
        self.assertIn(bytes.fromhex("48b86f782d6f70656e3b"), continuation)
        self.assertIn(bytes.fromhex("48b8782d726561643b20"), continuation)
        self.assertIn(bytes.fromhex("48b86578656320333c26"), continuation)
        self.assertIn(bytes.fromhex("48b82d3b206563686f20"), continuation)
        self.assertIn(bytes.fromhex("48b862757379626f782d"), continuation)
        self.assertIn(bytes.fromhex("48b8636c6f73650a0000"), continuation)
        self.assertIn(bytes.fromhex("483d2601000075fe"), continuation)
        self.assertIn(bytes.fromhex("4883f81875fe"), continuation)
        self.assertIn(bytes.fromhex("48b86578656320333c2f"), continuation)
        self.assertIn(bytes.fromhex("48b862696e2f62757379"), continuation)
        self.assertIn(bytes.fromhex("48b8626f783b7077640a"), continuation)
        self.assertIn(
            bytes.fromhex(
                "66c7042560945f002f00"
                "bf9cffffffbe60945f00ba000809004531d2b8010100000f05"
                "4883f80375fe66c7042560945f000000"
                "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
                "bf030000004889e6b8050000000f054885c075fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48833c240075fe48837c24080375fe48837c24100375fe"
                "48817c2418ed41000075fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("48817c24380010000075fe"), continuation)
        self.assertIn(
            bytes.fromhex(
                "bf030000004889e6ba00020000b8d90000000f054883f84875fe"
                "48833c240375fe48837c24080175fe"
                "66837c24101875fe807c24120475fe807c24132e75fe"
                "48837c24180375fe48837c24200275fe"
                "66837c24281875fe807c242a0475fe807c242b2e75fe807c242c2e75fe"
                "48837c24300475fe48837c24380375fe"
                "66837c24401875fe807c24420475fe"
                "807c24436275fe807c24446975fe807c24456e75fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "bf030000004889e6ba00020000b8d90000000f054885c075fe"
                "bf03000000b8030000000f054885c075fe"
                "48b82f62696e0a00000048890424"
                "bf010000004889e6ba05000000b8010000000f054883f80575fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "bf9cffffffbe60945f004889e24531d2b8060100000f054885c075fe"
                "48833c240075fe48837c24080475fe48837c24100275fe"
                "48817c2418ed41000075fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("48b82f62696e2f000000"), continuation)
        self.assertIn(
            bytes.fromhex(
                "bf030000004889e6ba00020000b8d90000000f054883f85075fe"
                "48833c240475fe48837c24080175fe"
            ),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "48837c24300575fe48837c24380375fe"
                "66837c24402075fe807c24420875fe"
                "48b862757379626f7800483944244375fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("48b82f62696e2f627573"), continuation)
        self.assertIn(bytes.fromhex("48b879626f780a000000"), continuation)
        self.assertIn(
            bytes.fromhex("bf010000004889e6ba0d000000b8010000000f054883f80d75fe"),
            continuation,
        )
        repeated_file_metadata = bytes.fromhex(
            "bf9cffffffbe60945f004889e24531d2b8060100000f054885c075fe"
            "48833c240075fe48837c24080575fe48837c24100175fe"
            "48817c24186d81000075fe"
        )
        self.assertEqual(continuation.count(repeated_file_metadata), 3)
        self.assertEqual(
            continuation.count(bytes.fromhex("48817c243060e41e0075fe")), 3
        )
        self.assertEqual(
            continuation.count(bytes.fromhex("48817c2440730f000075fe")), 3
        )
        self.assertIn(bytes.fromhex("b86b0000000f054885c075fe"), continuation)
        self.assertIn(bytes.fromhex("48b862757379626f782d"), continuation)
        self.assertIn(bytes.fromhex("48b86578656375746162"), continuation)
        self.assertIn(bytes.fromhex("c74424106c650a00"), continuation)
        self.assertIn(
            bytes.fromhex("bf010000004889e6ba13000000b8010000000f054883f81375fe"),
            continuation,
        )
        self.assertIn(bytes.fromhex("48b86e6f6e656d707479"), continuation)
        self.assertIn(bytes.fromhex("48c74424100a000000"), continuation)
        self.assertIn(
            bytes.fromhex("bf010000004889e6ba11000000b8010000000f054883f81175fe"),
            continuation,
        )
        self.assertIn(
            bytes.fromhex(
                "bf9cffffffbe60945f0031d24531d2b8010100000f054883f80375fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("48b86f70656e0a000000"), continuation)
        self.assertNotIn(bytes.fromhex("b8b70100000f05"), continuation)
        self.assertIn(bytes.fromhex("48b82f62696e2f627573"), continuation)
        self.assertIn(
            bytes.fromhex(
                "bf9cffffffbe60945f004889e24531d2b8060100000f054885c075fe"
                "48833c240075fe48837c24080575fe48837c24100175fe"
                "48817c24186d81000075fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("48b862757379626f782d"), continuation)
        self.assertIn(bytes.fromhex("48b866696c650a000000"), continuation)
        self.assertIn(
            bytes.fromhex("bf010000004889e6ba0d000000b8010000000f054883f80d75fe"),
            continuation,
        )
        self.assertIn(bytes.fromhex("4883f81875fe"), continuation)
        self.assertIn(bytes.fromhex("48b86578656320333c2f"), continuation)
        self.assertIn(bytes.fromhex("48b862696e2f62757379"), continuation)
        self.assertIn(bytes.fromhex("48b8626f783b7077640a"), continuation)
        self.assertIn(
            bytes.fromhex(
                "bf60945f00be00100000b84f0000000f054883f80275fe"
                "66813c2560945f002f0075fe"
                "c6042561945f000a66813c2560945f002f0a75fe"
                "bf01000000be60945f00ba02000000b8010000000f054883f80275fe"
            ),
            continuation,
        )
        self.assertIn(bytes.fromhex("4883f81775fe"), continuation)
        self.assertIn(bytes.fromhex("48b865786563202f6269"), continuation)
        self.assertIn(bytes.fromhex("48833c25788f5f000075fe"), continuation)
        self.assertTrue(continuation.endswith(bytes.fromhex("bf00905f00be00935f00ba00746000b83b0000000f050f0b")))
        self.assertEqual(fixture[0xA_6900 : 0xA_6912], b"/etc/busybox.conf\0")
        self.assertEqual(fixture[0xA_6912 : 0xA_691B], b"/dev/tty\0")
        self.assertEqual(fixture[0xA_6920 : 0xA_6935], b"/var/spool/mail/root\0")
        self.assertEqual(fixture[0xA_6938 : 0xA_6945], b"/etc/profile\0")

    def test_evidence_binds_inputs_artifacts_and_runtime_observations(self) -> None:
        evidence = self.evidence
        self.assertEqual(
            evidence["observations"]["static_process_runtime_profile"],
            "clean-negative-retry-maximum-aggregate-console-status-witness",
        )
        for bound in (self.evidence, self.external_evidence):
            self.assertEqual(bound["observations"]["static_execve_environment"],
                             "caller-owned-up-to-sixteen-entries-and-1024-bytes")
            self.assertEqual(bound["observations"]["static_execve_descriptor_inheritance"],
                             "reopened-non-cloexec-fd-three")
            self.assertEqual(bound["observations"]["static_execve_random_profile"],
                             "fresh-generator-extraction-before-reentry")
        self.assertEqual(evidence["schema"], "topal-kernel-toolchain-gate-qemu/232")
        self.assertEqual(evidence["publication"], "topalc-target-interface/1")
        self.assertEqual(evidence["provider"], "topal.provider.x86_64-qemu-pc-q35/222")
        self.assertEqual(
            evidence["artifact_schema"],
            "topal.systems-artifact.x86_64-qemu-pc-q35/232",
        )
        self.assertEqual(
            evidence["boot_schema"],
            "topal.boot-adapter.linux-x86-protocol-2.15-q35/132",
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
        self.assertEqual(observations["static_applet_newline_count"], 21)
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
        self.assertEqual(
            observations["static_shell_profile_close_profile"],
            "clean-consumed-system-profile-description-witness",
        )
        self.assertEqual(
            observations["static_shell_prompt_profile"],
            "clean-exact-root-prompt-witness",
        )
        self.assertEqual(
            observations["static_shell_refill_input_profile"],
            "clean-five-publication-command-refill-witness",
        )
        self.assertEqual(
            observations["static_shell_cd_root_profile"],
            "clean-exact-root-chdir-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_open_profile"],
            "clean-exact-root-directory-open-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_fstat_profile"],
            "clean-complete-root-directory-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_enumeration_profile"],
            "clean-three-record-root-directory-enumeration-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_output_profile"],
            "clean-exact-named-root-glob-match-witness",
        )
        self.assertEqual(
            observations["static_shell_named_root_entry_profile"],
            "clean-sealed-bin-directory-entry-witness",
        )
        self.assertEqual(
            observations["static_shell_named_root_metadata_profile"],
            "clean-complete-bin-directory-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_named_root_condition_profile"],
            "clean-selected-bin-busybox-glob-condition-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_open_profile"],
            "clean-exact-bin-directory-open-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_fstat_profile"],
            "clean-complete-bin-directory-fstat-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_enumeration_profile"],
            "clean-three-record-bin-busybox-enumeration-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_close_profile"],
            "clean-explicit-bin-directory-close-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_output_profile"],
            "clean-exact-bin-busybox-glob-match-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_entry_profile"],
            "clean-sealed-bin-busybox-dirent-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_metadata_profile"],
            "clean-complete-bin-busybox-file-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_condition_output_profile"],
            "clean-exact-bin-busybox-file-condition-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_executable_metadata_profile"],
            "clean-repeated-complete-bin-busybox-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_executable_credential_profile"],
            "clean-effective-root-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_executable_condition_output_profile"
            ],
            "clean-exact-bin-busybox-executable-condition-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_artifact_size_profile"],
            "clean-sealed-2024544-byte-bin-busybox-artifact-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_nonempty_metadata_profile"],
            "clean-third-complete-positive-bin-busybox-metadata-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_nonempty_condition_output_profile"
            ],
            "clean-exact-bin-busybox-nonempty-condition-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_read_only_open_profile"],
            "clean-exact-bin-busybox-read-only-open-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_open_condition_output_profile"
            ],
            "clean-exact-bin-busybox-open-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_header_read_profile"],
            "clean-four-byte-elf-header-read-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_header_terminal_probe_profile"
            ],
            "clean-header-terminal-probes-not-issued",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_header_poll_profile"],
            "clean-header-poll-not-issued",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_header_read_condition_output_profile"
            ],
            "clean-exact-bin-busybox-read-output-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_explicit_close_duplicate_profile"
            ],
            "clean-exact-f-dupfd-cloexec-ten-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_explicit_close_profile"],
            "clean-ordered-three-then-ten-close-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_explicit_close_condition_output_profile"
            ],
            "clean-exact-bin-busybox-close-output-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_explicit_close_terminal_absence_profile"
            ],
            "clean-pre-reopen-proved-closed-description-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_reopen_profile"],
            "clean-fresh-generation-descriptor-three-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_reopen_terminal_revoke_profile"],
            "clean-terminal-revoked-reopened-description-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_header_read_terminal_revoke_profile"
            ],
            "clean-explicit-close-revoked-header-read-state-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_open_terminal_revoke_profile"],
            "clean-reopen-terminal-revoked-live-bin-busybox-description-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_command_profile"],
            "clean-command-getcwd-root-output-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_console_fstat_profile"],
            "clean-command-console-fstat-absent",
        )
        self.assertEqual(
            observations["static_shell_pwd_terminal_probe_profile"],
            "clean-command-stdout-terminal-probe-absent",
        )
        self.assertEqual(
            observations["static_shell_pwd_getcwd_profile"],
            "clean-command-getcwd-root-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_output_profile"],
            "clean-exact-application-root-line-witness",
        )
        self.assertEqual(
            observations["static_shell_stack_profile"],
            "clean-affine-two-page-rw-nx-stack-witness",
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
            + "/ # "
            + "topal\n"
            + "/ # "
            + "/ # "
            + "/bin\n"
            + "/bin/busybox\n"
            + "busybox-file\n"
            + "busybox-executable\n"
            + "busybox-nonempty\n"
            + "busybox-open\n"
            + "busybox-read\n"
            + "busybox-close\n"
            + "/ # "
            + "/\n"
            + "/ # "
            + "runtime-alias\nruntime-clean\n" + "z" * 256
            + "\nTOPAL_KERNEL_PROCESS_EXIT_STATUS=23\n"
            + "".join(observations["markers_in_order"][newline_position:])
        )
        self.assertEqual(observations["serial_utf8"].replace(GATE.EXEC_COMMIT_MARKER.decode(), ""), expected_serial)
        self.assertEqual(
            observations["serial_sha256"],
            hashlib.sha256(observations["serial_utf8"].encode()).hexdigest(),
        )

    def test_external_evidence_binds_the_pinned_busybox_without_a_host_path(self) -> None:
        evidence = self.external_evidence
        self.assertEqual(evidence["observations"]["static_process_runtime_profile"],
                         "external-busybox-runtime-arguments-console-exit-witness")
        self.assertEqual(evidence["schema"], "topal-kernel-toolchain-gate-qemu/232")
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
        self.assertEqual(observations["static_applet_newline_count"], 12)
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
        self.assertEqual(
            observations["static_shell_profile_close_profile"],
            "external-optional-system-profile-close-absent",
        )
        self.assertEqual(
            observations["static_shell_prompt_profile"],
            "external-optional-root-prompt-absent",
        )
        self.assertEqual(
            observations["static_shell_refill_input_profile"],
            "external-five-publication-command-refill-witness",
        )
        self.assertEqual(
            observations["static_shell_cd_root_profile"],
            "external-busybox-exact-root-chdir-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_open_profile"],
            "external-busybox-exact-root-directory-open-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_fstat_profile"],
            "external-busybox-complete-root-directory-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_enumeration_profile"],
            "external-busybox-three-record-root-directory-enumeration-witness",
        )
        self.assertEqual(
            observations["static_shell_root_glob_output_profile"],
            "external-busybox-exact-named-root-glob-match-witness",
        )
        self.assertEqual(
            observations["static_shell_named_root_entry_profile"],
            "external-busybox-sealed-bin-directory-entry-witness",
        )
        self.assertEqual(
            observations["static_shell_named_root_metadata_profile"],
            "external-busybox-complete-bin-directory-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_named_root_condition_profile"],
            "external-busybox-selected-bin-busybox-glob-condition-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_open_profile"],
            "external-busybox-exact-bin-directory-open-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_fstat_profile"],
            "external-busybox-complete-bin-directory-fstat-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_enumeration_profile"],
            "external-busybox-three-record-bin-busybox-enumeration-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_close_profile"],
            "external-busybox-explicit-bin-directory-close-witness",
        )
        self.assertEqual(
            observations["static_shell_empty_bin_output_profile"],
            "external-busybox-exact-bin-busybox-glob-match-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_entry_profile"],
            "external-busybox-sealed-bin-busybox-dirent-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_metadata_profile"],
            "external-busybox-complete-bin-busybox-file-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_condition_output_profile"],
            "external-busybox-exact-bin-busybox-file-condition-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_executable_metadata_profile"],
            "external-busybox-repeated-complete-bin-busybox-metadata-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_executable_credential_profile"],
            "external-busybox-effective-root-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_executable_condition_output_profile"
            ],
            "external-busybox-exact-bin-busybox-executable-condition-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_artifact_size_profile"],
            "external-busybox-sealed-2024544-byte-bin-busybox-artifact-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_nonempty_metadata_profile"],
            "external-busybox-third-complete-positive-bin-busybox-metadata-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_nonempty_condition_output_profile"
            ],
            "external-busybox-exact-bin-busybox-nonempty-condition-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_read_only_open_profile"],
            "external-busybox-exact-bin-busybox-read-only-open-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_open_condition_output_profile"
            ],
            "external-busybox-exact-bin-busybox-open-output-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_header_read_profile"],
            "external-busybox-four-byte-elf-header-read-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_header_terminal_probe_profile"
            ],
            "external-busybox-ordered-five-regular-file-enotty-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_header_poll_profile"],
            "external-busybox-four-pollin-readiness-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_header_read_condition_output_profile"
            ],
            "external-busybox-exact-bin-busybox-read-output-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_explicit_close_duplicate_profile"
            ],
            "external-busybox-exact-f-dupfd-cloexec-ten-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_explicit_close_profile"],
            "external-busybox-ordered-three-then-ten-close-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_explicit_close_condition_output_profile"
            ],
            "external-busybox-exact-bin-busybox-close-output-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_explicit_close_terminal_absence_profile"
            ],
            "external-busybox-pre-reopen-proved-closed-description-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_reopen_profile"],
            "external-busybox-fresh-generation-descriptor-three-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_reopen_terminal_revoke_profile"],
            "external-busybox-terminal-revoked-reopened-description-witness",
        )
        self.assertEqual(
            observations[
                "static_shell_bin_busybox_header_read_terminal_revoke_profile"
            ],
            "external-busybox-explicit-close-revoked-header-read-state-witness",
        )
        self.assertEqual(
            observations["static_shell_bin_busybox_open_terminal_revoke_profile"],
            "external-busybox-reopen-terminal-revoked-live-bin-busybox-description-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_command_profile"],
            "external-busybox-pwd-cached-root-output-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_console_fstat_profile"],
            "external-command-console-fstat-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_terminal_probe_profile"],
            "external-command-stdout-enotty-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_getcwd_profile"],
            "external-cached-sealed-root-witness",
        )
        self.assertEqual(
            observations["static_shell_pwd_output_profile"],
            "external-exact-application-root-line-witness",
        )
        self.assertEqual(
            observations["static_shell_stack_profile"],
            "external-affine-two-page-rw-nx-stack-witness",
        )
        marker_serial = "".join(observations["markers_in_order"])
        echo_position = marker_serial.index(
            "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK"
        )
        expected_serial = (
            marker_serial[:echo_position]
            + "topal\n/bin\n/bin/busybox\nbusybox-file\nbusybox-executable\n"
            + "busybox-nonempty\nbusybox-open\nbusybox-read\nbusybox-close\n/\n"
            + "\nTOPAL_KERNEL_PROCESS_EXIT_STATUS=00\n"
            + marker_serial[echo_position:]
        )
        self.assertEqual(observations["serial_utf8"].replace(GATE.EXEC_COMMIT_MARKER.decode(), ""), expected_serial)

    def test_external_runtime_corpus_uses_shared_services_and_one_kernel(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-{EXEC-ARGUMENTS,CONSOLE-WRITE,EXIT-STATUS}-001
        identities = set()
        for workload, (command, output, status) in GATE.PROCESS_WORKLOADS.items():
            path = (LAB / "results" / f"topal-runtime-{workload}-gate.json")
            record = self.external_evidence if workload == "true" else json.loads(path.read_text())
            self.assertEqual(record["process_command_utf8"].encode(), command)
            self.assertEqual(record["process_command_sha256"], hashlib.sha256(command).hexdigest())
            self.assertEqual(record["process_output_utf8"].encode(), output)
            self.assertEqual(record["process_exit_status"], status)
            self.assertEqual(record["static_user_image_sha256"], GATE.PINNED_STATIC_BUSYBOX_SHA256)
            serial = record["observations"]["serial_utf8"].encode().replace(GATE.EXEC_COMMIT_MARKER, b"")
            self.assertIn(b"/\n" + output + f"\nTOPAL_KERNEL_PROCESS_EXIT_STATUS={status:02x}\n".encode(), serial)
            self.assertTrue(record["observations"]["qemu_running_after_markers"])
            self.assertTrue(record["observations"]["serial_quiescent_after_fatal_disposition"])
            identities.add(record["linked_kernel_sha256"])
        self.assertEqual(len(identities), 1)

    def test_file_and_signal_boundaries_bind_the_executed_clean_fixture(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-FD-{TABLE,READ,SEEK,METADATA,DUPLICATION,FLAGS,CLOSE}-001
        # TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001
        self.assertEqual(self.evidence["static_user_image_sha256"],
                         hashlib.sha256(GATE.static_user_fixture()).hexdigest())
        observations = self.evidence["observations"]
        self.assertTrue(observations["reusable_clean_boundary_corpus"])
        self.assertFalse(self.external_evidence["observations"]["reusable_clean_boundary_corpus"])
        self.assertEqual(observations["reusable_file_description_profile"],
                         "sixteen-owned-slots-shared-offsets-original-member-backing")
        self.assertEqual(observations["reusable_signal_registration_profile"],
                         "process-owned-actions-and-blocked-mask-without-delivery")
        self.assertIn("runtime-alias\nruntime-clean\n", observations["serial_utf8"])
        self.assertEqual(self.evidence["process_exit_status"], 0x23)
        self.assertTrue(observations["qemu_running_after_markers"])
        self.assertTrue(observations["serial_quiescent_after_fatal_disposition"])

    def test_namespace_and_prepared_identity_use_the_executed_boundary_corpus(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-PATH-{COPY,LOOKUP,OPEN,METADATA}-001
        # TOPAL-SYSTEMS-STATIC-PROCESS-{DIRECTORY-READ,CWD,ROOT-CREDENTIALS}-001
        observations = self.evidence["observations"]
        self.assertEqual(observations["reusable_namespace_profile"],
                         "owned-path-bytes-independent-opens-directory-cookies-and-cwd")
        self.assertEqual(observations["root_credentials_profile"],
                         "borrowed-prepared-root-identity-independent-of-metadata-order")
        self.assertEqual(self.evidence["static_user_image_sha256"],
                         hashlib.sha256(GATE.static_user_fixture()).hexdigest())
        self.assertTrue(observations["reusable_clean_boundary_corpus"])
        cases = json.loads((LAB / "namespace-cases.json").read_text())
        self.assertEqual(cases["schema"], "topal-kernel-namespace-witness/1")
        self.assertTrue(cases["pre_exec"])
        self.assertTrue(cases["after_exec"])

    def test_realtime_cases_observe_declared_hardware_dates_and_encoding(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-REALTIME-SECONDS-001
        evidence = json.loads((LAB / "results" / "topal-realtime-seconds-gate.json").read_text())
        self.assertEqual(evidence["schema"], "topal-kernel-realtime-seconds-qemu/1")
        expected = {
            "epoch": ("1970-01-01T00:00:00", "bcd", 0),
            "leap-day": ("2000-02-29T12:34:56", "bcd", 951827696),
            "non-leap-century": ("2100-03-01T00:00:00", "bcd", 4107542400),
            "last-whole-year": ("2261-12-31T23:59:00", "bcd", 9214646340),
            "binary-leap-day": ("2000-02-29T12:34:56", "binary", 951827696),
        }
        self.assertEqual({record["case"] for record in evidence["cases"]}, set(expected))
        for record in evidence["cases"]:
            date, encoding, epoch = expected[record["case"]]
            self.assertEqual(record["qualification_rtc_base"], date)
            self.assertEqual(record["qualification_rtc_encoding"], encoding)
            self.assertEqual(record["epoch_minimum"], epoch)
            self.assertEqual(record["epoch_maximum"], epoch + 30)
            self.assertGreaterEqual(record["observed_epoch"], epoch)
            self.assertLessEqual(record["observed_epoch"], epoch + 30)
            self.assertEqual(record["process_exit_status"], 0)
            self.assertEqual(record["process_command_utf8"], "exec /bin/busybox date +%s\n")
            self.assertEqual(record["linked_kernel_sha256"], self.external_evidence["linked_kernel_sha256"])
            self.assertEqual(record["static_user_image_sha256"], GATE.PINNED_STATIC_BUSYBOX_SHA256)
            observations = record["observations"]
            self.assertIn(f"/\n{record['observed_epoch']}\n\nTOPAL_KERNEL_PROCESS_EXIT_STATUS=00\n",
                          observations["serial_utf8"].replace(GATE.EXEC_COMMIT_MARKER.decode(), ""))
            self.assertEqual(observations["result"], "pass")
            self.assertTrue(observations["qemu_running_after_markers"])
            self.assertTrue(observations["serial_quiescent_after_fatal_disposition"])

    def test_exec_environments_and_payloads_bind_each_executed_fixture(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-{ENVIRONMENT,PAYLOAD}-001
        hashes=set()
        for mode in ("maximum", "empty", "null"):
            record = self.evidence if mode == "maximum" else json.loads(
                (LAB / "results" / f"topal-runtime-env-{mode}-gate.json").read_text())
            self.assertEqual(record["clean_environment_mode"], mode)
            expected=hashlib.sha256(GATE.static_user_fixture(mode)).hexdigest()
            self.assertEqual(record["static_user_image_sha256"], expected)
            self.assertEqual(record["linked_kernel_sha256"], self.external_evidence["linked_kernel_sha256"])
            self.assertEqual(record["process_exit_status"], 0x23)
            observations=record["observations"]
            self.assertEqual(observations["static_execve_environment"],
                             "caller-owned-up-to-sixteen-entries-and-1024-bytes")
            self.assertEqual(observations["static_execve_stack_payload_profile"],
                             "above-entry-vectors-separated-argv0-independent-execution-name")
            self.assertTrue(observations["reusable_clean_boundary_corpus"])
            self.assertEqual(observations["result"], "pass")
            self.assertTrue(observations["qemu_running_after_markers"])
            self.assertTrue(observations["serial_quiescent_after_fatal_disposition"])
            hashes.add(expected)
        self.assertEqual(len(hashes),3)

    def test_completed_deadline_notification_preserves_the_interrupted_frame(self) -> None:
        # TOPAL-SYSTEMS-DEADLINE-EVENT-001
        evidence=json.loads((LAB/"results/topal-deadline-entry-filter-gate.json").read_text())
        self.assertEqual(evidence["schema"],"topal-deadline-completed-notification-qemu/1")
        self.assertEqual(evidence["harness_sha256"],GATE.digest(LAB/"deadline_gate.py"))
        current=evidence["current"]
        self.assertEqual(current["result"],"pass")
        self.assertEqual(current["completed_event_state"],3)
        self.assertEqual(current["raw_vector"],242)
        self.assertTrue(current["filtered_entry_observed"])
        self.assertTrue(current["setup_registers_unchanged"])
        self.assertTrue(current["setup_frame_unchanged"])
        self.assertEqual(current["filtered_frame_sha256"],current["interrupted_frame_sha256"])
        self.assertEqual(current["temporary_helper_vector"],{"original":241,"injected":242,"restored":241})
        self.assertEqual(current["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
        serial=current["serial_utf8"]
        for marker in ("TOPAL_KERNEL_DEADLINE_PREEMPTED","TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED",
                       "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED"):
            self.assertEqual(serial.count(marker),1)
        self.assertEqual(current["runtime"]["observations"]["result"],"pass")
        self.assertTrue(current["runtime"]["observations"]["qemu_running_after_markers"])
        self.assertTrue(current["runtime"]["observations"]["serial_quiescent_after_fatal_disposition"])
        previous=evidence["predecessor"]
        self.assertEqual(previous["result"],"expected-predecessor-rejection")
        self.assertFalse(previous["filtered_entry_observed"])
        self.assertEqual(previous["completed_event_state"],3)
        self.assertIn("TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED",previous["serial_utf8"])
        self.assertNotIn("TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED",previous["serial_utf8"])

    def test_prepared_process_observations_bind_queries_and_replacement_shells(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-{IDENTITY,UNAME}-001
        # TOPAL-SYSTEMS-STATIC-PROCESS-ENOSYS-001
        self.assertEqual(self.evidence["observations"]["static_unknown_syscall_profile"],
                         "normalized-selector-enosys-without-argument-or-state-access")
        self.assertEqual(self.evidence["static_user_image_sha256"],
                         hashlib.sha256(GATE.static_user_fixture()).hexdigest())
        expected={"replacement-shell-pid":("1\n",0,2),
                  "replacement-shell-cwd":("1\n/bin\n",0,2),
                  "replacement-shell-script":("one\n/\n/bin\n",7,1),
                  "uname-kernel":("Linux 7.2.9-topal x86_64\n",0,1)}
        for workload,(output,status,commits) in expected.items():
            record=json.loads((LAB/"results"/f"topal-runtime-{workload}-gate.json").read_text())
            self.assertEqual(record["process_output_utf8"],output)
            self.assertEqual(record["process_exit_status"],status)
            self.assertEqual(record["process_exec_commits"],commits)
            self.assertEqual(record["observations"]["prepared_process_observation_profile"],
                             "retained-pid-tid-parent-and-immutable-390-byte-uts-view")
            self.assertEqual(record["observations"]["result"],"pass")
            self.assertEqual(record["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])

    def test_readonly_open_access_and_interactive_serial_cycles(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-PATH-OPEN-ACCESS-001
        self.assertEqual(self.evidence["static_user_image_sha256"],
                         hashlib.sha256(GATE.static_user_fixture()).hexdigest())
        self.assertEqual(self.evidence["observations"]["readonly_open_access_profile"],
                         "lookup-and-type-errors-before-immutable-view-write-denial")
        for workload, status in (("interactive-prompt", 7), ("interactive-prompt-changed", 9)):
            record=json.loads((LAB/"results"/f"topal-runtime-{workload}-gate.json").read_text())
            command,output,_=GATE.PROCESS_WORKLOADS[workload]
            self.assertEqual(record["process_command_utf8"].encode(),command)
            self.assertEqual(record["process_output_utf8"].encode(),output)
            self.assertEqual(record["process_exit_status"],status)
            self.assertEqual(record["process_exec_commits"],1)
            self.assertEqual(record["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
            self.assertEqual(record["observations"]["result"],"pass")
            self.assertTrue(record["observations"]["qemu_running_after_markers"])
            self.assertTrue(record["observations"]["serial_quiescent_after_fatal_disposition"])

    def test_borrowed_clock_relative_waits_and_controlled_saturation(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-{MONOTONIC,RELATIVE-SLEEP}-001
        self.assertEqual(self.evidence["static_user_image_sha256"],
                         hashlib.sha256(GATE.static_user_fixture()).hexdigest())
        self.assertEqual(self.evidence["observations"]["static_process_time_profile"],
                         "borrowed-root-clock-normalized-observations-and-saturating-relative-deadlines")
        evidence=json.loads((LAB/"results/topal-static-process-time-gate.json").read_text())
        self.assertEqual(evidence["schema"],"topal-static-process-time-qemu/1")
        self.assertEqual(evidence["harness_sha256"],sha256(LAB/"time_gate.py"))
        self.assertEqual({r["case"] for r in evidence["cases"]},
                         {"conversion-saturation","addition-saturation","resolution-rounding"})
        maximum=(1<<63)-1
        for record in evidence["cases"]:
            seconds,nanos=record["seconds"],record["nanoseconds"]
            duration=maximum if seconds>=9223372036 else seconds*1000000000+nanos
            self.assertEqual(record["converted_duration_nanoseconds"],duration)
            sample=(record["deadline_sample_epoch"]<<64)+record["deadline_sample_ticks"]
            target=(min(sample*10+duration,maximum)+9)//10
            self.assertEqual(record["expected_deadline_ticks"],target)
            self.assertEqual(record["observed_deadline_ticks"],target)
            self.assertTrue(record["borrowed_root_clock_binding"])
            self.assertTrue(record["initial_root_clock_state_zero"])
            self.assertTrue(record["setup_registers_unchanged"])
            self.assertEqual(record["hardware_counter_requested"],record["hardware_counter_before"]+300000000)
            self.assertGreaterEqual(record["hardware_counter_observed"],record["hardware_counter_requested"])
            self.assertLessEqual(record["hardware_counter_observed"],record["hardware_counter_requested"]+100000)
            if seconds:
                self.assertEqual(target,(maximum+9)//10)
                self.assertEqual(record["advance_hpet_configuration_restored"],1)
                self.assertGreaterEqual(record["advanced_counter_observed"],target)
                self.assertLessEqual(record["advanced_counter_observed"],target+100000)
            else:
                self.assertEqual(target,sample+1)
                self.assertIsNone(record["advanced_counter_observed"])
                self.assertIsNone(record["advance_hpet_configuration_restored"])
            runtime=record["runtime"]
            self.assertEqual(runtime["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
            self.assertEqual(runtime["static_user_image_sha256"],GATE.PINNED_STATIC_BUSYBOX_SHA256)
            self.assertEqual(runtime["process_exit_status"],0)
            self.assertEqual(runtime["process_exec_commits"],1)
            self.assertEqual(runtime["observations"]["result"],"pass")
            self.assertTrue(runtime["observations"]["qemu_running_after_markers"])
            self.assertTrue(runtime["observations"]["serial_quiescent_after_fatal_disposition"])

    def test_owned_numeric_runtime_binds_source_frames_budgets_and_complete_erasure(self) -> None:
        # TOPAL-SYSTEMS-NUMERIC-RUNTIME-001, TOPAL-SYSTEMS-SLEEP-DEADLINE-LIBRARY-001
        proof=json.loads((LAB/"results/topal-owned-numeric-runtime-gate.json").read_text())
        self.assertEqual(proof["schema"],"topal-owned-numeric-runtime-qemu/1")
        self.assertEqual(proof["harness_sha256"],sha256(LAB/"numeric_gate.py"))
        library=proof["library"]
        self.assertEqual(library["source_sha256"],sha256(GATE.LINUX_KERNEL/"kernel/policy/relative-sleep-deadline.t"))
        self.assertEqual(library["allocation"]["runtime_sha256"],sha256(GATE.REPOSITORY/"src/topal-compiler/src/runtime/value_core.ll"))
        allocation=library["allocation"]
        self.assertEqual(allocation["maximum_bytes"],allocation["source_evaluation_bytes"]+allocation["bridge_bytes"]+allocation["header_bytes"])
        self.assertEqual(allocation["maximum_bytes"],41544)
        self.assertEqual(allocation["limit_bytes"],65536)
        costs={"topal_kernel_numeric_allocate_v1":8,"topal_kernel_numeric_abort_v1":8}
        for function in library["functions"]:
            cost=function["frame_bytes"]+8+max((costs[name] for name in function["callees"]),default=0)
            self.assertEqual(cost,function["maximum_call_path_bytes"])
            costs[function["symbol"]]=cost
        maximum=costs[library["native_export"]]+library["native_alignment_reserve_bytes"]
        self.assertEqual(maximum,library["maximum_stack_bytes"])
        self.assertLessEqual(maximum,library["stack_limit_bytes"])
        self.assertEqual(library["native_leaf_return_bytes"],8)
        observation=proof["observation"]
        self.assertEqual(observation["result"],"pass")
        self.assertTrue(observation["binding_erased"])
        self.assertTrue(observation["terminal_arena_erased"])
        self.assertTrue(observation["calls"])
        self.assertIn("relative-sleep-deadline.t",observation["source_runtime_frames"][0])
        self.assertIn("topal.runtime.int.divmod",observation["source_runtime_frames"][0])
        for call in observation["calls"]:
            seconds,nanos,ticks,wrapped=(call[k] for k in ("seconds","nanoseconds","raw_ticks","wrapped"))
            limit=(1<<63)-1
            duration=limit if seconds>=9223372036 else seconds*1000000000+nanos
            expected=((limit if wrapped else min(ticks*10+duration,limit))+9)//10
            self.assertEqual(call["expected_deadline"],expected)
            self.assertEqual(call["observed_deadline"],expected)
            self.assertLessEqual(call["arena_highwater"],allocation["maximum_bytes"])
            for key in ("stack_aligned","interrupts_masked","callee_saved_preserved","simd_preserved",
                        "unused_storage_untouched","arena_erased","retired_stack_erased",
                        "volatile_pointers_cleared","binding_retained"):
                self.assertTrue(call[key],key)
        self.assertEqual(proof["runtime"]["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
        self.assertEqual(proof["runtime"]["observations"]["result"],"pass")
        wrong=proof["counterexample"]
        self.assertEqual(wrong["result"],"expected-deadline-rejection")
        self.assertTrue(wrong["root_and_provider_unchanged"])
        self.assertTrue(wrong["observation"]["independent_deadline_rejection"])
        self.assertEqual(wrong["observation"]["observed_deadline"],0)
        self.assertGreater(wrong["observation"]["expected_deadline"],0)
        self.assertNotEqual(wrong["library"]["object_sha256"],library["object_sha256"])
        self.assertNotIn("runtime",wrong)

    def test_source_kernel_library_executes_checked_source_with_owned_abi(self) -> None:
        # TOPAL-SYSTEMS-SOURCE-LIBRARY-001, TOPAL-SYSTEMS-KERNEL-POLICY-BINDING-001
        proof=json.loads((LAB/"results/topal-kernel-source-library-gate.json").read_text())
        self.assertEqual(proof["schema"],"topal-kernel-source-library-qemu/1")
        self.assertEqual(proof["harness_sha256"],sha256(LAB/"library_gate.py"))
        library=proof["library"]
        self.assertEqual(library["source_sha256"],sha256(GATE.LINUX_KERNEL/"kernel/policy/boot-open-access.t"))
        self.assertEqual(library["source_tags"],[0,1,2,3])
        self.assertEqual(library["stack_limit_bytes"],1024)
        self.assertEqual(library["native_alignment_reserve_bytes"],32)
        costs={}
        for function in library["functions"]:
            cost=function["frame_bytes"]+8+max((costs[name] for name in function["callees"]),default=0)
            self.assertEqual(function["maximum_call_path_bytes"],cost)
            costs[function["symbol"]]=cost
        maximum=costs[library["native_export"]]+32
        self.assertEqual(library["maximum_stack_bytes"],maximum)
        self.assertLessEqual(maximum,1024)
        observation=proof["observation"]
        self.assertTrue(observation["binding_erased"])
        self.assertTrue(observation["complete_finite_domain"])
        self.assertEqual({v["facts"] for v in observation["calls"]},set(range(8)))
        for call in observation["calls"]:
            self.assertEqual(call["choice"],[0,3,0,2,1,1,0,2][call["facts"]])
            self.assertTrue(call["stack_aligned"])
            self.assertTrue(call["callee_saved_preserved"])
            self.assertTrue(call["simd_preserved"])
        self.assertIn("boot-open-access.t",observation["source_helper_frames"][0])
        self.assertIn("topal_kernel_boot_open_policy_v1",observation["source_helper_frames"][0])
        runtime=proof["runtime"]
        self.assertEqual(runtime["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
        self.assertEqual(runtime["observations"]["result"],"pass")
        reordered,wrong=proof["variants"]
        self.assertEqual(reordered["case"],"enum-reordered")
        self.assertEqual(reordered["library"]["source_tags"],[3,2,1,0])
        self.assertEqual(reordered["runtime"]["observations"]["result"],"pass")
        self.assertNotEqual(reordered["linked_kernel_sha256"],runtime["linked_kernel_sha256"])
        self.assertEqual(wrong["result"],"expected-behavior-rejection")
        self.assertEqual(wrong["failure"]["observed_errno"],20)
        self.assertEqual(wrong["failure"]["expected_errno"],30)
        self.assertTrue(wrong["failure"]["user_comparison_failed"])
        self.assertTrue(wrong["failure"]["witness_failure_loop"])
        for variant in (reordered,wrong):
            self.assertTrue(variant["root_and_provider_unchanged"])
            self.assertNotEqual(variant["library"]["object_sha256"],library["object_sha256"])

    def test_successive_exec_entries_bind_live_generations_and_terminal_release(self) -> None:
        # TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-REPEAT-001
        records=[(self.evidence,3)]
        for workload in GATE.PROCESS_WORKLOADS:
            record=self.external_evidence if workload=="true" else json.loads(
                (LAB/"results"/f"topal-runtime-{workload}-gate.json").read_text())
            records.append((record,{"exec-chain":3,"exec-chain-false":2,"exec-chain-cwd":3,
                                    "replacement-shell-pid":2,"replacement-shell-cwd":2,
                                    "child-echo":2,"child-status":2,"child-repeated":3,
                                    "child-cwd":2,"child-offset":3,
                                    "pipe-substitution":2,"pipe-substitution-changed":2,
                                    "pipe-substitution-large":2}.get(workload,1)))
        for record,count in records:
            serial=record["observations"]["serial_utf8"]
            marker=GATE.EXEC_COMMIT_MARKER.decode()
            self.assertEqual(record["process_exec_commits"],count)
            self.assertEqual(serial.count(marker),count)
            self.assertEqual(serial.count("TOPAL_KERNEL_PROCESS_EXIT_STATUS="),1)
            self.assertLess(serial.rindex(marker),serial.index("TOPAL_KERNEL_PROCESS_EXIT_STATUS="))
            self.assertEqual(record["observations"]["result"],"pass")
            self.assertEqual(record["linked_kernel_sha256"],self.evidence["linked_kernel_sha256"])
            self.assertTrue(record["observations"]["qemu_running_after_markers"])
            self.assertTrue(record["observations"]["serial_quiescent_after_fatal_disposition"])


if __name__ == "__main__":
    unittest.main()
