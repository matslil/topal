#!/usr/bin/env python3
"""Build and execute the initial Topal x86-64 kernel toolchain gate."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any


LAB = Path(__file__).resolve().parent
REPOSITORY = LAB.parents[3]
LINUX_KERNEL = REPOSITORY / "linux-kernel"
BUILD_ROOT = LINUX_KERNEL / "build" / "x86_64" / "topal-toolchain-gate"
MANIFEST_PATH = LAB / "manifest.json"
SOURCE_PATH = LINUX_KERNEL / "kernel" / "arch" / "x86_64" / "toolchain-gate.t"
EVIDENCE_PATH = LAB / "results" / "topal-toolchain-gate.json"
MARKERS = (
    b"TOPAL_KERNEL_FRAME_ALLOCATED",
    b"TOPAL_KERNEL_FRAME_MAPPED",
    b"TOPAL_KERNEL_TRANSLATION_ACTIVE",
    b"TOPAL_KERNEL_TRANSLATION_EDITED",
    b"TOPAL_KERNEL_FRAME_EXTENT_OK",
    b"TOPAL_KERNEL_INTERRUPTS_MASKED",
    b"TOPAL_KERNEL_MEMORY_DESCRIBED",
    b"TOPAL_KERNEL_BOOT",
    b"TOPAL_KERNEL_FAULT_RESUMED",
    b"TOPAL_KERNEL_ATOMIC_OK",
    b"TOPAL_KERNEL_MEMORY_OK",
    b"TOPAL_KERNEL_INTERRUPT_OK",
    b"TOPAL_KERNEL_TIME_OK",
    b"TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED",
    b"TOPAL_KERNEL_DEADLINE_PREEMPTED",
    b"TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED",
    b"TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED",
    b"TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED",
    b"TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED",
    b"TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED",
    b"TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED",
    b"TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED",
    b"TOPAL_KERNEL_USER_IMAGE_READY",
    b"TOPAL_KERNEL_BOOT_RESOURCE_OK",
    b"TOPAL_KERNEL_USER_COPY_TO_OK",
    b"TOPAL_KERNEL_USER_COPY_FROM_OK",
    b"TOPAL_KERNEL_USER_SYSCALL_OK",
    b"TOPAL_KERNEL_USER_IMAGE_ENTRY_OK",
    b"TOPAL_KERNEL_USER_IMAGE_STACK_OK",
    b"TOPAL_KERNEL_USER_IMAGE_AUXV_OK",
    b"TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK",
    b"TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK",
    b"TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK",
    b"TOPAL_KERNEL_USER_IMAGE_ELF_OK",
    b"TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK",
    b"TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK",
    b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK",
    b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK",
    b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK",
    b"TOPAL_KERNEL_USER_IMAGE_BSS_OK",
    b"TOPAL_KERNEL_USER_IMAGE_DATA_OK",
    b"TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK",
    b"TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK",
    b"TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK",
    b"TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK",
    b"TOPAL_KERNEL_USER_IMAGE_RETAINED",
    b"TOPAL_KERNEL_USER_MAPPING_OK",
    b"TOPAL_KERNEL_FRAME_DISJOINT_OK",
    b"TOPAL_KERNEL_USER_BACKING_OK",
)


class GateError(Exception):
    """The Topal kernel toolchain gate could not be reproduced."""


def manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def require_program(name: str) -> str:
    path = shutil.which(name)
    if path is None:
        raise GateError(f"required host program is unavailable: {name}")
    return path


def verify_host() -> str:
    data = manifest()
    qemu = require_program(data["emulator"]["program"])
    version = subprocess.run(
        [qemu, "--version"], check=True, text=True, capture_output=True
    ).stdout.splitlines()[0]
    expected_version = data["emulator"]["version"]
    if f"version {expected_version}" not in version:
        raise GateError(f"QEMU version mismatch: {version!r}; expected {expected_version}")
    firmware = Path(data["firmware"]["path"])
    if not firmware.is_file() or digest(firmware) != data["firmware"]["sha256"]:
        raise GateError(f"firmware does not match the pinned manifest: {firmware}")
    return version


def compile_systems_artifact(
    linked: Path, initial_user_image: Path | None = None
) -> None:
    compiler_command = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "topal-compiler",
        "--bin",
        "topalc",
        "--",
        "--target",
        "x86_64-unknown-none",
        "--cpu",
        "generic",
        "--board",
        "topal-qemu-pc-q35-10.2",
        "--emit",
        "executable",
    ]
    if initial_user_image is not None:
        compiler_command.extend(["--initial-user-image", str(initial_user_image)])
    compiler_command.extend(
        [
        "-o",
        str(linked),
        str(SOURCE_PATH),
        ]
    )
    print("+", " ".join(compiler_command), flush=True)
    subprocess.run(compiler_command, cwd=REPOSITORY, check=True)


def build_image(destination: Path) -> tuple[Path, Path, Path, Path, Path]:
    generated = destination / "generated"
    compile_systems_artifact(generated)
    generated_initial_user_image = generated / "initial-user.elf"
    supplied_initial_user_image = destination / "supplied-initial-user.elf"
    supplied = bytearray(generated_initial_user_image.read_bytes())
    supplied[4097] = 0xA5
    supplied_initial_user_image.write_bytes(supplied)

    linked = destination / "linked"
    compile_systems_artifact(linked, supplied_initial_user_image)
    published_initial_user_image = linked / "initial-user.elf"
    if published_initial_user_image.read_bytes() != supplied:
        raise GateError("compiler did not publish the exact supplied initial user image")
    if digest(published_initial_user_image) == digest(generated_initial_user_image):
        raise GateError("supplied initial user image is not distinct from the generated default")

    boot = destination / "boot"
    packaging_command = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "topal-kernel-toolchain-gate-builder",
        "--",
        str(linked),
        str(boot),
    ]
    print("+", " ".join(packaging_command), flush=True)
    subprocess.run(packaging_command, cwd=REPOSITORY, check=True)
    image = boot / "bzImage"
    boot_resource = boot / "initramfs.cpio"
    boot_provenance = boot / "boot-provenance.json"
    artifact_provenance = linked / "provenance.json"
    for path in (image, boot_resource, boot_provenance, artifact_provenance):
        if not path.is_file():
            raise GateError(f"builder omitted required output: {path}")
    return (
        image,
        boot_resource,
        boot_provenance,
        artifact_provenance,
        generated_initial_user_image,
    )


def qemu_command(image: Path, boot_resource: Path, serial: Path) -> list[str]:
    data = manifest()
    emulator = data["emulator"]
    machine = data["machine"]
    firmware = data["firmware"]
    return [
        emulator["program"],
        "-name",
        "topal-toolchain-gate,process=topal-toolchain-gate",
        "-nodefaults",
        "-no-reboot",
        "-display",
        "none",
        "-monitor",
        "none",
        "-machine",
        f"{emulator['machine']},smm={'on' if machine['smm'] else 'off'},"
        f"vmport={'on' if machine['vmport'] else 'off'},"
        f"usb={'on' if machine['usb'] else 'off'},"
        f"sata={'on' if machine['sata'] else 'off'},hpet=on,dump-guest-core=off",
        "-accel",
        f"tcg,thread={emulator['tcg_thread']}",
        "-cpu",
        emulator["cpu"],
        "-smp",
        f"cpus={emulator['vcpus']},sockets={emulator['topology']['sockets']},"
        f"cores={emulator['topology']['cores']},threads={emulator['topology']['threads']}",
        "-m",
        f"{emulator['memory_mib']}M",
        "-uuid",
        emulator["uuid"],
        "-bios",
        firmware["path"],
        "-rtc",
        "base=utc,clock=vm,driftfix=slew",
        "-global",
        "ICH9-LPC.disable_s3=1",
        "-global",
        "ICH9-LPC.disable_s4=1",
        "-chardev",
        f"file,id=serial0,path={serial}",
        "-device",
        "isa-serial,chardev=serial0,index=0",
        "-kernel",
        str(image),
        "-initrd",
        str(boot_resource),
    ]


def observe_gate(
    image: Path, boot_resource: Path, serial: Path, qemu_log: Path, timeout: float
) -> bytes:
    command = qemu_command(image, boot_resource, serial)
    print("+", " ".join(map(str, command)), flush=True)
    with qemu_log.open("wb") as output:
        process = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                status = process.poll()
                if status is not None:
                    raise GateError(f"QEMU exited with status {status} before the gate completed")
                observed = serial.read_bytes() if serial.exists() else b""
                positions = [observed.find(marker) for marker in MARKERS]
                if positions[0] >= 0 and all(
                    later > earlier for earlier, later in zip(positions, positions[1:])
                ):
                    time.sleep(0.25)
                    settled = serial.read_bytes()
                    time.sleep(1.0)
                    if process.poll() is not None:
                        raise GateError("QEMU exited instead of remaining in the fatal halt")
                    if serial.read_bytes() != settled:
                        raise GateError("serial output continued after the fatal disposition")
                    return settled
                time.sleep(0.05)
            raise GateError(f"timed out after {timeout:g}s waiting for all kernel markers")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)


def evidence_record(
    qemu_version: str,
    image: Path,
    boot_resource: Path,
    boot_provenance: Path,
    artifact_provenance: Path,
    generated_initial_user_image: Path,
    serial: bytes,
) -> dict[str, Any]:
    data = manifest()
    boot = json.loads(boot_provenance.read_text(encoding="utf-8"))
    artifact = json.loads(artifact_provenance.read_text(encoding="utf-8"))
    return {
        "schema": "topal-kernel-toolchain-gate-qemu/54",
        "publication": "topalc-target-interface/1",
        "source_sha256": digest(SOURCE_PATH),
        "manifest_sha256": digest(MANIFEST_PATH),
        "qemu": qemu_version,
        "firmware_sha256": digest(Path(data["firmware"]["path"])),
        "machine": data["emulator"]["machine"],
        "cpu": data["emulator"]["cpu"],
        "provider": boot["provider"],
        "artifact_schema": artifact["schema"],
        "boot_schema": boot["schema"],
        "boot_resource_schema": boot["boot_resource_schema"],
        "boot_resource_member": boot["boot_resource_member"],
        "boot_resource_bytes": boot["boot_resource_bytes"],
        "initial_user_image_bytes": boot["initial_user_image_bytes"],
        "initial_user_image_sha256": boot["initial_user_image_sha256"],
        "generated_initial_user_image_sha256": digest(generated_initial_user_image),
        "initial_user_image_input": "supplied",
        "linked_kernel_sha256": boot["linked_kernel_sha256"],
        "artifact_provenance_sha256": digest(artifact_provenance),
        "boot_image_sha256": digest(image),
        "boot_resource_sha256": digest(boot_resource),
        "boot_provenance_sha256": digest(boot_provenance),
        "observations": {
            "markers_in_order": [marker.decode("ascii") for marker in MARKERS],
            "serial_utf8": serial.decode("utf-8"),
            "serial_sha256": hashlib.sha256(serial).hexdigest(),
            "qemu_running_after_markers": True,
            "serial_quiescent_after_fatal_disposition": True,
            "result": "pass",
        },
    }


def verify(timeout: float, write_evidence: bool) -> dict[str, Any]:
    qemu_version = verify_host()
    BUILD_ROOT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="run-", dir=BUILD_ROOT) as temporary:
        root = Path(temporary)
        (
            image,
            boot_resource,
            boot_provenance,
            artifact_provenance,
            generated_initial_user_image,
        ) = build_image(root / "artifact")
        boot = json.loads(boot_provenance.read_text(encoding="utf-8"))
        if digest(boot_resource) != boot["boot_resource_sha256"]:
            raise GateError("published boot resource does not match boot provenance")
        serial_path = root / "serial.log"
        qemu_log = root / "qemu.log"
        try:
            serial = observe_gate(image, boot_resource, serial_path, qemu_log, timeout)
        except GateError as error:
            serial_text = (
                serial_path.read_text(encoding="utf-8", errors="replace")
                if serial_path.exists()
                else "serial log was not created"
            )
            qemu_text = (
                qemu_log.read_text(encoding="utf-8", errors="replace")
                if qemu_log.exists()
                else "QEMU log was not created"
            )
            raise GateError(
                f"{error}\nserial output:\n{serial_text}\nQEMU output:\n{qemu_text}"
            ) from error
        record = evidence_record(
            qemu_version,
            image,
            boot_resource,
            boot_provenance,
            artifact_provenance,
            generated_initial_user_image,
            serial,
        )
    if write_evidence:
        EVIDENCE_PATH.parent.mkdir(parents=True, exist_ok=True)
        staged = EVIDENCE_PATH.with_suffix(".json.new")
        staged.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        staged.replace(EVIDENCE_PATH)
        print(f"wrote {EVIDENCE_PATH}")
    return record


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--write-evidence", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        result = verify(args.timeout, args.write_evidence)
        print(json.dumps(result, indent=2, sort_keys=True))
    except (GateError, OSError, subprocess.SubprocessError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
