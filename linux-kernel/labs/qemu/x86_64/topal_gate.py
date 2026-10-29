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
PINNED_STATIC_BUSYBOX_EVIDENCE_PATH = (
    LAB / "results" / "topal-pinned-static-busybox-gate.json"
)
PINNED_STATIC_BUSYBOX_SHA256 = (
    "7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd"
)
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
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_OK",
    b"TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_OK",
    b"TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_OK",
    b"TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_OK",
    b"TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_OK",
    b"TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_OK",
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
    image[0xA_6580 : 0xA_6580 + 882] = bytes(
        [
            0xB8, 0x0C, 0, 0, 0,
            0x31, 0xFF,
            0x0F, 0x05,
            0x48, 0x3D, 0x00, 0x70, 0x5F, 0x00,
            0x75, 0xFE,
            0xBF, 0x80, 0x7D, 0x5F, 0x00,
            0xB8, 0x0C, 0, 0, 0,
            0x0F, 0x05,
            0x48, 0x3D, 0x80, 0x7D, 0x5F, 0x00,
            0x75, 0xFE,
            0xC6, 0x04, 0x25, 0x00, 0x70, 0x5F, 0x00, 0x5A,
            0x80, 0x3C, 0x25, 0x00, 0x70, 0x5F, 0x00, 0x5A,
            0x75, 0xFE,
            0xC6, 0x04, 0x25, 0x00, 0x74, 0x5F, 0x00, 0x6B,
            0xBF, 0x02, 0x10, 0x00, 0x00,
            0xBE, 0x00, 0x74, 0x5F, 0x00,
            0xB8, 0x9E, 0, 0, 0,
            0x0F, 0x05,
            0x48, 0x85, 0xC0,
            0x75, 0xFE,
            0x64, 0x8A, 0x04, 0x25, 0, 0, 0, 0,
            0x3C, 0x6B,
            0x75, 0xFE,
            0xC7, 0x04, 0x25, 0xD0, 0x76, 0x5F, 0x00, 1, 0, 0, 0,
            0xBF, 0xD0, 0x76, 0x5F, 0x00,
            0xB8, 0xDA, 0, 0, 0,
            0x0F, 0x05,
            0x48, 0x83, 0xF8, 1,
            0x75, 0xFE,
            0x48, 0xC7, 0x04, 0x25, 0xE0, 0x76, 0x5F, 0x00, 0xE0, 0x76, 0x5F, 0x00,
            0x48, 0xC7, 0x04, 0x25, 0xE8, 0x76, 0x5F, 0x00, 0xE0, 0xFF, 0xFF, 0xFF,
            0x48, 0xC7, 0x04, 0x25, 0xF0, 0x76, 0x5F, 0x00, 0, 0, 0, 0,
            0xBF, 0xE0, 0x76, 0x5F, 0x00,
            0xBE, 0x18, 0, 0, 0,
            0xB8, 0x11, 0x01, 0, 0,
            0x0F, 0x05,
            0x48, 0x85, 0xC0,
            0x75, 0xFE,
            0xBF, 0x00, 0x73, 0x5F, 0x00,
            0xBE, 0x21, 0, 0, 0,
            0x31, 0xD2,
            0x41, 0xBA, 0x53, 0x30, 0x05, 0x53,
            0xB8, 0x4E, 0x01, 0, 0,
            0x0F, 0x05,
            0x48, 0x83, 0xF8, 0xDA,
            0x75, 0xFE,
            0x48, 0x81, 0xEC, 0x40, 0x01, 0, 0,
            0x31, 0xFF,
            0xBE, 0x03, 0, 0, 0,
            0x31, 0xD2,
            0x49, 0x89, 0xE2,
            0xB8, 0x2E, 0x01, 0, 0,
            0x0F, 0x05,
            0x48, 0x85, 0xC0,
            0x75, 0xFE,
            0x48, 0x81, 0x3C, 0x24, 0x00, 0x00, 0x80, 0x00,
            0x75, 0xFE,
            0x48, 0x83, 0x7C, 0x24, 0x08, 0xFF,
            0x75, 0xFE,
            0x48, 0x89, 0x04, 0x24,
            0x48, 0x89, 0x44, 0x24, 0x08,
            0x48, 0x81, 0xC4, 0x40, 0x01, 0, 0,
            0xBF, 0x9C, 0xFF, 0xFF, 0xFF,
            0xBE, 0xDA, 0xFD, 0x5B, 0x00,
            0xBA, 0x00, 0xE0, 0x5F, 0x00,
            0x41, 0xBA, 0x00, 0x10, 0, 0,
            0xB8, 0x0B, 0x01, 0, 0,
            0x0F, 0x05,
            0x48, 0x83, 0xF8, 0xDA,
            0x75, 0xFE,
            0xBF, 0x68, 0x5E, 0x5F, 0x00,
            0xBE, 0x08, 0, 0, 0,
            0xBA, 0x01, 0, 0, 0,
            0xB8, 0x3E, 0x01, 0, 0,
            0x0F, 0x05,
            0x48, 0x83, 0xF8, 0x08,
            0x75, 0xFE,
            0x4C, 0x8B, 0x14, 0x25, 0x68, 0x5E, 0x5F, 0x00,
            0x48, 0xC7, 0x04, 0x25, 0x68, 0x5E, 0x5F, 0x00, 0, 0, 0, 0,
            0xBF, 0x70, 0x5E, 0x5F, 0x00,
            0xBE, 0x08, 0, 0, 0,
            0xBA, 0x01, 0, 0, 0,
            0xB8, 0x3E, 0x01, 0, 0,
            0x0F, 0x05,
            0x48, 0x83, 0xF8, 0x08,
            0x75, 0xFE,
            0x4C, 0x8B, 0x14, 0x25, 0x70, 0x5E, 0x5F, 0x00,
            0x48, 0xC7, 0x04, 0x25, 0x70, 0x5E, 0x5F, 0x00, 0, 0, 0, 0,
            0xB8, 0x0C, 0, 0, 0,
            0x31, 0xFF,
            0x0F, 0x05,
            0x48, 0x3D, 0x80, 0x7D, 0x5F, 0x00,
            0x75, 0xFE,
            0xBF, 0x80, 0x8D, 0x61, 0x00,
            0xB8, 0x0C, 0, 0, 0,
            0x0F, 0x05,
            0x48, 0x3D, 0x80, 0x8D, 0x61, 0x00,
            0x75, 0xFE,
            0xC6, 0x04, 0x25, 0x7F, 0x8D, 0x61, 0x00, 0x6D,
            0x80, 0x3C, 0x25, 0x7F, 0x8D, 0x61, 0x00, 0x6D,
            0x75, 0xFE,
            0xBF, 0x00, 0x90, 0x61, 0x00,
            0xB8, 0x0C, 0, 0, 0,
            0x0F, 0x05,
            0x48, 0x3D, 0x00, 0x90, 0x61, 0x00,
            0x75, 0xFE,
            0xC6, 0x04, 0x25, 0xFF, 0x8F, 0x61, 0x00, 0x71,
            0x80, 0x3C, 0x25, 0xFF, 0x8F, 0x61, 0x00, 0x71,
            0x75, 0xFE,
            0xBF, 0x00, 0x50, 0x5E, 0x00,
            0xBE, 0x00, 0x80, 0x00, 0x00,
            0xBA, 0x01, 0x00, 0x00, 0x00,
            0xB8, 0x0A, 0x00, 0x00, 0x00,
            0x0F, 0x05,
            0x48, 0x85, 0xC0,
            0x75, 0xFE,
            0x48, 0x81, 0xEC, 0x40, 0x01, 0x00, 0x00,
            0xBF, 0x10, 0x00, 0x00, 0x00,
            0x48, 0x89, 0xE6,
            0xB8, 0x9D, 0x00, 0x00, 0x00,
            0x0F, 0x05,
            0x48, 0x85, 0xC0,
            0x75, 0xFE,
            0x48, 0xB8, 0x62, 0x75, 0x73, 0x79, 0x62, 0x6F, 0x78, 0x00,
            0x48, 0x39, 0x04, 0x24,
            0x75, 0xFE,
            0x48, 0x83, 0x7C, 0x24, 0x08, 0x00,
            0x75, 0xFE,
            0x48, 0xC7, 0x04, 0x24, 0, 0, 0, 0,
            0x48, 0xC7, 0x44, 0x24, 0x08, 0, 0, 0, 0,
            0x48, 0x81, 0xC4, 0x40, 0x01, 0x00, 0x00,
            0xB8, 0x66, 0x00, 0x00, 0x00,
            0x0F, 0x05,
            0x48, 0x85, 0xC0,
            0x75, 0xFE,
            0x48, 0x81, 0xEC, 0x20, 0x02, 0x00, 0x00,
            0xBF, 0x9C, 0xFF, 0xFF, 0xFF,
            0xBE, 0x00, 0x69, 0x4A, 0x00,
            0x48, 0x89, 0xE2,
            0x45, 0x31, 0xD2,
            0xB8, 0x06, 0x01, 0x00, 0x00,
            0x0F, 0x05,
            0x48, 0x83, 0xF8, 0xFE,
            0x75, 0xFE,
            0x48, 0x81, 0xC4, 0x20, 0x02, 0x00, 0x00,
            0xB8, 0x68, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
            0x31, 0xFF,
            0xB8, 0x6A, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
            0x31, 0xFF,
            0xB8, 0x69, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
            0x48, 0x81, 0xEC, 0x40, 0x02, 0x00, 0x00,
            0xC6, 0x04, 0x24, 0x0A,
            0xBF, 0x01, 0x00, 0x00, 0x00,
            0x48, 0x89, 0xE6,
            0xBA, 0x01, 0x00, 0x00, 0x00,
            0xB8, 0x01, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x83, 0xF8, 0x01, 0x75, 0xFE,
            0xC6, 0x04, 0x24, 0x00,
            0x48, 0x81, 0xC4, 0x40, 0x02, 0x00, 0x00,
            0xB8, 0x27, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x83, 0xF8, 0x01, 0x75, 0xFE,
            0x48, 0x81, 0xEC, 0x40, 0x02, 0x00, 0x00,
            0x48, 0xC7, 0x04, 0x24, 0x80, 0x65, 0x4A, 0x00,
            0x48, 0xC7, 0x44, 0x24, 0x08, 0x00, 0x00, 0x00, 0x04,
            0x48, 0xC7, 0x44, 0x24, 0x10, 0x80, 0x65, 0x4A, 0x00,
            0x48, 0xB8, 0xFF, 0xFF, 0xFF, 0x7F, 0xFE, 0xFF, 0xFF, 0xFF,
            0x48, 0x89, 0x44, 0x24, 0x18,
            0xBF, 0x11, 0x00, 0x00, 0x00,
            0x48, 0x89, 0xE6,
            0x31, 0xD2,
            0x41, 0xBA, 0x08, 0x00, 0x00, 0x00,
            0xB8, 0x0D, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
            0x31, 0xC0,
            0x48, 0x89, 0x04, 0x24,
            0x48, 0x89, 0x44, 0x24, 0x08,
            0x48, 0x89, 0x44, 0x24, 0x10,
            0x48, 0x89, 0x44, 0x24, 0x18,
            0x48, 0x81, 0xC4, 0x40, 0x02, 0x00, 0x00,
            0xB8, 0x6E, 0x00, 0x00, 0x00,
            0x0F, 0x05, 0x48, 0x85, 0xC0, 0x75, 0xFE,
            0xE9, 0x0E, 0x07, 0x00, 0x00,
        ]
    )
    continuation = bytes.fromhex(
        "4881ec000400004889e7b83f0000000f054885c075fe48b84c696e7578000000"
        "4839042475fe48b87838365f36340000483984240401000075fe48b8286e6f6e"
        "65290000483984244501000075fe31c04889e7b931000000fcf348abbf60945f"
        "00be00100000b84f0000000f054883f80275fe66813c2560945f002f0075fe48"
        "c7042560945f0000000000c604246d31ffbe015400004889e2b8100000000f05"
        "4883f8e775fe803c246d75fec604240048b8a5a5a5a5a5a5a5a54889e7b904"
        "000000f348abbf0200000031f64889e241ba08000000b80d0000000f054885c0"
        "75fe31c0b9040000004889e7fcf348af75fe48b8a5a5a5a5a5a5a5a54889e7"
        "b904000000f348abbf0300000031f64889e241ba08000000b80d0000000f0548"
        "85c075fe31c0b9040000004889e7fcf348af75fe48c704240100000048c74424"
        "080000000448c744241080654a0048b8ffffff7ffeffffff4889442418bf030000"
        "004889e631d241ba08000000b80d0000000f054885c075fe31c0488904244889"
        "4424084889442410488944241848b8a5a5a5a5a5a5a5a54889e7b904000000f3"
        "48abbf0f00000031f64889e241ba08000000b80d0000000f054885c075fe31c0"
        "b9040000004889e7fcf348af75febf010000004889e6b8050000000f054885c0"
        "75fe48833c240075fe48837c24080175fe48837c24100175fe48817c24188021"
        "000075fe48837c24200075fe48817c24280044000075fe48837c24300075fe48"
        "817c24380010000075fe48837c24400075fe48837c24480075fe48837c245000"
        "75fe48837c24580075fe48837c24600075fe48837c24680075fe48837c247000"
        "75fe48837c24780075fe4883bc24800000000075fe4883bc24880000000075fe"
        "31c0b9120000004889e7fcf348ab31c0b9120000004889e7fcf348af75fe"
        "bf9cffffffbe12694a00ba020000004531d2b8010100000f054883f8fe75fe"
        "c604246dbf02000000be015400004889e2b8100000000f054883f8e775fe"
        "803c246d75febf01000000be015400004889e2b8100000000f054883f8e775fe"
        "803c246d75fe31ffbe015400004889e2b8100000000f054883f8e775fe"
        "803c246d75fec6042400803c240075fe"
        "c7042473683a20bf020000004889e6ba04000000b8010000000f054883f80475fe"
        "c7042400000000833c240075fe"
        "48b863616e27742061634889042448b863657373207474794889442408"
        "48b83b206a6f6220636f488944241048b86e74726f6c2074754889442418"
        "48b8726e6564206f66664889442420"
        "bf020000004889e6ba28000000b8010000000f054883f82875fe"
        "31c0b9050000004889e7fcf348ab31c0b9050000004889e7fcf348af75fe"
        "c604240abf020000004889e6ba01000000b8010000000f054883f80175fe"
        "c6042400803c240075fe"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
        "bf9cffffffbe20694a004889e24531d2b8060100000f054883f8fe75fe"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348af75fe"
        "31c0b9120000004889e7fcf348ab31c0b9120000004889e7fcf348af75fe"
        "48b80a0a42757379426f4889042448b8782076312e33372e488944240848b830"
        "20284465626961488944241048b86e20313a312e3337488944241848b82e302d"
        "362b623929488944242048b8206275696c742d69488944242848b86e20736865"
        "6c6c20488944243048b828617368290a456e488944243848b874657220276865"
        "6c488944244048b8702720666f722061488944244848b8206c697374206f6648"
        "8944245048b8206275696c742d69488944245848b86e20636f6d6d616e488944"
        "246048b864732e0a0a0000004889442468bf010000004889e6ba6d000000b801"
        "0000000f054883f86d75fe31c0b90e0000004889e7fcf348ab31c0b90e000000"
        "4889e7fcf348af75fe"
        "c604246d31ffbe015400004889e2b8100000000f054883f8e775fe803c246d75fe"
        "c6042400803c240075fe"
        "66c7042560945f00a5a5bf60945f00be00100000b84f0000000f054883f80275fe"
        "66813c2560945f002f0075fe66c7042560945f00000066833c2560945f000075fe"
        "b86b0000000f054885c075fe"
        "bf9cffffffbe38694a0031d24531d2b8010100000f054883f80375fe"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
        "bf030000004889e6b8050000000f054885c075fe"
        "48833c240075fe48837c24080275fe48837c24100175fe"
        "48817c24182481000075fe48837c24200075fe48837c24280075fe"
        "48837c24300075fe48817c24380010000075fe"
        "48837c24400075fe48837c24480075fe48837c24500075fe"
        "48837c24580075fe48837c24600075fe48837c24680075fe"
        "48837c24700075fe48837c24780075fe"
        "4883bc24800000000075fe4883bc24880000000075fe"
        "31c0b9120000004889e7fcf348ab"
        "31c0b9120000004889e7fcf348af75fe"
        "48b8a5a5a5a5a5a5a5a54889e7b910000000fcf348ab"
        "bf030000004889e6ba8000000031c00f054885c075fe"
        "48b8a5a5a5a5a5a5a5a54889e7b910000000fcf348af75fe"
        "31c0b9100000004889e7fcf348ab"
        "31c0b9100000004889e7fcf348af75fe"
        "bf03000000b8030000000f054885c075fe"
        "c704242f202320bf020000004889e6ba04000000b8010000000f054883f80475fe"
        "48c704240000000048833c240075fe"
        "31ffbe608f5f00baff03000031c00f054883f80b75fe"
        "48b86563686f20746f7048390425608f5f0075fe"
        "48b8616c0a000000000048390425688f5f0075fe"
        "48b8746f70616c0a000048890424bf010000004889e6ba06000000b801000000"
        "0f054883f80675fe48c704240000000048833c240075fe"
        "c704242f202320bf020000004889e6ba04000000b8010000000f054883f80475fe"
        "48c704240000000048833c240075fe"
        "31ffbe608f5f00baff03000031c00f054883f80575fe"
        "48b86364202f0a00000048390425608f5f0075fe"
        "48833c25688f5f000075fe"
        "66c7042560945f002f00"
        "bf60945f00b8500000000f054885c075fe"
        "66c7042560945f000000"
        "c704242f202320bf020000004889e6ba04000000b8010000000f054883f80475fe"
        "48c704240000000048833c240075fe"
        "31ffbe608f5f00baff03000031c00f054883f85075fe"
        "48b86563686f202f2a3b48390425608f5f0075fe"
        "48b82074657374202d6448390425688f5f0075fe"
        "48b8202f62696e20262648390425708f5f0075fe"
        "48b8206563686f202f6248390425788f5f0075fe"
        "48b8696e2f2a3b20746548390425808f5f0075fe"
        "48b87374202d66202f6248390425888f5f0075fe"
        "48b8696e2f627573796248390425908f5f0075fe"
        "48b86f7820262620656348390425988f5f0075fe"
        "48b8686f20627573796248390425a08f5f0075fe"
        "48b86f782d66696c650a48390425a88f5f0075fe"
        "66c7042560945f002f00"
        "bf9cffffffbe60945f00ba000809004531d2b8010100000f054883f80375fe"
        "66c7042560945f000000"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
        "bf030000004889e6b8050000000f054885c075fe"
        "48833c240075fe48837c24080375fe48837c24100375fe"
        "48817c2418ed41000075fe48837c24200075fe48837c24280075fe"
        "48837c24300075fe48817c24380010000075fe"
        "48837c24400075fe48837c24480075fe48837c24500075fe"
        "48837c24580075fe48837c24600075fe48837c24680075fe"
        "48837c24700075fe48837c24780075fe"
        "4883bc24800000000075fe4883bc24880000000075fe"
        "31c0b9120000004889e7fcf348ab"
        "31c0b9120000004889e7fcf348af75fe"
        "31c0b9400000004889e7fcf348ab"
        "bf030000004889e6ba00020000b8d90000000f054883f84875fe"
        "48833c240375fe48837c24080175fe"
        "66837c24101875fe807c24120475fe807c24132e75fe"
        "48837c24180375fe48837c24200275fe"
        "66837c24281875fe807c242a0475fe807c242b2e75fe807c242c2e75fe"
        "48837c24300475fe48837c24380375fe"
        "66837c24401875fe807c24420475fe"
        "807c24436275fe807c24446975fe807c24456e75fe"
        "31c0b9090000004889e7fcf348ab"
        "bf030000004889e6ba00020000b8d90000000f054885c075fe"
        "bf03000000b8030000000f054885c075fe"
        "48b82f62696e0a00000048890424"
        "bf010000004889e6ba05000000b8010000000f054883f80575fe"
        "31c04889042448833c240075fe"
        "48b82f62696e000000004889042560945f00"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
        "bf9cffffffbe60945f004889e24531d2b8060100000f054885c075fe"
        "48833c240075fe48837c24080475fe48837c24100275fe"
        "48817c2418ed41000075fe48837c24200075fe48837c24280075fe"
        "48837c24300075fe48817c24380010000075fe"
        "48837c24400075fe48837c24480075fe48837c24500075fe"
        "48837c24580075fe48837c24600075fe48837c24680075fe"
        "48837c24700075fe48837c24780075fe"
        "4883bc24800000000075fe4883bc24880000000075fe"
        "31c0b9120000004889e7fcf348ab"
        "31c0b9120000004889e7fcf348af75fe"
        "48c7042560945f000000000048833c2560945f000075fe"
        "48b82f62696e2f0000004889042560945f00"
        "bf9cffffffbe60945f00ba000809004531d2b8010100000f054883f80375fe"
        "48c7042560945f0000000000"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
        "bf030000004889e6b8050000000f054885c075fe"
        "48833c240075fe48837c24080475fe48837c24100275fe"
        "48817c2418ed41000075fe48837c24200075fe48837c24280075fe"
        "48837c24300075fe48817c24380010000075fe"
        "48837c24400075fe48837c24480075fe48837c24500075fe"
        "48837c24580075fe48837c24600075fe48837c24680075fe"
        "48837c24700075fe48837c24780075fe"
        "4883bc24800000000075fe4883bc24880000000075fe"
        "31c0b9120000004889e7fcf348ab"
        "31c0b9120000004889e7fcf348af75fe"
        "31c0b9400000004889e7fcf348ab"
        "bf030000004889e6ba00020000b8d90000000f054883f85075fe"
        "48833c240475fe48837c24080175fe"
        "66837c24101875fe807c24120475fe807c24132e75fe"
        "48837c24180375fe48837c24200275fe"
        "66837c24281875fe807c242a0475fe807c242b2e75fe807c242c2e75fe"
        "48837c24300575fe48837c24380375fe"
        "66837c24402075fe807c24420875fe"
        "48b862757379626f7800483944244375fe"
        "31c0b90a0000004889e7fcf348ab"
        "bf030000004889e6ba00020000b8d90000000f054885c075fe"
        "31c0b90a0000004889e7fcf348af75fe"
        "bf03000000b8030000000f054885c075fe"
        "48b82f62696e2f62757348890424"
        "48b879626f780a0000004889442408"
        "bf010000004889e6ba0d000000b8010000000f054883f80d75fe"
        "31c048890424488944240848833c240075fe48837c24080075fe"
        "48b82f62696e2f6275734889042560945f00"
        "48b879626f78000000004889042568945f00"
        "48b8a5a5a5a5a5a5a5a54889e7b912000000fcf348ab"
        "bf9cffffffbe60945f004889e24531d2b8060100000f054885c075fe"
        "48833c240075fe48837c24080575fe48837c24100175fe"
        "48817c24186d81000075fe48837c24200075fe48837c24280075fe"
        "48837c24300075fe48817c24380010000075fe"
        "48837c24400075fe48837c24480075fe48837c24500075fe"
        "48837c24580075fe48837c24600075fe48837c24680075fe"
        "48837c24700075fe48837c24780075fe"
        "4883bc24800000000075fe4883bc24880000000075fe"
        "31c0b9120000004889e7fcf348ab"
        "31c0b9120000004889e7fcf348af75fe"
        "48c7042560945f000000000048c7042568945f0000000000"
        "48b862757379626f782d48890424"
        "48b866696c650a0000004889442408"
        "bf010000004889e6ba0d000000b8010000000f054883f80d75fe"
        "31c048890424488944240848833c240075fe48837c24080075fe"
        "c704242f202320bf020000004889e6ba04000000b8010000000f054883f80475fe"
        "48c704240000000048833c240075fe"
        "31ffbe608f5f00baff03000031c00f054883f80475fe"
        "48b87077640a0000000048390425608f5f0075fe"
        "48833c25688f5f000075fe"
        "66c7042560945f00a5a5"
        "bf60945f00be00100000b84f0000000f054883f80275fe"
        "66813c2560945f002f0075fe"
        "c6042561945f000a66813c2560945f002f0a75fe"
        "bf01000000be60945f00ba02000000b8010000000f054883f80275fe"
        "66c7042560945f00000066833c2560945f000075fe"
        "c704242f202320bf020000004889e6ba04000000b8010000000f054883f80475fe"
        "48c704240000000048833c240075fe"
        "31ffbe608f5f00baff03000031c00f054883f80575fe"
        "48b8657869740a00000048390425608f5f0075fe"
        "48833c25688f5f000075fe"
        "4881c40004000031ffb8e70000000f050f0b"
    )
    image[0xA_7000 : 0xA_7000 + len(continuation)] = continuation
    image[0xA_6900 : 0xA_6912] = b"/etc/busybox.conf\0"
    image[0xA_6912 : 0xA_691B] = b"/dev/tty\0"
    image[0xA_6920 : 0xA_6935] = b"/var/spool/mail/root\0"
    image[0xA_6938 : 0xA_6945] = b"/etc/profile\0"
    return bytes(image)


