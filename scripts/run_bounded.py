#!/usr/bin/env python3
"""Run a repository command in a host-safe systemd memory cgroup."""

from __future__ import annotations

import argparse
import fcntl
import os
import subprocess
import sys
import uuid
from pathlib import Path


GIB = 1024**3
MINIMUM_BUDGET = 256 * 1024**2


def parse_size(value: str) -> int:
    suffixes = {"K": 1024, "M": 1024**2, "G": GIB, "T": 1024**4}
    normalized = value.strip().upper()
    if not normalized:
        raise ValueError("empty size")
    if normalized[-1] in suffixes:
        return int(normalized[:-1]) * suffixes[normalized[-1]]
    return int(normalized)


def memory_information(path: Path = Path("/proc/meminfo")) -> tuple[int, int]:
    values = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        name, separator, remainder = line.partition(":")
        if separator and name in {"MemTotal", "MemAvailable"}:
            values[name] = int(remainder.split()[0]) * 1024
    if set(values) != {"MemTotal", "MemAvailable"}:
        raise RuntimeError(f"{path} does not report MemTotal and MemAvailable")
    return values["MemTotal"], values["MemAvailable"]


def bounded_memory(
    requested: int, total: int, available: int, reserve: int | None = None
) -> tuple[int, int]:
    retained = reserve if reserve is not None else max(GIB, total // 5)
    safe = available - retained
    if safe < MINIMUM_BUDGET:
        raise RuntimeError(
            "insufficient available memory: "
            f"{available // 1024**2} MiB available, "
            f"{retained // 1024**2} MiB reserved"
        )
    return min(requested, safe), retained


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--memory-limit", default="4G")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    parsed = parser.parse_args()
    if parsed.command[:1] == ["--"]:
        parsed.command = parsed.command[1:]
    if not parsed.command:
        parser.error("a command is required after --")
    return parsed


def main() -> int:
    parsed = arguments()
    requested = parse_size(parsed.memory_limit)
    if requested < 1:
        raise ValueError("--memory-limit must be positive")
    runtime_directory = Path(os.environ.get("XDG_RUNTIME_DIR", "/tmp"))
    lock_path = runtime_directory / f"topal-bounded-{os.getuid()}.lock"
    with lock_path.open("w", encoding="utf-8") as lock:
        print("bounded validation: waiting for exclusive host budget", file=sys.stderr)
        fcntl.flock(lock, fcntl.LOCK_EX)
        total, available = memory_information()
        limit, retained = bounded_memory(requested, total, available)
        unit = f"topal-bounded-{os.getpid()}-{uuid.uuid4().hex[:8]}"
        print(
            "bounded validation: "
            f"limit={limit // 1024**2}MiB swap=0MiB "
            f"host-reserve={retained // 1024**2}MiB",
            file=sys.stderr,
            flush=True,
        )
        completed = subprocess.run(
            [
                "systemd-run",
                "--user",
                "--scope",
                "--quiet",
                "--collect",
                f"--unit={unit}",
                "--property=MemoryAccounting=yes",
                f"--property=MemoryMax={limit}",
                "--property=MemorySwapMax=0",
                "--",
                *parsed.command,
            ],
            check=False,
        )
        return completed.returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, ValueError) as error:
        print(f"run-bounded: {error}", file=sys.stderr)
        raise SystemExit(2) from error
