#!/usr/bin/env python3
"""Tests for the pinned Linux interface inventory generator."""

from pathlib import Path
import tempfile
import unittest

import inventory_linux as inventory


class InventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / "source"
        self.headers = self.root / "headers"
        self.source.mkdir()
        (self.headers / "include/asm").mkdir(parents=True)
        (self.headers / "include/asm/unistd_64.h").write_text("#define X 1\n")
        (self.source / "Makefile").write_text(
            "VERSION = 7\nPATCHLEVEL = 2\nSUBLEVEL = 9\nEXTRAVERSION =\n"
        )
        syscall_directory = self.source / "arch/x86/entry/syscalls"
        syscall_directory.mkdir(parents=True)
        (syscall_directory / "syscall_64.tbl").write_text(
            "0 common read sys_read\n"
            "1 64 write sys_write - noreturn\n"
            "2 64 unavailable\n"
            "512 x32 x32_call compat_sys_call\n"
        )
        vdso_directory = self.source / "arch/x86/entry/vdso/vdso64"
        vdso_directory.mkdir(parents=True)
        (vdso_directory / "vdso64.lds.S").write_text(
            "VERSION {\nLINUX_2.6 {\nglobal:\n"
            "plain;\n#ifdef CONFIG_OPTION\nconditional;\n#endif\n"
            "local: *;\n};\n}\n"
        )
        abi_directory = self.source / "Documentation/ABI/stable"
        abi_directory.mkdir(parents=True)
        (abi_directory / "example").write_text("What: /sys/example\n")
        dt_directory = self.source / "Documentation/devicetree/bindings/test"
        dt_directory.mkdir(parents=True)
        (dt_directory / "example.yaml").write_text("$id: example\n")

    def test_release_and_native_syscalls(self) -> None:
        self.assertEqual(inventory.kernel_release(self.source), "7.2.9")
        document = inventory.syscall_inventory(self.source)
        self.assertEqual(
            [entry["number"] for entry in document["entries"]], [0, 1, 2]
        )
        self.assertTrue(document["entries"][1]["noreturn"])
        self.assertFalse(document["entries"][2]["implemented"])

    def test_input_verification_rejects_a_different_release(self) -> None:
        (self.source / "Makefile").write_text(
            "VERSION = 7\nPATCHLEVEL = 2\nSUBLEVEL = 8\nEXTRAVERSION =\n"
        )
        with self.assertRaisesRegex(inventory.InventoryError, "expected Linux 7.2.9"):
            inventory.verify_inputs(self.source, self.headers, None)

    def test_input_verification_rejects_an_unverified_archive(self) -> None:
        archive = self.root / "linux.tar.xz"
        archive.write_bytes(b"not the pinned archive")
        with self.assertRaisesRegex(inventory.InventoryError, "does not match"):
            inventory.verify_inputs(self.source, self.headers, archive)

    def test_vdso_conditions_are_retained(self) -> None:
        entries = inventory.vdso_inventory(self.source)["entries"]
        self.assertEqual(entries[0]["conditions"], [])
        self.assertEqual(entries[1]["conditions"], ["ifdef CONFIG_OPTION"])

    def test_ioctl_definitions_are_indexed_without_copying_expressions(self) -> None:
        header = self.headers / "include/linux/example.h"
        header.parent.mkdir(parents=True)
        header.write_text("#define EXAMPLE _IOW(0x42, 1, int)\n")
        entries = inventory.ioctl_inventory(self.headers)["entries"]
        self.assertEqual(entries[0]["name"], "EXAMPLE")
        self.assertEqual(entries[0]["family"], "_IOW")
        self.assertNotIn("expression", entries[0])

    def test_documents_are_deterministic_and_checkable(self) -> None:
        generated = inventory.documents(self.source, self.headers)
        output = self.root / "output"
        inventory.write_documents(output, generated)
        self.assertEqual(inventory.check_documents(output, generated), [])
        syscall_lines = (output / "syscalls.json").read_text().splitlines()
        self.assertIn('"number": 0', syscall_lines[4])
        (output / "summary.json").write_text("{}\n")
        self.assertEqual(
            inventory.check_documents(output, generated),
            [f"out of date {output / 'summary.json'}"],
        )


if __name__ == "__main__":
    unittest.main()