def checked_static_user_image(external: Path | None) -> tuple[bytes, str]:
    if external is None:
        return static_user_fixture(), "supplied-clean-room-fixture"
    image = external.read_bytes()
    actual = hashlib.sha256(image).hexdigest()
    if actual != PINNED_STATIC_BUSYBOX_SHA256:
        raise GateError(
            "external static user image does not match the approved pinned SHA-256"
        )
    return image, "pinned-external-static-busybox"


def build_image(
    destination: Path, external_static_user_image: Path | None = None
) -> tuple[Path, Path, Path, Path, Path, Path, str]:
    generated = destination / "generated"
    compile_systems_artifact(generated)
    generated_initial_user_image = generated / "initial-user.elf"
    supplied_initial_user_image = destination / "supplied-initial-user.elf"
    supplied = bytearray(generated_initial_user_image.read_bytes())
    supplied[4097] = 0xA5
    supplied_initial_user_image.write_bytes(supplied)
    supplied_static_user_image = destination / "supplied-static-user.elf"
    static_user_bytes, static_user_input = checked_static_user_image(
        external_static_user_image
    )
    supplied_static_user_image.write_bytes(static_user_bytes)

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
        static_user_input,
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
    static_user_input: str,
    serial: bytes,
) -> dict[str, Any]:
    data = manifest()
    boot = json.loads(boot_provenance.read_text(encoding="utf-8"))
    artifact = json.loads(artifact_provenance.read_text(encoding="utf-8"))
    if boot["static_user_image_sha256"] != digest(static_user_image):
        raise GateError("boot provenance does not bind the static user image")
    return {
        "schema": "topal-kernel-toolchain-gate-qemu/210",
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
        "static_user_image_input": static_user_input,
        "linked_kernel_sha256": boot["linked_kernel_sha256"],
        "artifact_provenance_sha256": digest(artifact_provenance),
        "boot_image_sha256": digest(image),
        "boot_resource_sha256": digest(boot_resource),
        "boot_provenance_sha256": digest(boot_provenance),
        "observations": {
            "markers_in_order": [marker.decode("ascii") for marker in MARKERS],
            "serial_utf8": serial.decode("utf-8"),
            "serial_sha256": hashlib.sha256(serial).hexdigest(),
            "static_applet_newline_count": serial.count(b"\n"),
            "static_shell_getpid_profile": (
                "external-shell-echo-command"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-pid-one-witness"
            ),
            "static_shell_sigchld_profile": (
                "external-shell-echo-command"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-installed-cleared-witness"
            ),
            "static_shell_getppid_profile": (
                "external-shell-echo-command"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-no-parent-witness"
            ),
            "static_shell_echo_command_profile": (
                "external-busybox-sh-echo-command-output-terminal"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-sealed-echo-command-witness"
            ),
            "static_shell_console_fstat_profile": (
                "external-optional-console-fstat-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-complete-console-fstat-witness"
            ),
            "static_shell_dev_tty_open_profile": (
                "external-optional-dev-tty-open-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-dev-tty-open-enoent-witness"
            ),
            "static_shell_standard_terminal_probe_profile": (
                "external-optional-standard-terminal-probes-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-stderr-stdout-stdin-enotty-witness"
            ),
            "static_shell_terminal_warning_profile": (
                "external-optional-terminal-warning-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-three-segment-stderr-warning-witness"
            ),
            "static_shell_mailpath_stat_profile": (
                "external-optional-mailpath-stat-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-root-mailpath-enoent-witness"
            ),
            "static_shell_banner_profile": (
                "external-optional-shell-banner-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-109-byte-shell-banner-witness"
            ),
            "static_shell_post_banner_terminal_probe_profile": (
                "external-optional-post-banner-terminal-probe-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-post-banner-stdin-enotty-witness"
            ),
            "static_shell_post_banner_getcwd_profile": (
                "external-optional-post-banner-getcwd-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-post-banner-sealed-root-witness"
            ),
            "static_shell_post_banner_geteuid_profile": (
                "external-optional-post-banner-geteuid-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-post-banner-effective-root-witness"
            ),
            "static_shell_profile_open_profile": (
                "external-optional-system-profile-open-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-read-only-system-profile-descriptor-three-witness"
            ),
            "static_shell_profile_fstat_profile": (
                "external-optional-system-profile-metadata-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-complete-system-profile-metadata-witness"
            ),
            "static_shell_profile_read_profile": (
                "external-optional-system-profile-read-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-empty-system-profile-eof-witness"
            ),
            "static_shell_profile_close_profile": (
                "external-optional-system-profile-close-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-consumed-system-profile-description-witness"
            ),
            "static_shell_prompt_profile": (
                "external-optional-root-prompt-absent"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-root-prompt-witness"
            ),
            "static_shell_refill_input_profile": (
                "external-five-publication-command-refill-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-five-publication-command-refill-witness"
            ),
            "static_shell_cd_root_profile": (
                "external-busybox-exact-root-chdir-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-root-chdir-witness"
            ),
            "static_shell_root_glob_open_profile": (
                "external-busybox-exact-root-directory-open-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-root-directory-open-witness"
            ),
            "static_shell_root_glob_fstat_profile": (
                "external-busybox-complete-root-directory-metadata-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-complete-root-directory-metadata-witness"
            ),
            "static_shell_root_glob_enumeration_profile": (
                "external-busybox-three-record-root-directory-enumeration-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-three-record-root-directory-enumeration-witness"
            ),
            "static_shell_root_glob_output_profile": (
                "external-busybox-exact-named-root-glob-match-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-named-root-glob-match-witness"
            ),
            "static_shell_named_root_entry_profile": (
                "external-busybox-sealed-bin-directory-entry-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-sealed-bin-directory-entry-witness"
            ),
            "static_shell_named_root_metadata_profile": (
                "external-busybox-complete-bin-directory-metadata-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-complete-bin-directory-metadata-witness"
            ),
            "static_shell_named_root_condition_profile": (
                "external-busybox-selected-bin-busybox-glob-condition-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-selected-bin-busybox-glob-condition-witness"
            ),
            "static_shell_empty_bin_open_profile": (
                "external-busybox-exact-bin-directory-open-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-bin-directory-open-witness"
            ),
            "static_shell_empty_bin_fstat_profile": (
                "external-busybox-complete-bin-directory-fstat-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-complete-bin-directory-fstat-witness"
            ),
            "static_shell_empty_bin_enumeration_profile": (
                "external-busybox-three-record-bin-busybox-enumeration-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-three-record-bin-busybox-enumeration-witness"
            ),
            "static_shell_empty_bin_close_profile": (
                "external-busybox-explicit-bin-directory-close-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-explicit-bin-directory-close-witness"
            ),
            "static_shell_empty_bin_output_profile": (
                "external-busybox-exact-bin-busybox-glob-match-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-bin-busybox-glob-match-witness"
            ),
            "static_shell_bin_busybox_entry_profile": (
                "external-busybox-sealed-bin-busybox-dirent-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-sealed-bin-busybox-dirent-witness"
            ),
            "static_shell_bin_busybox_metadata_profile": (
                "external-busybox-complete-bin-busybox-file-metadata-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-complete-bin-busybox-file-metadata-witness"
            ),
            "static_shell_bin_busybox_condition_output_profile": (
                "external-busybox-exact-bin-busybox-file-condition-output-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-bin-busybox-file-condition-output-witness"
            ),
            "static_shell_pwd_command_profile": (
                "external-busybox-pwd-cached-root-output-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-command-getcwd-root-output-witness"
            ),
            "static_shell_pwd_console_fstat_profile": (
                "external-command-console-fstat-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-command-console-fstat-absent"
            ),
            "static_shell_pwd_terminal_probe_profile": (
                "external-command-stdout-enotty-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-command-stdout-terminal-probe-absent"
            ),
            "static_shell_pwd_getcwd_profile": (
                "external-cached-sealed-root-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-command-getcwd-root-witness"
            ),
            "static_shell_pwd_output_profile": (
                "external-exact-application-root-line-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-exact-application-root-line-witness"
            ),
            "static_shell_stack_profile": (
                "external-affine-two-page-rw-nx-stack-witness"
                if static_user_input == "pinned-external-static-busybox"
                else "clean-affine-two-page-rw-nx-stack-witness"
            ),
            "qemu_running_after_markers": True,
            "serial_quiescent_after_fatal_disposition": True,
            "result": "pass",
        },
    }


