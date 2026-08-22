#!/usr/bin/env python3
"""Unit tests for resource-baseline comparison policy."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("test_resource_usage.py")
SPEC = importlib.util.spec_from_file_location("test_resource_usage", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {MODULE_PATH}")
RESOURCE_USAGE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = RESOURCE_USAGE
SPEC.loader.exec_module(RESOURCE_USAGE)


class CompareTests(unittest.TestCase):
    def baseline(self) -> dict[str, object]:
        return {
            "tests": {
                "case": {
                    "cpu_time_ns": 100,
                    "memory_peak_bytes": 1_000,
                }
            }
        }

    def measurement(
        self, cpu_time_ns: int = 100, memory_peak_bytes: int = 1_000
    ) -> dict[str, object]:
        return {
            "case": RESOURCE_USAGE.Measurement(
                cpu_time_ns=cpu_time_ns,
                memory_peak_bytes=memory_peak_bytes,
                status="passed",
            )
        }

    def test_exactly_twenty_percent_increase_is_accepted(self) -> None:
        self.assertEqual(
            RESOURCE_USAGE.compare(self.baseline(), self.measurement(120, 1_200)),
            [],
        )

    def test_more_than_twenty_percent_increase_is_rejected(self) -> None:
        problems = RESOURCE_USAGE.compare(
            self.baseline(), self.measurement(121, 1_201)
        )
        self.assertEqual(len(problems), 2)
        self.assertTrue(all("increased" in problem for problem in problems))

    def test_inventory_changes_are_rejected(self) -> None:
        added = RESOURCE_USAGE.compare({"tests": {}}, self.measurement())
        removed = RESOURCE_USAGE.compare(self.baseline(), {})
        self.assertEqual(added, ["new test lacks baseline: case"])
        self.assertEqual(removed, ["baseline test no longer exists: case"])

    def test_failed_test_is_rejected(self) -> None:
        measured = {
            "case": RESOURCE_USAGE.Measurement(100, 1_000, "timeout")
        }
        self.assertEqual(
            RESOURCE_USAGE.compare(self.baseline(), measured),
            ["test did not pass (timeout): case"],
        )


class BaselineExtensionTests(unittest.TestCase):
    def test_only_new_tests_are_added(self) -> None:
        baseline = {
            "schema": 1,
            "samples_per_test": 50,
            "environment": {"host": "original"},
            "tests": {
                "existing": {
                    "cpu_time_ns": 100,
                    "memory_peak_bytes": 1_000,
                    "review_note": "preserve the complete entry",
                },
                "removed": {
                    "cpu_time_ns": 200,
                    "memory_peak_bytes": 2_000,
                },
            },
        }
        measured = {
            "existing": RESOURCE_USAGE.Measurement(999, 9_999, "passed"),
            "new": RESOURCE_USAGE.Measurement(300, 3_000, "passed"),
        }

        extended, additions = RESOURCE_USAGE.extend_baseline(baseline, measured)

        self.assertEqual(additions, 1)
        self.assertEqual(extended["environment"], {"host": "original"})
        self.assertEqual(extended["tests"]["existing"], baseline["tests"]["existing"])
        self.assertEqual(extended["tests"]["removed"], baseline["tests"]["removed"])
        self.assertEqual(
            extended["tests"]["new"],
            {"cpu_time_ns": 300, "memory_peak_bytes": 3_000},
        )


if __name__ == "__main__":
    unittest.main()
