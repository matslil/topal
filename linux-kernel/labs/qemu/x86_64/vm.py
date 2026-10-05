#!/usr/bin/env python3
"""Prepare and qualify the pinned Topal x86-64 Linux reference VM."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any
import urllib.request


LAB = Path(__file__).resolve().parent
REPOSITORY = LAB.parents[3]
LINUX_KERNEL = REPOSITORY / "linux-kernel"
CACHE = LINUX_KERNEL / ".cache" / "qemu-x86_64"
DOWNLOADS = CACHE / "downloads"
STATE = CACHE / "state"
SOURCE_ROOT = CACHE / "sources"
BUILD = LINUX_KERNEL / "build" / "x86_64" / "linux-7.2.9"
MANIFEST_PATH = LAB / "manifest.json"
REFERENCE_CONFIG = LAB / "reference-kernel.config"
CONFIG_FRAGMENT = LAB / "reference-kernel.config.fragment"
EVIDENCE = LAB / "results" / "verification.json"


class LabError(Exception):
    """The pinned VM contract could not be satisfied."""


def manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def run(
    command: list[str],
    *,
    check: bool = True,
    capture: bool = False,
    timeout: int | None = None,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(command), flush=True)
    return subprocess.run(
        command,
        check=check,
        text=True,
        capture_output=capture,
        timeout=timeout,
        env=env,
    )


def digest(path: Path, algorithm: str) -> str:
    value = hashlib.new(algorithm)
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def verify_file(path: Path, algorithm: str, expected: str) -> None:
    if not path.is_file():
        raise LabError(f"missing input: {path}")
    actual = digest(path, algorithm)
    if actual != expected:
        raise LabError(f"{path} {algorithm} is {actual}, expected {expected}")


def download(url: str, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    partial = destination.with_suffix(destination.suffix + ".partial")
    print(f"downloading {url}", flush=True)
    with urllib.request.urlopen(url) as response, partial.open("wb") as output:
        shutil.copyfileobj(response, output, length=1024 * 1024)
    partial.replace(destination)


def fetch() -> None:
    data = manifest()["inputs"]
    for key, algorithm in (("debian_cloud_image", "sha512"), ("linux", "sha256")):
        record = data[key]
        path = DOWNLOADS / record["file"]
        if not path.exists():
            download(record["url"], path)
        verify_file(path, algorithm, record[algorithm])
        print(f"verified {path.name}", flush=True)


def require_program(name: str) -> str:
    path = shutil.which(name)
    if path is None:
        raise LabError(f"required host program is unavailable: {name}")
    return path


def verify_host() -> None:
    data = manifest()
    for program in ("genisoimage", "qemu-img", "qemu-system-x86_64", "ssh", "ssh-keygen"):
        require_program(program)
    version = run(["qemu-system-x86_64", "--version"], capture=True).stdout.splitlines()[0]
    expected_version = data["emulator"]["version"]
    if f"version {expected_version}" not in version:
        raise LabError(f"QEMU version mismatch: {version!r}; expected {expected_version}")
    firmware = Path(data["firmware"]["path"])
    verify_file(firmware, "sha256", data["firmware"]["sha256"])
    if firmware.stat().st_size != data["firmware"]["size"]:
        raise LabError(f"firmware size mismatch: {firmware}")


def extract_linux() -> Path:
    data = manifest()["inputs"]["linux"]
    archive = DOWNLOADS / data["file"]
    source = SOURCE_ROOT / f"linux-{data['release']}"
    if not source.is_dir():
        SOURCE_ROOT.mkdir(parents=True, exist_ok=True)
        run(["tar", "-C", str(SOURCE_ROOT), "-xf", str(archive)])
    return source


def configure_kernel(write: bool) -> None:
    fetch()
    source = extract_linux()
    BUILD.mkdir(parents=True, exist_ok=True)
    run(["make", "-C", str(source), f"O={BUILD}", "ARCH=x86_64", "x86_64_defconfig"])
    run(
        [
            str(source / "scripts/kconfig/merge_config.sh"),
            "-m",
            "-O",
            str(BUILD),
            str(BUILD / ".config"),
            str(CONFIG_FRAGMENT),
        ]
    )
    run(["make", "-C", str(source), f"O={BUILD}", "ARCH=x86_64", "olddefconfig"])
    if write:
        shutil.copyfile(BUILD / ".config", REFERENCE_CONFIG)
        print(f"wrote {REFERENCE_CONFIG}")
    elif not REFERENCE_CONFIG.is_file():
        raise LabError(f"missing committed kernel configuration: {REFERENCE_CONFIG}")
    elif (BUILD / ".config").read_bytes() != REFERENCE_CONFIG.read_bytes():
        raise LabError("generated kernel configuration differs from reference-kernel.config")


def build_kernel() -> None:
    fetch()
    source = extract_linux()
    if not REFERENCE_CONFIG.is_file():
        raise LabError("run configure-kernel --write-config before building")
    BUILD.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(REFERENCE_CONFIG, BUILD / ".config")
    before = (BUILD / ".config").read_bytes()
    build_data = manifest()["inputs"]["linux"]
    verify_file(REFERENCE_CONFIG, "sha256", build_data["configuration_sha256"])
    build_environment = os.environ.copy()
    build_environment.update(
        {
            "KBUILD_BUILD_HOST": build_data["build_host"],
            "KBUILD_BUILD_TIMESTAMP": "Sun Oct 4 00:00:00 UTC 2026",
            "KBUILD_BUILD_USER": build_data["build_user"],
            "KBUILD_BUILD_VERSION": str(build_data["build_version"]),
            "SOURCE_DATE_EPOCH": str(build_data["source_date_epoch"]),
        }
    )
    run(
        ["make", "-C", str(source), f"O={BUILD}", "ARCH=x86_64", "olddefconfig"],
        env=build_environment,
    )
    if (BUILD / ".config").read_bytes() != before:
        raise LabError("Linux 7.2.9 changed the committed reference configuration")
    run(
        ["make", "-C", str(source), f"O={BUILD}", "ARCH=x86_64", "-j2", "bzImage"],
        env=build_environment,
    )
    verify_file(
        BUILD / "arch" / "x86" / "boot" / "bzImage",
        "sha256",
        build_data["bzimage_sha256"],
    )


def state_paths() -> dict[str, Path]:
    return {
        "overlay": STATE / "root.qcow2",
        "seed": STATE / "seed.iso",
        "key": STATE / "id_ed25519",
        "public_key": STATE / "id_ed25519.pub",
        "serial": STATE / "serial.log",
        "qemu": STATE / "qemu.log",
    }


def prepare(reset: bool) -> None:
    fetch()
    verify_host()
    paths = state_paths()
    STATE.mkdir(parents=True, exist_ok=True)
    base = DOWNLOADS / manifest()["inputs"]["debian_cloud_image"]["file"]
    if reset:
        for key in ("overlay", "seed", "serial", "qemu"):
            paths[key].unlink(missing_ok=True)
        for partial in STATE.glob("verification-*.json"):
            partial.unlink()
    if not paths["key"].exists():
        run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", "topal-step3", "-f", str(paths["key"])])
    if not paths["overlay"].exists():
        run(
            [
                "qemu-img",
                "create",
                "-q",
                "-f",
                "qcow2",
                "-F",
                "qcow2",
                "-b",
                str(base.resolve()),
                str(paths["overlay"]),
                "12G",
            ]
        )
    template = (LAB / "cloud-init" / "user-data.in").read_text(encoding="utf-8")
    public_key = paths["public_key"].read_text(encoding="utf-8").strip()
    rendered = template.replace("@SSH_PUBLIC_KEY@", public_key)
    with tempfile.TemporaryDirectory(prefix="topal-seed-", dir=STATE) as temporary:
        seed_root = Path(temporary)
        (seed_root / "user-data").write_text(rendered, encoding="utf-8")
        shutil.copyfile(LAB / "cloud-init" / "meta-data", seed_root / "meta-data")
        run(
            [
                "genisoimage",
                "-quiet",
                "-output",
                str(paths["seed"]),
                "-volid",
                "cidata",
                "-joliet",
                "-rock",
                str(seed_root / "user-data"),
                str(seed_root / "meta-data"),
            ]
        )


def qemu_command(boot: str, ssh_port: int, serial_backend: str) -> list[str]:
    data = manifest()
    emulator = data["emulator"]
    firmware = data["firmware"]
    paths = state_paths()
    machine = data["machine"]
    command = [
        "qemu-system-x86_64",
        "-name",
        "topal-linux-reference,process=topal-linux-reference",
        "-nodefaults",
        "-no-reboot",
        "-display",
        "none",
        "-monitor",
        "none",
        "-machine",
        f"{emulator['machine']},smm={'on' if machine['smm'] else 'off'},vmport={'on' if machine['vmport'] else 'off'},usb={'on' if machine['usb'] else 'off'},sata={'on' if machine['sata'] else 'off'},dump-guest-core=off",
        "-accel",
        f"tcg,thread={emulator['tcg_thread']}",
        "-cpu",
        emulator["cpu"],
        "-smp",
        f"cpus={emulator['vcpus']},sockets={emulator['topology']['sockets']},cores={emulator['topology']['cores']},threads={emulator['topology']['threads']}",
        "-m",
        f"{emulator['memory_mib']}M",
        "-uuid",
        emulator["uuid"],
        "-bios",
        firmware["path"],
        "-rtc",
        "base=utc,clock=vm,driftfix=slew",
        "-boot",
        "order=c,strict=on",
        "-global",
        "ICH9-LPC.disable_s3=1",
        "-global",
        "ICH9-LPC.disable_s4=1",
        "-drive",
        f"if=none,id=root,file={paths['overlay']},format=qcow2,cache=none,aio=threads,discard=ignore",
        "-device",
        "virtio-blk-pci-non-transitional,drive=root,addr=03.0,serial=topal-root,bootindex=1,num-queues=1,queue-size=256,discard=off,write-zeroes=off",
        "-netdev",
        f"user,id=net0,hostfwd=tcp:127.0.0.1:{ssh_port}-:22",
        "-device",
        "virtio-net-pci-non-transitional,netdev=net0,addr=04.0,mac=52:54:00:54:4f:50,mq=off,rx_queue_size=256,tx_queue_size=256",
        "-object",
        "rng-random,id=rng0,filename=/dev/urandom",
        "-device",
        "virtio-rng-pci-non-transitional,rng=rng0,addr=05.0,max-bytes=1024,period=1000",
    ]
    if serial_backend == "stdio":
        command.extend(["-chardev", "stdio,id=serial0,signal=off"])
    else:
        command.extend(["-chardev", f"file,id=serial0,path={paths['serial']}"])
    command.extend(["-device", "isa-serial,chardev=serial0,index=0"])
    command.extend(
        [
            "-drive",
            f"if=none,id=seed,file={paths['seed']},format=raw,readonly=on",
            "-device",
            "virtio-blk-pci-non-transitional,drive=seed,addr=06.0,serial=topal-seed,num-queues=1,queue-size=128",
        ]
    )
    if boot == "reference":
        kernel = REPOSITORY / data["boot"]["reference"]["kernel"]
        if not kernel.is_file():
            raise LabError(f"missing reference kernel: {kernel}; run build-kernel")
        command.extend(
            [
                "-kernel",
                str(kernel),
                "-append",
                data["boot"]["reference"]["command_line"],
            ]
        )
    elif boot != "distro":
        raise LabError(f"unknown boot mode: {boot}")
    return command


def ssh_command(port: int, remote: str, *, timeout: int = 60) -> subprocess.CompletedProcess[str]:
    key = state_paths()["key"]
    command = [
        "ssh",
        "-i",
        str(key),
        "-p",
        str(port),
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "UserKnownHostsFile=/dev/null",
        "-o",
        "LogLevel=ERROR",
        "-o",
        "ConnectTimeout=5",
        "topal@127.0.0.1",
        remote,
    ]
    return run(command, check=False, capture=True, timeout=timeout)


def wait_for_ssh(process: subprocess.Popen[bytes], port: int, timeout: int = 600) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        status = process.poll()
        if status is not None:
            raise LabError(f"QEMU exited with status {status} before SSH became ready")
        probe = ssh_command(port, "true", timeout=10)
        if probe.returncode == 0:
            return
        time.sleep(2)
    raise LabError("timed out waiting for SSH")


def remote_checked(port: int, command: str, timeout: int = 120) -> str:
    result = ssh_command(port, command, timeout=timeout)
    if result.returncode != 0:
        raise LabError(
            f"guest command failed ({result.returncode}): {command}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result.stdout.strip()


def collect_guest(port: int) -> dict[str, str]:
    commands = {
        "uname": "uname -srvmo",
        "kernel_release": "uname -r",
        "os_release": "cat /etc/os-release",
        "packages": "dpkg-query -W -f='${Package}\\t${Version}\\n' libc6 busybox-static docker.io podman",
        "docker_version": "sudo docker version --format '{{.Client.Version}}/{{.Server.Version}}'",
        "podman_version": "sudo podman version --format '{{.Client.Version}}'",
        "cgroup_filesystems": "grep -E 'cgroup|cgroup2' /proc/filesystems",
        "namespaces": "ls -1 /proc/self/ns",
        "kernel_command_line": "cat /proc/cmdline",
        "memory_map": "sudo cat /proc/iomem",
        "interrupts": "sudo cat /proc/interrupts",
        "clocksource": "cat /sys/devices/system/clocksource/clocksource0/current_clocksource",
        "consoles": "cat /proc/consoles",
        "block_devices": "lsblk --json --output NAME,PATH,TYPE,SIZE,FSTYPE,MOUNTPOINTS,SERIAL",
        "pci": "lspci -nn",
        "cpu": "sed -n '1,32p' /proc/cpuinfo",
        "acpi_tables": "sudo sh -c \"find /sys/firmware/acpi/tables -maxdepth 1 -type f -print0 | sort -z | xargs -0 sha256sum\"",
    }
    return {name: remote_checked(port, command) for name, command in commands.items()}


def serial_tail() -> str:
    path = state_paths()["serial"]
    if not path.exists():
        return "serial log was not created"
    return "\n".join(path.read_text(encoding="utf-8", errors="replace").splitlines()[-80:])


def verify_boot(boot: str, port: int) -> dict[str, Any]:
    prepare(reset=False)
    paths = state_paths()
    paths["serial"].unlink(missing_ok=True)
    with paths["qemu"].open("wb") as qemu_log:
        process = subprocess.Popen(qemu_command(boot, port, "file"), stdout=qemu_log, stderr=subprocess.STDOUT)
        try:
            wait_for_ssh(process, port)
            if boot == "distro":
                cloud_init = ssh_command(port, "cloud-init status --wait", timeout=1200)
                if cloud_init.returncode != 0:
                    print(
                        "cloud-init did not complete cleanly; resuming the idempotent "
                        "provisioner over SSH",
                        flush=True,
                    )
                remote_checked(
                    port,
                    "sudo test -e /var/lib/topal-smoke/provisioned || (sudo dpkg --configure -a && sudo apt-get -o Acquire::Check-Valid-Until=false install -y --no-install-recommends docker-cli && sudo /usr/local/sbin/topal-provision)",
                    timeout=1200,
                )
                remote_checked(port, "test -e /var/lib/topal-smoke/provisioned")
            release = remote_checked(port, "uname -r")
            if boot == "reference" and release != manifest()["inputs"]["linux"]["release"]:
                raise LabError(f"reference VM booted kernel {release}, expected 7.2.9")
            smoke = remote_checked(port, "sudo /usr/local/sbin/topal-smoke", timeout=300)
            if "STEP3_SMOKE_OK" not in smoke:
                raise LabError("guest smoke test did not emit its completion marker")
            facts = collect_guest(port)
            remote_checked(port, "sudo systemctl poweroff", timeout=30)
            try:
                process.wait(timeout=60)
            except subprocess.TimeoutExpired:
                process.terminate()
                process.wait(timeout=15)
            if process.returncode != 0:
                raise LabError(f"QEMU exited with status {process.returncode}")
            return {"boot": boot, "smoke": smoke, "facts": facts}
        except Exception as error:
            process.terminate()
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=15)
            raise LabError(f"{error}\nlast serial output:\n{serial_tail()}") from error


def verification_evidence(results: list[dict[str, Any]]) -> dict[str, Any]:
    data = manifest()
    return {
        "schema": "topal-linux-reference-vm-verification/1",
        "profile": data["profile"],
        "manifest_sha256": digest(MANIFEST_PATH, "sha256"),
        "qemu": run(["qemu-system-x86_64", "--version"], capture=True).stdout.splitlines()[0],
        "firmware_sha256": digest(Path(data["firmware"]["path"]), "sha256"),
        "base_image_sha512": digest(
            DOWNLOADS / data["inputs"]["debian_cloud_image"]["file"], "sha512"
        ),
        "reference_config_sha256": digest(REFERENCE_CONFIG, "sha256"),
        "reference_bzimage_sha256": digest(
            BUILD / "arch" / "x86" / "boot" / "bzImage", "sha256"
        ),
        "results": results,
    }


def record_result(result: dict[str, Any]) -> None:
    partial = STATE / f"verification-{result['boot']}.json"
    partial.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    partials = [STATE / "verification-distro.json", STATE / "verification-reference.json"]
    if all(path.is_file() for path in partials):
        results = [json.loads(path.read_text(encoding="utf-8")) for path in partials]
        EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
        EVIDENCE.write_text(
            json.dumps(verification_evidence(results), indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        print(f"wrote {EVIDENCE}")
    else:
        print(f"wrote partial evidence {partial}")


def verify_all(port: int, write_evidence: bool) -> None:
    results = [verify_boot("distro", port), verify_boot("reference", port)]
    evidence = verification_evidence(results)
    if write_evidence:
        EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
        EVIDENCE.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {EVIDENCE}")
    print("Step 3 distribution and Linux 7.2.9 reference boots passed")


def assemble_evidence() -> None:
    partials = [STATE / "verification-distro.json", STATE / "verification-reference.json"]
    missing = [path for path in partials if not path.is_file()]
    if missing:
        raise LabError(f"missing partial verification evidence: {', '.join(map(str, missing))}")
    results = [json.loads(path.read_text(encoding="utf-8")) for path in partials]
    EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
    EVIDENCE.write_text(
        json.dumps(verification_evidence(results), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"wrote {EVIDENCE}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("fetch", help="download and verify immutable inputs")
    prepare_parser = subparsers.add_parser("prepare", help="create the writable VM and NoCloud seed")
    prepare_parser.add_argument("--reset", action="store_true", help="replace the writable VM overlay")
    configure = subparsers.add_parser("configure-kernel", help="derive the reference Linux configuration")
    configure.add_argument("--write-config", action="store_true", help="update the committed reference config")
    subparsers.add_parser("build-kernel", help="build the pinned Linux 7.2.9 bzImage")
    subparsers.add_parser("assemble-evidence", help="combine the latest successful boot records")
    run_parser = subparsers.add_parser("run", help="run an interactive serial console")
    run_parser.add_argument("--boot", choices=("distro", "reference"), required=True)
    run_parser.add_argument("--ssh-port", type=int, default=2222)
    verify_parser = subparsers.add_parser("verify", help="boot and qualify one kernel")
    verify_parser.add_argument("--boot", choices=("distro", "reference"), required=True)
    verify_parser.add_argument("--ssh-port", type=int, default=2222)
    verify_parser.add_argument("--write-evidence", action="store_true")
    all_parser = subparsers.add_parser("verify-all", help="qualify distro and reference kernels")
    all_parser.add_argument("--ssh-port", type=int, default=2222)
    all_parser.add_argument("--write-evidence", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "fetch":
            fetch()
        elif args.command == "prepare":
            prepare(args.reset)
        elif args.command == "configure-kernel":
            configure_kernel(args.write_config)
        elif args.command == "build-kernel":
            build_kernel()
        elif args.command == "assemble-evidence":
            assemble_evidence()
        elif args.command == "run":
            prepare(reset=False)
            return subprocess.call(qemu_command(args.boot, args.ssh_port, "stdio"))
        elif args.command == "verify":
            result = verify_boot(args.boot, args.ssh_port)
            print(json.dumps(result, indent=2, sort_keys=True))
            if args.write_evidence:
                record_result(result)
        elif args.command == "verify-all":
            verify_all(args.ssh_port, args.write_evidence)
        else:
            raise AssertionError(args.command)
    except (LabError, OSError, subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
