#!/usr/bin/env python3
"""Qualify the exact checked-out Rust workspace with the unchanged host wrapper."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[3]
ENVIRONMENT = {
    "RUST_MIN_STACK": "16777216",
    "RUST_TEST_THREADS": "1",
    "CARGO_BUILD_JOBS": "1",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_CODEGEN_UNITS": "1024",
    "CARGO_PROFILE_DEV_CODEGEN_UNITS": "1024",
    "CARGO_INCREMENTAL": "0",
    "MALLOC_ARENA_MAX": "1",
    "MALLOC_CONF": (
        "narenas:1,dirty_decay_ms:0,muzzy_decay_ms:0,"
        "background_thread:false,retain:false,tcache:false"
    ),
}
COMMANDS = (
    ("check", ["cargo", "check", "--locked", "--workspace", "--all-targets", "-j1"]),
    ("test", ["cargo", "test", "--locked", "--workspace", "--all-targets", "-j1", "--no-fail-fast"]),
    ("doc-test", ["cargo", "test", "--locked", "--workspace", "--doc", "-j1", "--no-fail-fast"]),
)


def capture(*command: str) -> str:
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check-environment", action="store_true")
    args = parser.parse_args()
    for name in ("cargo", "rustc", "cc", "gdb", "systemd-run", "systemctl", "sha256sum"):
        if shutil.which(name) is None:
            parser.error(f"required tool is missing: {name}")
    subprocess.run(["systemctl", "--user", "show-environment"], check=True, stdout=subprocess.DEVNULL)
    tool_directory = Path(capture("rustc", "--print", "target-libdir")).parent / "bin"
    tools = {}
    for name in ("llvm-as", "opt", "llc", "llvm-link", "llvm-nm", "llvm-readobj", "llvm-objdump", "rust-lld"):
        command = [str(tool_directory / name)]
        if name == "rust-lld":
            command += ["-flavor", "gnu"]
        tools[name] = capture(*command, "--version")
    if args.check_environment:
        print(json.dumps({"rustc": capture("rustc", "--version", "--verbose"), "llvm": tools}, indent=2))
        return 0
    if capture("git", "status", "--porcelain", "--untracked-files=no"):
        parser.error("commit tracked changes before recording qualification")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    record_path = output / "qualification.json"
    if record_path.exists():
        parser.error("use a fresh output directory; an earlier result must not survive a failed retry")
    environment = dict(os.environ, **ENVIRONMENT)
    environment["TOPAL_LLVM_TOOLS"] = str(tool_directory)
    record = {
        "schema": 1,
        "head": capture("git", "rev-parse", "HEAD"),
        "tree": capture("git", "rev-parse", "HEAD^{tree}"),
        "platform": platform.platform(),
        "rustc": capture("rustc", "--version", "--verbose"),
        "llvm": tools,
        "runner_sha256": digest(Path(__file__).resolve()),
        "bounded_wrapper_sha256": digest(ROOT / "scripts/run_bounded.py"),
        "environment": {**ENVIRONMENT, "TOPAL_LLVM_TOOLS": str(tool_directory)},
        "commands": [],
        "complete": False,
        "passed": False,
        "scope": "Rust workspace checks, all-target implementation tests and doc tests; no new VM execution",
    }

    def save() -> None:
        record_path.write_text(json.dumps(record, indent=2) + "\n")

    save()
    for name, command in COMMANDS:
        bounded = [sys.executable, str(ROOT / "scripts/run_bounded.py"), "--", *command]
        log = output / f"{name}.log"
        print(f"START {name}: {' '.join(command)}", flush=True)
        entry = {"name": name, "command": bounded, "log": log.name, "returncode": None}
        record["commands"].append(entry)
        save()
        with log.open("w") as stream:
            result = subprocess.run(bounded, cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT)
        entry.update(returncode=result.returncode, log_sha256=digest(log))
        save()
        print(f"{'PASS' if result.returncode == 0 else 'FAIL'} {name}: {log}", flush=True)
        if result.returncode:
            with log.open("rb") as stream:
                stream.seek(max(0, log.stat().st_size - 12000))
                print(stream.read().decode(errors="replace"), file=sys.stderr)
    record["complete"] = True
    record["passed"] = all(entry["returncode"] == 0 for entry in record["commands"])
    save()
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
