#!/usr/bin/env python3
"""Build and execute the initial Topal x86-64 kernel toolchain gate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import pty
import shutil
import struct
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
SERIAL_INPUT = b"read"
MARKERS = (
    b"TOPAL_KERNEL_FRAME_ALLOCATED",
    b"TOPAL_KERNEL_FRAME_MAPPED",
    b"TOPAL_KERNEL_TRANSLATION_ACTIVE",
    b"TOPAL_KERNEL_TRANSLATION_EDITED",
    b"TOPAL_KERNEL_FRAME_EXTENT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_FRAME_EXTENT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK",
    b"TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK",
    b"TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PROCESS_ENTRY_RESTORED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK",
    b"TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK",
    b"TOPAL_KERNEL_INTERRUPTS_MASKED",
    b"TOPAL_KERNEL_MEMORY_DESCRIBED",
    b"TOPAL_KERNEL_PCI_VIRTIO_RNG_OK",
    b"TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK",
    b"TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED",
    b"TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED",
    b"TOPAL_KERNEL_PCI_BUS_MASTER_OK",
    b"TOPAL_KERNEL_DMA_BUFFERS_OK",
    b"TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK",
    b"TOPAL_KERNEL_DMA_DEVICE_OWNED",
    b"TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK",
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
    b"TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE",
    b"TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED",
    b"TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL",
    b"TOPAL_KERNEL_VIRTIO_USED_RING_OK",
    b"TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED",
    b"TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED",
    b"TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED",
    b"TOPAL_KERNEL_ENTROPY_POOL_CREATED",
    b"TOPAL_KERNEL_ENTROPY_POOL_MIXED",
    b"TOPAL_KERNEL_ENTROPY_POOL_CREDITED",
    b"TOPAL_KERNEL_ENTROPY_GENERATOR_READY",
    b"TOPAL_KERNEL_ENTROPY_OUTPUT_READY",
    b"TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK",
    b"TOPAL_KERNEL_USER_WRITE_OK",
    b"TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK",
    b"TOPAL_KERNEL_USER_GETRANDOM_OK",
    b"TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK",
    b"TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK",
    b"TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK",
    b"TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK",
    b"TOPAL_KERNEL_USER_READ_OK",
    b"TOPAL_KERNEL_USER_CLOSE_OK",
    b"TOPAL_KERNEL_USER_DUP_OK",
    b"TOPAL_KERNEL_USER_FCNTL_OK",
    b"TOPAL_KERNEL_USER_FSTAT_OK",
    b"TOPAL_KERNEL_USER_TCGETS_OK",
    b"TOPAL_KERNEL_USER_WINSIZE_OK",
    b"TOPAL_KERNEL_USER_LSEEK_OK",
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
    linked: Path,
    initial_user_image: Path | None = None,
    static_user_image: Path | None = None,
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
    if static_user_image is not None:
        compiler_command.extend(["--static-user-image", str(static_user_image)])
    compiler_command.extend(
        [
        "-o",
        str(linked),
        str(SOURCE_PATH),
        ]
    )
    print("+", " ".join(compiler_command), flush=True)
    subprocess.run(compiler_command, cwd=REPOSITORY, check=True)


def static_user_fixture() -> bytes:
    """Create approved ELF metadata with clean-room payload bytes."""
    image = bytearray(2_024_544)
    image[:16] = bytes(
        [0x7F, ord("E"), ord("L"), ord("F"), 2, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0]
    )
    struct.pack_into(
        "<HHIQQQIHHHHHH",
        image,
        16,
        2,
        62,
        1,
        0x004A_6580,
        64,
        0,
        0,
        64,
        56,
        11,
        0,
        0,
        0,
    )
    headers = (
        (1, 4, 0, 0x0040_0000, 0x6E0, 0x6E0, 0x1000),
        (1, 5, 0x1000, 0x0040_1000, 0x18_B285, 0x18_B285, 0x1000),
        (1, 4, 0x18_D000, 0x0058_D000, 0x5_7040, 0x5_7040, 0x1000),
        (1, 6, 0x1E_4660, 0x005E_5660, 0x9728, 0x1_0B68, 0x1000),
        (4, 4, 0x2A8, 0x0040_02A8, 0x20, 0x20, 8),
        (4, 4, 0x2C8, 0x0040_02C8, 0x24, 0x24, 4),
        (4, 4, 0x1E_4020, 0x005E_4020, 0x20, 0x20, 4),
        (7, 4, 0x1E_4660, 0x005E_5660, 0x40, 0x82, 8),
        (0x6474_E553, 4, 0x2A8, 0x0040_02A8, 0x20, 0x20, 8),
        (0x6474_E551, 6, 0, 0, 0, 0, 0x10),
        (0x6474_E552, 4, 0x1E_4660, 0x005E_5660, 0x79A0, 0x79A0, 1),
    )
    for index, (kind, flags, offset, virtual, file_size, memory_size, alignment) in enumerate(headers):
        struct.pack_into(
            "<IIQQQQQQ",
            image,
            64 + index * 56,
            kind,
            flags,
            offset,
            virtual,
            virtual,
            file_size,
            memory_size,
            alignment,
        )
    image[0x1000] = 0xA5
    image[0xA_6580 : 0xA_6580 + 66] = bytes(
        [
            0xB8, 0x0C, 0, 0, 0,
            0x31, 0xFF,
            0x0F, 0x05,
            0x48, 0x3D, 0x00, 0x70, 0x5F, 0x00,
            0x75, 0x2F,
            0xBF, 0xC0, 0x7D, 0x5F, 0x00,
            0xB8, 0x0C, 0, 0, 0,
            0x0F, 0x05,
            0x48, 0x3D, 0xC0, 0x7D, 0x5F, 0x00,
            0x75, 0x1B,
            0xC6, 0x04, 0x25, 0x00, 0x70, 0x5F, 0x00, 0x5A,
            0x80, 0x3C, 0x25, 0x00, 0x70, 0x5F, 0x00, 0x5A,
            0x75, 0x09,
            0x31, 0xFF,
            0xB8, 0xE7, 0, 0, 0,
            0x0F, 0x05,
            0x0F, 0x0B,
        ]
    )
    return bytes(image)


def build_image(destination: Path) -> tuple[Path, Path, Path, Path, Path, Path]:
    generated = destination / "generated"
    compile_systems_artifact(generated)
    generated_initial_user_image = generated / "initial-user.elf"
    supplied_initial_user_image = destination / "supplied-initial-user.elf"
    supplied = bytearray(generated_initial_user_image.read_bytes())
    supplied[4097] = 0xA5
    supplied_initial_user_image.write_bytes(supplied)
    supplied_static_user_image = destination / "supplied-static-user.elf"
    supplied_static_user_image.write_bytes(static_user_fixture())

    linked = destination / "linked"
    compile_systems_artifact(
        linked, supplied_initial_user_image, supplied_static_user_image
    )
    published_initial_user_image = linked / "initial-user.elf"
    if published_initial_user_image.read_bytes() != supplied:
        raise GateError("compiler did not publish the exact supplied initial user image")
    if digest(published_initial_user_image) == digest(generated_initial_user_image):
        raise GateError("supplied initial user image is not distinct from the generated default")
    published_static_user_image = linked / "static-user.elf"
    if published_static_user_image.read_bytes() != supplied_static_user_image.read_bytes():
        raise GateError("compiler did not publish the exact supplied static user image")

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
    artifact = json.loads(artifact_provenance.read_text(encoding="utf-8"))
    static_outputs = [
        output
        for output in artifact["outputs"]
        if output["identity"] == "static-user.elf"
    ]
    if len(static_outputs) != 1 or static_outputs[0]["sha256"] != digest(
        published_static_user_image
    ):
        raise GateError("artifact provenance does not bind the static user image")
    return (
        image,
        boot_resource,
        boot_provenance,
        artifact_provenance,
        generated_initial_user_image,
        published_static_user_image,
    )


def qemu_command(
    image: Path, boot_resource: Path, serial: Path, serial_tty: str | None = None
) -> list[str]:
    data = manifest()
    emulator = data["emulator"]
    machine = data["machine"]
    firmware = data["firmware"]
    entropy = data["devices"]["entropy"]
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
        (
            f"serial,id=serial0,path={serial_tty}"
            if serial_tty is not None
            else f"socket,id=serial0,path={serial.with_suffix('.sock')},server=on,wait=on"
        ),
        "-device",
        "isa-serial,chardev=serial0,index=0",
        "-object",
        f"rng-random,id=entropy0,filename={entropy['backend']}",
        "-device",
        (
            f"{entropy['model']},rng=entropy0,bus=pcie.0,addr=05.0,"
            f"max-bytes={entropy['max_bytes']},period={entropy['period_ms']}"
        ),
        "-kernel",
        str(image),
        "-initrd",
        str(boot_resource),
    ]


def observe_gate(
    image: Path, boot_resource: Path, serial: Path, qemu_log: Path, timeout: float
) -> bytes:
    master_fd, slave_fd = pty.openpty()
    os.set_blocking(master_fd, False)
    command = qemu_command(image, boot_resource, serial, os.ttyname(slave_fd))
    print("+", " ".join(map(str, command)), flush=True)
    injected = False
    received = bytearray()

    def drain_client() -> None:
        while True:
            try:
                block = os.read(master_fd, 4096)
            except (BlockingIOError, OSError):
                break
            if not block:
                break
            received.extend(block)
        serial.write_bytes(received)

    with qemu_log.open("wb") as output:
        process = subprocess.Popen(
            command,
            stdout=output,
            stderr=subprocess.STDOUT,
        )
        os.close(slave_fd)
        try:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                status = process.poll()
                if status is not None:
                    raise GateError(f"QEMU exited with status {status} before the gate completed")
                drain_client()
                observed = bytes(received)
                if (
                    not injected
                    and b"TOPAL_KERNEL_BOOT_RESOURCE_OK" in observed
                ):
                    os.write(master_fd, SERIAL_INPUT)
                    injected = True
                positions = [observed.find(marker) for marker in MARKERS]
                if positions[0] >= 0 and all(
                    later > earlier for earlier, later in zip(positions, positions[1:])
                ):
                    time.sleep(0.25)
                    drain_client()
                    settled = bytes(received)
                    time.sleep(1.0)
                    if process.poll() is not None:
                        raise GateError("QEMU exited instead of remaining in the fatal halt")
                    drain_client()
                    if bytes(received) != settled:
                        raise GateError("serial output continued after the fatal disposition")
                    return settled
                time.sleep(0.05)
            raise GateError(f"timed out after {timeout:g}s waiting for all kernel markers")
        finally:
            os.close(master_fd)
            try:
                os.close(slave_fd)
            except OSError:
                pass
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
    static_user_image: Path,
    serial: bytes,
) -> dict[str, Any]:
    data = manifest()
    boot = json.loads(boot_provenance.read_text(encoding="utf-8"))
    artifact = json.loads(artifact_provenance.read_text(encoding="utf-8"))
    if boot["static_user_image_sha256"] != digest(static_user_image):
        raise GateError("boot provenance does not bind the static user image")
    return {
        "schema": "topal-kernel-toolchain-gate-qemu/161",
        "publication": "topalc-target-interface/1",
        "source_sha256": digest(SOURCE_PATH),
        "manifest_sha256": digest(MANIFEST_PATH),
        "qemu": qemu_version,
        "firmware_sha256": digest(Path(data["firmware"]["path"])),
        "machine": data["emulator"]["machine"],
        "cpu": data["emulator"]["cpu"],
        "serial_input_utf8": SERIAL_INPUT.decode("ascii"),
        "serial_input_sha256": hashlib.sha256(SERIAL_INPUT).hexdigest(),
        "provider": boot["provider"],
        "artifact_schema": artifact["schema"],
        "boot_schema": boot["schema"],
        "boot_resource_schema": boot["boot_resource_schema"],
        "boot_resource_member": boot["boot_resource_member"],
        "static_boot_resource_member": boot["static_boot_resource_member"],
        "boot_resource_bytes": boot["boot_resource_bytes"],
        "initial_user_image_bytes": boot["initial_user_image_bytes"],
        "initial_user_image_sha256": boot["initial_user_image_sha256"],
        "generated_initial_user_image_sha256": digest(generated_initial_user_image),
        "initial_user_image_input": "supplied",
        "static_user_image_bytes": boot["static_user_image_bytes"],
        "static_user_image_sha256": boot["static_user_image_sha256"],
        "static_user_image_input": "supplied-clean-room-fixture",
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
            static_user_image,
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
            static_user_image,
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
