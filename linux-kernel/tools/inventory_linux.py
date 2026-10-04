#!/usr/bin/env python3
"""Build a deterministic Linux userspace-interface source inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile
from typing import Any, Iterable


SCHEMA = "topal-linux-interface-inventory/1"
BASELINE_RELEASE = "7.2.9"
BASELINE_ARCHIVE_SHA256 = (
    "b4c5dfbe51a364a6c7f03869200f88c8e1f77403539005f14b7fc6bc91b8d8ba"
)
OUTPUT_NAMES = (
    "summary.json",
    "syscalls.json",
    "installed-uapi-headers.json",
    "abi-documents.json",
    "vdso-symbols.json",
    "ioctl-definitions.json",
    "devicetree-bindings.json",
)


class InventoryError(Exception):
    """An input does not describe the pinned inventory boundary."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def canonical_bytes(document: Any) -> bytes:
    return (json.dumps(document, indent=2, sort_keys=True) + "\n").encode("utf-8")


def inventory_bytes(document: dict[str, Any]) -> bytes:
    """Serialize inventories with one entry per line for reviewable diffs."""
    if "entries" not in document:
        return canonical_bytes(document)
    lines = ["{"]
    keys = sorted(document)
    for key_index, key in enumerate(keys):
        suffix = "," if key_index + 1 < len(keys) else ""
        encoded_key = json.dumps(key)
        if key != "entries":
            encoded_value = json.dumps(document[key], sort_keys=True)
            lines.append(f"  {encoded_key}: {encoded_value}{suffix}")
            continue
        lines.append(f"  {encoded_key}: [")
        entries = document[key]
        for entry_index, entry in enumerate(entries):
            entry_suffix = "," if entry_index + 1 < len(entries) else ""
            encoded_entry = json.dumps(entry, sort_keys=True)
            lines.append(f"    {encoded_entry}{entry_suffix}")
        lines.append(f"  ]{suffix}")
    lines.append("}")
    return ("\n".join(lines) + "\n").encode("utf-8")


def aggregate_digest(entries: Iterable[dict[str, Any]]) -> str:
    digest = hashlib.sha256()
    for entry in entries:
        digest.update(canonical_bytes(entry))
    return digest.hexdigest()


def relative_file_records(root: Path, files: Iterable[Path]) -> list[dict[str, Any]]:
    records = []
    for path in sorted(files):
        data = path.read_bytes()
        records.append(
            {
                "path": path.relative_to(root).as_posix(),
                "sha256": sha256_bytes(data),
                "size": len(data),
            }
        )
    return records


def kernel_release(source: Path) -> str:
    values: dict[str, str] = {}
    pattern = re.compile(r"^(VERSION|PATCHLEVEL|SUBLEVEL|EXTRAVERSION)\s*=\s*(.*)$")
    for line in (source / "Makefile").read_text(encoding="utf-8").splitlines():
        match = pattern.match(line)
        if match:
            values[match.group(1)] = match.group(2).strip()
    required = ("VERSION", "PATCHLEVEL", "SUBLEVEL", "EXTRAVERSION")
    if any(field not in values for field in required):
        raise InventoryError("kernel Makefile does not contain a complete release")
    return (
        f"{values['VERSION']}.{values['PATCHLEVEL']}.{values['SUBLEVEL']}"
        f"{values['EXTRAVERSION']}"
    )


def verify_inputs(source: Path, headers: Path, archive: Path | None) -> None:
    if kernel_release(source) != BASELINE_RELEASE:
        raise InventoryError(
            f"expected Linux {BASELINE_RELEASE}, found {kernel_release(source)}"
        )
    syscall_table = source / "arch/x86/entry/syscalls/syscall_64.tbl"
    if not syscall_table.is_file():
        raise InventoryError(f"missing native syscall table: {syscall_table}")
    if not (headers / "include/asm/unistd_64.h").is_file():
        raise InventoryError("headers root is not an x86-64 headers_install result")
    if archive is not None:
        actual = sha256_file(archive)
        if actual != BASELINE_ARCHIVE_SHA256:
            raise InventoryError(
                f"archive digest {actual} does not match {BASELINE_ARCHIVE_SHA256}"
            )


