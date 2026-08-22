#!/usr/bin/env python3
"""Record or compare per-test CPU time and peak memory in systemd cgroups."""

from __future__ import annotations

import argparse
import concurrent.futures
import dataclasses
import json
import os
import platform
import resource
import subprocess
import sys
import threading
import time
import uuid
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BASELINE = ROOT / "se" / "test-resource-baseline.json"
THRESHOLD_PERCENT = 20
PRINT_LOCK = threading.Lock()


@dataclasses.dataclass(frozen=True)
class TestCase:
    identity: str
    executable: str
    name: str
    working_directory: str


@dataclasses.dataclass(frozen=True)
class Measurement:
    cpu_time_ns: int
    memory_peak_bytes: int
    status: str


def command(arguments: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
    return subprocess.run(arguments, check=True, text=True, **kwargs)


def cargo_metadata() -> dict[str, Any]:
    result = command(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        capture_output=True,
    )
    return json.loads(result.stdout)


def test_artifacts() -> list[dict[str, Any]]:
    result = command(
        [
            "cargo",
            "build",
            "--workspace",
            "--tests",
            "--message-format=json",
        ],
        cwd=ROOT,
        capture_output=True,
    )
    artifacts = []
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("profile", {}).get("test") is True
            and message.get("executable") is not None
        ):
            artifacts.append(message)
    return artifacts


def discover_tests(rust_min_stack: int) -> list[TestCase]:
    metadata = cargo_metadata()
    package_directories = {
        package["id"]: str(Path(package["manifest_path"]).parent)
        for package in metadata["packages"]
    }
    tests = []
    for artifact in test_artifacts():
        executable = artifact["executable"]
        listed = command(
            [executable, "--list", "--format", "terse"],
            cwd=package_directories[artifact["package_id"]],
            capture_output=True,
            env={**os.environ, "RUST_MIN_STACK": str(rust_min_stack)},
        )
        package = artifact["package_id"].split("#")[-2].rsplit("/", 1)[-1]
        target = artifact["target"]
        target_identity = f"{','.join(target['kind'])}:{target['name']}"
        for line in listed.stdout.splitlines():
            if not line.endswith(": test"):
                continue
            name = line.removesuffix(": test")
            identity = f"{package}::{target_identity}::{name}"
            tests.append(
                TestCase(
                    identity=identity,
                    executable=executable,
                    name=name,
                    working_directory=package_directories[artifact["package_id"]],
                )
            )
    identities = [test.identity for test in tests]
    if len(identities) != len(set(identities)):
        raise RuntimeError("test discovery produced duplicate stable identities")
    return sorted(tests, key=lambda test: test.identity)


def systemctl_properties(unit: str) -> dict[str, str]:
    result = command(
        [
            "systemctl",
            "--user",
            "show",
            unit,
            "--property=ActiveState",
            "--property=SubState",
            "--property=Result",
            "--property=ExecMainCode",
            "--property=ExecMainStatus",
            "--property=CPUUsageNSec",
            "--property=MemoryPeak",
        ],
        capture_output=True,
    )
    return dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)


