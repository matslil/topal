"""Verify TK277 provenance equality, accounting for existing per-build tool headers."""

from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


if not __debug__:
    raise RuntimeError("Run this qualification without Python optimization")


def digest(value):
    return hashlib.sha256(value).hexdigest()


def compare(baseline: Path, fresh: Path):
    directories = (baseline, fresh)
    records = [json.loads((path / "provenance.json").read_text()) for path in directories]
    normalized = deepcopy(records)
    evidence = []
    work_roots = []
    for directory in directories:
        header = (directory / "time-library.retained.ll").read_text().splitlines()[0]
        match = re.fullmatch(r"; ModuleID = '([^']+/work)/time-library\.bc'", header)
        assert match, header
        work_roots.append(match[1])

    # Compare every published byte. The four llvm-dis headers are the only
    # permitted textual variation; even their module body must match exactly.
    outputs = [{item["identity"]: item["sha256"] for item in record["outputs"]}
               for record in records]
    assert outputs[0].keys() == outputs[1].keys()
    for name in outputs[0]:
        values = [(directory / name).read_bytes() for directory in directories]
        for index, value in enumerate(values):
            assert digest(value) == outputs[index][name], name
        if name in {stem + ".retained.ll" for stem in
                    ("time-library", "image-library", "dispatch-library", "process-identity-library")}:
            stem = name.removesuffix(".retained.ll")
            headers = []
            bodies = []
            for index, value in enumerate(values):
                header, body = value.split(b"\n", 1)
                assert header.decode() == f"; ModuleID = '{work_roots[index]}/{stem}.bc'"
                headers.append(header.decode())
                bodies.append(body)
            assert bodies[0] == bodies[1], name
            common = digest(b"; ModuleID = '<per-build-bitcode-input>'\n" + bodies[0])
            slot = stem.replace("-", "_")
            for index in range(2):
                assert records[index][slot]["retained_llvm_sha256"] == outputs[index][name]
                normalized[index][slot]["retained_llvm_sha256"] = common
                entry = next(item for item in normalized[index]["outputs"] if item["identity"] == name)
                entry["sha256"] = common
            evidence.append({"output": name, "verified_headers": headers,
                             "raw_sha256": [item[name] for item in outputs],
                             "common_normalized_sha256": common})
        else:
            assert values[0] == values[1], name

    sysroot = subprocess.run(["rustc", "--print", "target-libdir"], check=True,
                             capture_output=True, text=True).stdout.strip()
    objdump = Path(sysroot).parent / "bin" / "llvm-objdump"
    for stem in ("kernel-library", "time-library", "image-library",
                 "dispatch-library", "process-identity-library"):
        slot = stem.replace("-", "_")
        object_path = fresh / (stem + ".o")
        value = subprocess.run([str(objdump), "-d", "--no-show-raw-insn", str(object_path)],
                               check=True, capture_output=True).stdout
        prefix = b"\n" + str(object_path).encode() + b":\tfile format elf64-x86-64\n"
        assert value.startswith(prefix), value[:250]
        suffix = value[len(prefix):]
        raw = []
        # Reconstruct each original tool header on the identical object bytes,
        # and verify its recorded hash before normalizing that one header.
        for index in range(2):
            header = b"\n" + (work_roots[index] + "/" + stem + ".o").encode()
            header += b":\tfile format elf64-x86-64\n"
            original = digest(header + suffix)
            assert records[index][slot]["disassembly_sha256"] == original, (slot, index)
            raw.append(original)
            normalized[index][slot]["disassembly_sha256"] = digest(b"<object-header>\n" + suffix)
        evidence.append({"slot": slot, "verified_original_work_roots": work_roots,
                         "raw_disassembly_sha256": raw,
                         "common_normalized_sha256": normalized[0][slot]["disassembly_sha256"]})

    assert normalized[0] == normalized[1], "provenance differs beyond verified tool headers"
    return {"baseline_provenance_sha256": digest((baseline / "provenance.json").read_bytes()),
            "fresh_provenance_sha256": digest((fresh / "provenance.json").read_bytes()),
            "common_normalized_provenance_sha256": digest(json.dumps(normalized[0], sort_keys=True).encode()),
            "all_other_provenance_fields_identical": True,
            "verified_per_build_headers": evidence}


def refusal_controls(baseline: Path, fresh: Path):
    refused = []
    for fault in ("resource-fact", "retained-module-body", "object-bytes", "disassembly-hash"):
        with tempfile.TemporaryDirectory(prefix="topal-modular-refusal-") as directory:
            copied = Path(directory) / "baseline"
            shutil.copytree(baseline, copied)
            provenance = copied / "provenance.json"
            record = json.loads(provenance.read_text())
            if fault == "resource-fact":
                record["bootstrap_storage_capacity"] += 1
            elif fault == "disassembly-hash":
                record["kernel_library"]["disassembly_sha256"] = "0" * 64
            else:
                name = "time-library.retained.ll" if fault == "retained-module-body" else "time-library.o"
                path = copied / name
                value = bytearray(path.read_bytes())
                if fault == "retained-module-body":
                    value.extend(b"\n; controlled non-header difference\n")
                    field = "retained_llvm_sha256"
                else:
                    value[64] ^= 1
                    field = "object_sha256"
                path.write_bytes(value)
                changed = digest(value)
                record["time_library"][field] = changed
                next(item for item in record["outputs"] if item["identity"] == name)["sha256"] = changed
            provenance.write_text(json.dumps(record, indent=2) + "\n")
            try:
                compare(copied, fresh)
            except AssertionError:
                refused.append(fault)
            else:
                raise AssertionError(f"provenance comparison admitted {fault}")
    return refused