def syscall_inventory(source: Path) -> dict[str, Any]:
    table = source / "arch/x86/entry/syscalls/syscall_64.tbl"
    entries = []
    for line_number, line in enumerate(
        table.read_text(encoding="utf-8").splitlines(), start=1
    ):
        content = line.split("#", 1)[0].strip()
        if not content:
            continue
        fields = content.split()
        if len(fields) < 3:
            raise InventoryError(f"invalid syscall row at {table}:{line_number}")
        number, abi, name = fields[:3]
        if abi == "x32":
            continue
        if abi not in {"common", "64"}:
            raise InventoryError(f"unexpected native syscall ABI {abi!r}")
        record: dict[str, Any] = {
            "abi": abi,
            "entry": fields[3] if len(fields) >= 4 else None,
            "name": name,
            "number": int(number),
            "source_line": line_number,
        }
        if len(fields) >= 5 and fields[4] != "-":
            record["compat_entry"] = fields[4]
        if fields[-1] == "noreturn":
            record["noreturn"] = True
        if len(fields) == 3:
            record["implemented"] = False
        entries.append(record)
    numbers = [entry["number"] for entry in entries]
    if len(numbers) != len(set(numbers)):
        raise InventoryError("native syscall table contains duplicate numbers")
    return {
        "architecture": "x86_64",
        "baseline": BASELINE_RELEASE,
        "entries": entries,
        "excluded_abis": ["x32", "i386"],
        "schema": SCHEMA,
        "source": "arch/x86/entry/syscalls/syscall_64.tbl",
    }


def installed_headers_inventory(headers: Path) -> dict[str, Any]:
    include = headers / "include"
    files = [path for path in include.rglob("*") if path.is_file()]
    entries = relative_file_records(include, files)
    return {
        "architecture": "x86_64",
        "baseline": BASELINE_RELEASE,
        "entries": entries,
        "schema": SCHEMA,
        "source": "make ARCH=x86_64 headers_install",
    }


def abi_document_inventory(source: Path) -> dict[str, Any]:
    root = source / "Documentation/ABI"
    entries = []
    for path in sorted(path for path in root.rglob("*") if path.is_file()):
        relative = path.relative_to(source).as_posix()
        stability = path.relative_to(root).parts[0]
        ordinal = 0
        for line_number, line in enumerate(
            path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1
        ):
            match = re.match(r"^What:\s*(.*)$", line)
            if not match:
                continue
            ordinal += 1
            entries.append(
                {
                    "ordinal": ordinal,
                    "source": relative,
                    "source_line": line_number,
                    "stability": stability,
                    "what": match.group(1).strip(),
                }
            )
    return {
        "baseline": BASELINE_RELEASE,
        "entries": entries,
        "schema": SCHEMA,
        "source": "Documentation/ABI",
    }


def vdso_inventory(source: Path) -> dict[str, Any]:
    script = source / "arch/x86/entry/vdso/vdso64/vdso64.lds.S"
    conditions: list[str] = []
    in_global = False
    entries = []
    directives = re.compile(r"^#(if|ifdef|ifndef|elif|else|endif)\b\s*(.*)$")
    symbol = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*;$")
    for line_number, raw in enumerate(
        script.read_text(encoding="utf-8").splitlines(), start=1
    ):
        line = raw.strip()
        directive = directives.match(line)
        if directive:
            kind, expression = directive.groups()
            if kind in {"if", "ifdef", "ifndef"}:
                conditions.append(f"{kind} {expression}".strip())
            elif kind == "elif":
                if not conditions:
                    raise InventoryError("vDSO elif without matching condition")
                conditions[-1] = f"elif {expression}".strip()
            elif kind == "else":
                if not conditions:
                    raise InventoryError("vDSO else without matching condition")
                conditions[-1] = f"else ({conditions[-1]})"
            elif kind == "endif":
                if not conditions:
                    raise InventoryError("vDSO endif without matching condition")
                conditions.pop()
            continue
        if line == "global:":
            in_global = True
            continue
        if line.startswith("local:"):
            in_global = False
        match = symbol.match(line) if in_global else None
        if match:
            entries.append(
                {
                    "conditions": list(conditions),
                    "name": match.group(1),
                    "source_line": line_number,
                    "version": "LINUX_2.6",
                }
            )
    return {
        "architecture": "x86_64",
        "baseline": BASELINE_RELEASE,
        "entries": entries,
        "schema": SCHEMA,
        "source": "arch/x86/entry/vdso/vdso64/vdso64.lds.S",
    }


