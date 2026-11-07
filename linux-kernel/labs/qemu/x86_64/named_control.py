#!/usr/bin/env python3
"""Run independent generic file laws on pinned Linux in a disposable VM overlay."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import re
import shlex
import socket
import subprocess
import tempfile

LAB = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('reference_vm', LAB / 'vm.py')
VM = importlib.util.module_from_spec(spec)
spec.loader.exec_module(VM)


def verify(write_evidence=False):
    manifest = VM.manifest()
    paths = VM.state_paths()
    kernel = VM.REPOSITORY / manifest['boot']['reference']['kernel']
    if VM.digest(kernel, 'sha256') != manifest['inputs']['linux']['bzimage_sha256']:
        raise VM.LabError('volatile-file control needs the exact pinned reference kernel')
    with tempfile.TemporaryDirectory(prefix='topal-named-control-', dir='/tmp') as temporary:
        root = Path(temporary)
        executable = root / 'named-file-control'
        subprocess.run(['cc', '-O2', '-Wall', '-Wextra', '-Werror',
                        str(LAB / 'named_file_control.c'), '-o', str(executable)], check=True)
        versions = subprocess.check_output(['readelf', '--version-info', str(executable)], text=True)
        needed = sorted(set(re.findall(r'GLIBC_(\d+(?:\.\d+)+)', versions)))
        if any(tuple(map(int, version.split('.'))) > (2, 41) for version in needed):
            raise VM.LabError('control binary requires a newer libc than the pinned guest')
        overlay = root / 'control.qcow2'
        subprocess.run(['qemu-img', 'create', '-f', 'qcow2', '-F', 'qcow2', '-b',
                        str(paths['overlay']), str(overlay)], check=True)
        with socket.socket() as probe:
            probe.bind(('127.0.0.1', 0))
            port = probe.getsockname()[1]
        command = VM.qemu_command('reference', port, 'file')
        command = [value.replace(f'file={paths["overlay"]},', f'file={overlay},')
                   .replace(f'path={paths["serial"]}', f'path={root / "serial.log"}')
                   .replace('topal-linux-reference', 'topal-named-control') for value in command]
        print('START disposable pinned-Linux filesystem control', flush=True)
        with (root / 'qemu.log').open('wb') as output:
            process = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
            try:
                VM.wait_for_ssh(process, port)
                release = VM.remote_checked(port, 'uname -r')
                if release != manifest['inputs']['linux']['release']:
                    raise VM.LabError('control booted the wrong kernel: ' + release)
                subprocess.run(['scp', '-i', str(paths['key']), '-P', str(port),
                    '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=no',
                    '-o', 'UserKnownHostsFile=/dev/null', '-o', 'LogLevel=ERROR',
                    str(executable), 'topal@127.0.0.1:/tmp/topal-named-file-control'], check=True)
                directory = '/tmp/topal-named-control'
                VM.remote_checked(port, 'sudo mkdir -p ' + shlex.quote(directory))
                VM.remote_checked(port, 'sudo mount -t tmpfs -o size=32768,nr_inodes=5,mode=1777,noatime topal-named-control ' + shlex.quote(directory))
                mount = VM.remote_checked(port, 'findmnt -n -o FSTYPE,OPTIONS --target ' + shlex.quote(directory))
                immutable = directory + '-readonly'
                VM.remote_checked(port, 'sudo mkdir -p ' + shlex.quote(immutable))
                VM.remote_checked(port, 'sudo mount -t tmpfs -o size=4096,nr_inodes=8,mode=1777,noatime topal-named-readonly ' + shlex.quote(immutable))
                VM.remote_checked(port, 'sudo touch ' + shlex.quote(immutable + '/exists'))
                VM.remote_checked(port, 'sudo mount -o remount,ro ' + shlex.quote(immutable))
                result = VM.remote_checked(port, 'sudo /tmp/topal-named-file-control ' + shlex.quote(directory) + ' ' + shlex.quote(immutable))
                if not result.startswith('NAMED_FILE_CONTROL_PASS '):
                    raise VM.LabError('control did not complete its value-law assertions')
                dispositions = json.loads(result.removeprefix('NAMED_FILE_CONTROL_PASS '))
                expected = dict(create_dot=21, create_missing_trailing=21, create_regular_trailing=21,
                    exclusive_dot=17, exclusive_existing=17, exclusive_regular_trailing=21,
                    read_missing_trailing=2, readonly_create_dot=21, readonly_create_existing=0,
                    readonly_create_missing=30, readonly_exclusive_existing=17,
                    readonly_unlink_long_final=30, readonly_unlink_missing=30,
                    readonly_unlink_missing_trailing=30, unlink_dot=21,
                    unlink_invalid_flags_before_copy=22, unlink_missing_trailing=2,
                    unlink_parent=21, unlink_regular_trailing=20)
                if dispositions != expected:
                    raise VM.LabError('pinned named-file error dispositions changed: ' + repr(dispositions))
                print('Pinned Linux dispositions:', json.dumps(dispositions, sort_keys=True), flush=True)
                libc = VM.remote_checked(port, "dpkg-query -W -f='${Version}' libc6")
                VM.remote_checked(port, 'sudo umount ' + shlex.quote(directory))
                VM.remote_checked(port, 'sudo umount ' + shlex.quote(immutable))
                VM.remote_checked(port, 'sudo systemctl poweroff', timeout=30)
                process.wait(timeout=60)
                if process.returncode != 0:
                    raise VM.LabError('reference VM did not shut down cleanly')
                evidence = dict(schema='topal-named-file-linux-control/1', result='pass',
                    harness_sha256=VM.digest(Path(__file__), 'sha256'),
                    dispositions=dispositions,
                    source_sha256=VM.digest(LAB / 'named_file_control.c', 'sha256'),
                    executable_sha256=VM.digest(executable, 'sha256'),
                    manifest_sha256=VM.digest(VM.MANIFEST_PATH, 'sha256'),
                    reference_kernel_sha256=VM.digest(kernel, 'sha256'),
                    kernel_release=release, guest_libc=libc, needed_glibc=needed,
                    filesystem=mount, output=result,
                    scope='generic named-file error ordering, independent descriptions, unlink, capacity and long records; no full tmpfs-driver equivalence',
                    disposable_overlay=True)
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=15)
    if write_evidence:
        (LAB / 'results/topal-named-file-linux-control.json').write_text(json.dumps(evidence, indent=2, sort_keys=True) + '\n')
    print('PASS pinned Linux named-file dispositions, bytes, lifetime, capacity and long records', flush=True)
    return evidence


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-evidence', action='store_true')
    args = parser.parse_args()
    verify(args.write_evidence)