def verify(
    timeout: float,
    write_evidence: bool,
    external_static_user_image: Path | None = None,
) -> dict[str, Any]:
    # Reject an unapproved external image before invoking any build tool.
    checked_static_user_image(external_static_user_image)
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
            static_user_input,
        ) = build_image(root / "artifact", external_static_user_image)
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
        expected_newline_count = (
            5
            if static_user_input == "pinned-external-static-busybox"
            else 12
        )
        if serial.count(b"\n") != expected_newline_count:
            raise GateError(
                "static applet newline count did not match the selected qualification input"
            )
        record = evidence_record(
            qemu_version,
            image,
            boot_resource,
            boot_provenance,
            artifact_provenance,
            generated_initial_user_image,
            static_user_image,
            static_user_input,
            serial,
        )
    if write_evidence:
        evidence_path = (
            PINNED_STATIC_BUSYBOX_EVIDENCE_PATH
            if external_static_user_image is not None
            else EVIDENCE_PATH
        )
        evidence_path.parent.mkdir(parents=True, exist_ok=True)
        staged = evidence_path.with_suffix(".json.new")
        staged.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        staged.replace(evidence_path)
        print(f"wrote {evidence_path}")
    return record


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--write-evidence", action="store_true")
    parser.add_argument("--external-static-user-image", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        result = verify(
            args.timeout,
            args.write_evidence,
            args.external_static_user_image,
        )
        print(json.dumps(result, indent=2, sort_keys=True))
    except (GateError, OSError, subprocess.SubprocessError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
