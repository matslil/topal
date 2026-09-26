#!/usr/bin/env python3
"""Unit tests for host-safe validation memory budgeting."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("run_bounded.py")
SPEC = importlib.util.spec_from_file_location("run_bounded", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {MODULE_PATH}")
RUN_BOUNDED = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = RUN_BOUNDED
SPEC.loader.exec_module(RUN_BOUNDED)


class MemoryBudgetTests(unittest.TestCase):
    def test_requested_limit_is_retained_when_host_has_headroom(self) -> None:
        self.assertEqual(
            RUN_BOUNDED.bounded_memory(
                4 * RUN_BOUNDED.GIB,
                16 * RUN_BOUNDED.GIB,
                12 * RUN_BOUNDED.GIB,
            ),
            (4 * RUN_BOUNDED.GIB, 16 * RUN_BOUNDED.GIB // 5),
        )

    def test_limit_shrinks_to_preserve_host_reserve(self) -> None:
        limit, reserve = RUN_BOUNDED.bounded_memory(
            4 * RUN_BOUNDED.GIB,
            8 * RUN_BOUNDED.GIB,
            3 * RUN_BOUNDED.GIB,
        )
        self.assertEqual(reserve, 8 * RUN_BOUNDED.GIB // 5)
        self.assertEqual(limit, 3 * RUN_BOUNDED.GIB - reserve)

    def test_command_is_refused_when_reserve_cannot_be_preserved(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "insufficient available memory"):
            RUN_BOUNDED.bounded_memory(
                4 * RUN_BOUNDED.GIB,
                8 * RUN_BOUNDED.GIB,
                RUN_BOUNDED.GIB,
            )

    def test_meminfo_requires_total_and_available_memory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "meminfo"
            path.write_text("MemTotal: 8192 kB\n", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "MemTotal and MemAvailable"):
                RUN_BOUNDED.memory_information(path)


class SizeTests(unittest.TestCase):
    def test_binary_suffixes_are_supported(self) -> None:
        self.assertEqual(RUN_BOUNDED.parse_size("4G"), 4 * RUN_BOUNDED.GIB)
        self.assertEqual(RUN_BOUNDED.parse_size("512m"), 512 * 1024**2)


if __name__ == "__main__":
    unittest.main()