def measure_test(
    test: TestCase,
    index: int,
    run_identity: str,
    memory_limit: str,
    rust_min_stack: int,
    timeout_seconds: int,
    samples: int,
) -> tuple[str, Measurement]:
    unit = f"topal-test-resource-{run_identity}-{index}.service"
    result_path = ROOT / "target" / "test-resource-usage" / f"{run_identity}-{index}.json"
    result_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        command(
            [
                "systemd-run",
                "--user",
                "--quiet",
                "--remain-after-exit",
                f"--unit={unit.removesuffix('.service')}",
                f"--setenv=RUST_MIN_STACK={rust_min_stack}",
                f"--property=WorkingDirectory={test.working_directory}",
                "--property=CPUAccounting=yes",
                "--property=MemoryAccounting=yes",
                f"--property=MemoryMax={memory_limit}",
                "--property=MemorySwapMax=0",
                "--property=StandardOutput=null",
                "--property=StandardError=null",
                "--",
                sys.executable,
                str(Path(__file__).resolve()),
                "__worker",
                str(result_path),
                str(samples),
                test.executable,
                test.name,
            ],
            capture_output=True,
        )
        deadline = time.monotonic() + timeout_seconds
        while True:
            properties = systemctl_properties(unit)
            if (
                properties.get("SubState") == "exited"
                or properties.get("ActiveState") == "failed"
            ):
                break
            if time.monotonic() >= deadline:
                subprocess.run(
                    ["systemctl", "--user", "stop", unit],
                    check=False,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
                return test.identity, Measurement(0, 0, "timeout")
            time.sleep(0.02)
        result = properties.get("Result", "unknown")
        exit_status = properties.get("ExecMainStatus", "unknown")
        worker_result = (
            json.loads(result_path.read_text(encoding="utf-8"))
            if result_path.is_file()
            else {}
        )
        status = (
            "passed"
            if result == "success"
            and exit_status == "0"
            and worker_result.get("status") == "passed"
            else f"{result}:{exit_status}"
        )
        measurement = Measurement(
            cpu_time_ns=int(worker_result.get("cpu_time_ns", 0)),
            memory_peak_bytes=int(worker_result.get("memory_peak_bytes", 0)),
            status=status,
        )
        with PRINT_LOCK:
            print(
                f"[{index + 1}] {status:>12} "
                f"cpu={measurement.cpu_time_ns / 1_000_000:.1f}ms "
                f"memory={measurement.memory_peak_bytes / 1_048_576:.1f}MiB "
                f"{test.identity}",
                flush=True,
            )
        return test.identity, measurement
    finally:
        subprocess.run(
            ["systemctl", "--user", "stop", unit],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        subprocess.run(
            ["systemctl", "--user", "reset-failed", unit],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        result_path.unlink(missing_ok=True)


def worker(arguments: list[str]) -> int:
    if len(arguments) != 4:
        return 2
    result_path, samples_text, executable, test_name = arguments
    samples = int(samples_text)
    status = "passed"
    for _ in range(samples):
        completed = subprocess.run(
            [executable, "--exact", test_name, "--test-threads=1"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if completed.returncode != 0:
            status = f"exit:{completed.returncode}"
            break
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    measured_samples = samples if status == "passed" else 1
    result = {
        "status": status,
        "cpu_time_ns": int((usage.ru_utime + usage.ru_stime) * 1_000_000_000)
        // measured_samples,
        "memory_peak_bytes": int(usage.ru_maxrss) * 1024,
    }
    Path(result_path).write_text(json.dumps(result), encoding="utf-8")
    return 0 if status == "passed" else 1


def parse_size(value: str) -> int:
    suffixes = {"K": 1024, "M": 1024**2, "G": 1024**3, "T": 1024**4}
    normalized = value.strip().upper()
    if normalized[-1:] in suffixes:
        return int(normalized[:-1]) * suffixes[normalized[-1]]
    return int(normalized)


def available_memory() -> int:
    for line in Path("/proc/meminfo").read_text(encoding="utf-8").splitlines():
        if line.startswith("MemAvailable:"):
            return int(line.split()[1]) * 1024
    raise RuntimeError("/proc/meminfo does not report MemAvailable")


def worker_count(requested: int | None, memory_limit: str) -> int:
    cpus = os.cpu_count() or 1
    if requested is not None:
        return requested
    available = available_memory()
    reserve = max(1024**3, available // 10)
    memory_workers = max(1, (available - reserve) // parse_size(memory_limit))
    return max(1, min(cpus, memory_workers))


def environment() -> dict[str, Any]:
    rustc = command(["rustc", "-Vv"], capture_output=True).stdout.strip().splitlines()
    return {
        "architecture": platform.machine(),
        "logical_cpus": os.cpu_count() or 1,
        "operating_system": platform.system(),
        "rustc": rustc,
    }


def run_measurements(arguments: argparse.Namespace) -> dict[str, Measurement]:
    tests = discover_tests(arguments.rust_min_stack)
    jobs = worker_count(arguments.jobs, arguments.memory_limit)
    print(f"Measuring {len(tests)} tests with {jobs} workers", flush=True)
    run_identity = uuid.uuid4().hex[:10]
    measured: dict[str, Measurement] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as executor:
        futures = [
            executor.submit(
                measure_test,
                test,
                index,
                run_identity,
                arguments.memory_limit,
                arguments.rust_min_stack,
                arguments.timeout,
                arguments.samples,
            )
            for index, test in enumerate(tests)
        ]
        for future in concurrent.futures.as_completed(futures):
            identity, measurement = future.result()
            measured[identity] = measurement
    return measured


def baseline_document(
    measured: dict[str, Measurement], measured_samples: int
) -> dict[str, Any]:
    return {
        "schema": 1,
        "allowed_increase_percent": THRESHOLD_PERCENT,
        "samples_per_test": measured_samples,
        "environment": environment(),
        "tests": {
            identity: {
                "cpu_time_ns": measurement.cpu_time_ns,
                "memory_peak_bytes": measurement.memory_peak_bytes,
            }
            for identity, measurement in sorted(measured.items())
        },
    }


def extend_baseline(
    baseline: dict[str, Any], measured: dict[str, Measurement]
) -> tuple[dict[str, Any], int]:
    """Add measurements for new identities without changing existing entries."""
    expected = baseline["tests"]
    additions = {
        identity: {
            "cpu_time_ns": measured[identity].cpu_time_ns,
            "memory_peak_bytes": measured[identity].memory_peak_bytes,
        }
        for identity in sorted(set(measured) - set(expected))
    }
    extended = dict(baseline)
    extended["tests"] = {**expected, **additions}
    return extended, len(additions)


def failed_tests(measured: dict[str, Measurement]) -> list[str]:
    return [identity for identity, result in measured.items() if result.status != "passed"]


def compare(
    baseline: dict[str, Any], measured: dict[str, Measurement]
) -> list[str]:
    problems = []
    expected = baseline["tests"]
    current_names = set(measured)
    expected_names = set(expected)
    for identity in sorted(current_names - expected_names):
        problems.append(f"new test lacks baseline: {identity}")
    for identity in sorted(expected_names - current_names):
        problems.append(f"baseline test no longer exists: {identity}")
    for identity in sorted(current_names & expected_names):
        current = measured[identity]
        if current.status != "passed":
            problems.append(f"test did not pass ({current.status}): {identity}")
            continue
        for metric in ("cpu_time_ns", "memory_peak_bytes"):
            old = int(expected[identity][metric])
            new = getattr(current, metric)
            if new * 100 > old * (100 + THRESHOLD_PERCENT):
                increase = ((new / old) - 1) * 100 if old else float("inf")
                problems.append(
                    f"{metric} increased {increase:.1f}%: {identity} "
                    f"(baseline={old}, current={new})"
                )
    return problems


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("baseline", "compare"))
    parser.add_argument("--baseline", type=Path, default=DEFAULT_BASELINE)
    parser.add_argument("--jobs", type=int)
    parser.add_argument("--memory-limit", default="4G")
    parser.add_argument("--rust-min-stack", type=int, default=32 * 1024 * 1024)
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--samples", type=int, default=50)
    parser.add_argument("--approve-baseline-update", action="store_true")
    parser.add_argument("--replace-existing-baseline", action="store_true")
    parsed = parser.parse_args()
    if parsed.jobs is not None and parsed.jobs < 1:
        parser.error("--jobs must be at least 1")
    if parsed.samples < 1:
        parser.error("--samples must be at least 1")
    if parsed.mode == "baseline" and not parsed.approve_baseline_update:
        parser.error("baseline mode requires --approve-baseline-update")
    if parsed.mode == "compare" and parsed.approve_baseline_update:
        parser.error("--approve-baseline-update only applies to baseline mode")
    if parsed.replace_existing_baseline and parsed.mode != "baseline":
        parser.error("--replace-existing-baseline only applies to baseline mode")
    if parsed.replace_existing_baseline and not parsed.approve_baseline_update:
        parser.error("--replace-existing-baseline requires --approve-baseline-update")
    return parsed


def main() -> int:
    parsed = arguments()
    measured = run_measurements(parsed)
    failures = failed_tests(measured)
    if failures:
        for failure in sorted(failures):
            print(f"test failed: {failure}", file=sys.stderr)
        return 1
    if parsed.mode == "baseline":
        parsed.baseline.parent.mkdir(parents=True, exist_ok=True)
        added = len(measured)
        document = baseline_document(measured, parsed.samples)
        if parsed.baseline.exists() and not parsed.replace_existing_baseline:
            existing = json.loads(parsed.baseline.read_text(encoding="utf-8"))
            if existing.get("schema") != 1:
                raise RuntimeError("unsupported baseline schema")
            if existing.get("samples_per_test") != parsed.samples:
                raise RuntimeError("baseline sample count differs from --samples")
            document, added = extend_baseline(existing, measured)
        parsed.baseline.write_text(
            json.dumps(document, indent=2) + "\n",
            encoding="utf-8",
        )
        if parsed.replace_existing_baseline:
            print(f"Replaced baseline for {len(measured)} tests in {parsed.baseline}")
        else:
            print(f"Added {added} new tests to baseline {parsed.baseline}")
        return 0
    baseline = json.loads(parsed.baseline.read_text(encoding="utf-8"))
    if baseline.get("schema") != 1:
        raise RuntimeError("unsupported baseline schema")
    if baseline.get("samples_per_test") != parsed.samples:
        raise RuntimeError("baseline sample count differs from --samples")
    problems = compare(baseline, measured)
    if problems:
        print("Resource comparison requires investigation:", file=sys.stderr)
        for problem in problems:
            print(f"- {problem}", file=sys.stderr)
        print(
            "Add new tests with baseline mode and --approve-baseline-update. "
            "After human approval of changed existing measurements, also pass "
            "--replace-existing-baseline.",
            file=sys.stderr,
        )
        return 1
    print(f"All {len(measured)} tests are within the {THRESHOLD_PERCENT}% resource threshold")
    return 0


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "__worker":
        raise SystemExit(worker(sys.argv[2:]))
    try:
        raise SystemExit(main())
    except (KeyError, OSError, RuntimeError, subprocess.CalledProcessError, ValueError) as error:
        print(f"test-resource-usage: {error}", file=sys.stderr)
        raise SystemExit(2) from error