def logical_lines(text: str) -> Iterable[tuple[int, str]]:
    pending = ""
    first = 0
    for number, physical in enumerate(text.splitlines(), start=1):
        if not pending:
            first = number
        stripped = physical.rstrip()
        pending += stripped[:-1] if stripped.endswith("\\") else stripped
        if stripped.endswith("\\"):
            continue
        yield first, pending
        pending = ""
    if pending:
        yield first, pending


def ioctl_inventory(headers: Path) -> dict[str, Any]:
    include = headers / "include"
    pattern = re.compile(
        r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)[^\n]*?"
        r"\b(_IO|_IOR|_IOW|_IOWR)\s*\("
    )
    entries = []
    for path in sorted(include.rglob("*.h")):
        relative = path.relative_to(include).as_posix()
        for line_number, line in logical_lines(
            path.read_text(encoding="utf-8", errors="replace")
        ):
            match = pattern.match(line)
            if not match:
                continue
            name, family = match.groups()
            entries.append(
                {
                    "definition_sha256": sha256_bytes(line.encode("utf-8")),
                    "family": family,
                    "name": name,
                    "source": relative,
                    "source_line": line_number,
                }
            )
    return {
        "architecture": "x86_64",
        "baseline": BASELINE_RELEASE,
        "coverage_note": (
            "Indexes direct installed-header macro definitions using _IO families; "
            "aliases and computed or nonstandard commands require semantic follow-up."
        ),
        "entries": entries,
        "schema": SCHEMA,
        "source": "installed UAPI headers",
    }


def devicetree_inventory(source: Path) -> dict[str, Any]:
    root = source / "Documentation/devicetree/bindings"
    files = [
        path
        for path in root.rglob("*")
        if path.is_file() and path.suffix in {".yaml", ".txt"}
    ]
    entries = relative_file_records(source, files)
    return {
        "baseline": BASELINE_RELEASE,
        "coverage_note": (
            "Bindings are cross-architecture design input; their presence does not "
            "make Device Tree the initial x86 board-discovery mechanism."
        ),
        "entries": entries,
        "schema": SCHEMA,
        "source": "Documentation/devicetree/bindings",
    }


def documents(source: Path, headers: Path) -> dict[str, Any]:
    result = {
        "syscalls.json": syscall_inventory(source),
        "installed-uapi-headers.json": installed_headers_inventory(headers),
        "abi-documents.json": abi_document_inventory(source),
        "vdso-symbols.json": vdso_inventory(source),
        "ioctl-definitions.json": ioctl_inventory(headers),
        "devicetree-bindings.json": devicetree_inventory(source),
    }
    summaries = {}
    for name, document in result.items():
        entries = document["entries"]
        summaries[name] = {
            "count": len(entries),
            "entries_sha256": aggregate_digest(entries),
        }
    result["summary.json"] = {
        "architecture": "x86_64",
        "archive_sha256": BASELINE_ARCHIVE_SHA256,
        "baseline": BASELINE_RELEASE,
        "inventories": summaries,
        "schema": SCHEMA,
    }
    return result


def write_documents(output: Path, generated: dict[str, Any]) -> None:
    output.mkdir(parents=True, exist_ok=True)
    for name in OUTPUT_NAMES:
        destination = output / name
        with tempfile.NamedTemporaryFile(dir=output, delete=False) as temporary:
            temporary.write(inventory_bytes(generated[name]))
            temporary_path = Path(temporary.name)
        temporary_path.replace(destination)


def check_documents(output: Path, generated: dict[str, Any]) -> list[str]:
    differences = []
    for name in OUTPUT_NAMES:
        destination = output / name
        if not destination.is_file():
            differences.append(f"missing {destination}")
        elif destination.read_bytes() != inventory_bytes(generated[name]):
            differences.append(f"out of date {destination}")
    return differences


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--headers", type=Path, required=True)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        verify_inputs(arguments.source, arguments.headers, arguments.archive)
        generated = documents(arguments.source, arguments.headers)
        if arguments.check:
            differences = check_documents(arguments.output, generated)
            if differences:
                for difference in differences:
                    print(difference, file=sys.stderr)
                return 1
            print("Linux interface inventory is current")
            return 0
        write_documents(arguments.output, generated)
        print(f"wrote Linux interface inventory to {arguments.output}")
        return 0
    except (InventoryError, OSError, ValueError) as error:
        print(f"inventory error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
