#!/usr/bin/env python3
"""Run independent generic file laws on pinned Linux in a disposable VM overlay."""
from __future__ import annotations
import argparse
import importlib.util
import json
from pathlib import Path
import re
import shlex
import shutil
import time
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
    with tempfile.TemporaryDirectory(prefix='topal-positional-control-', dir='/tmp') as temporary:
        root = Path(temporary)
        executable = root / 'positional-file-control'
        subprocess.run(['cc', '-O2', '-Wall', '-Wextra', '-Werror',
                        str(LAB / 'positional_file_control.c'), '-o', str(executable)], check=True)
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
                   .replace('topal-linux-reference', 'topal-positional-control') for value in command]
        accelerator=command.index('-accel')+1
        command[accelerator]+=',tb-size=32'
        print('START disposable pinned-Linux filesystem control', flush=True)
        with (root / 'qemu.log').open('wb') as output:
            process = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
            try:
                deadline=time.monotonic()+600
                while True:
                    try:
                        VM.wait_for_ssh(process, port, timeout=max(1,int(deadline-time.monotonic())))
                        break
                    except subprocess.TimeoutExpired:
                        if time.monotonic()>=deadline:
                            raise

                release = VM.remote_checked(port, 'uname -r')
                if release != manifest['inputs']['linux']['release']:
                    raise VM.LabError('control booted the wrong kernel: ' + release)
                subprocess.run(['scp', '-i', str(paths['key']), '-P', str(port),
                    '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=no',
                    '-o', 'UserKnownHostsFile=/dev/null', '-o', 'LogLevel=ERROR',
                    str(executable), 'topal@127.0.0.1:/tmp/topal-positional-file-control'], check=True)
                directory = '/tmp/topal-positional-control'
                VM.remote_checked(port, 'sudo mkdir -p ' + shlex.quote(directory))
                VM.remote_checked(port, 'sudo mount -t tmpfs -o size=32768,nr_inodes=5,mode=1777,noatime topal-positional-control ' + shlex.quote(directory))
                mount = VM.remote_checked(port, 'findmnt -n -o FSTYPE,OPTIONS --target ' + shlex.quote(directory))
                immutable = directory + '-readonly'
                VM.remote_checked(port, 'sudo mkdir -p ' + shlex.quote(immutable))
                VM.remote_checked(port, 'sudo mount -t tmpfs -o size=4096,nr_inodes=8,mode=1777,noatime topal-positional-readonly ' + shlex.quote(immutable))
                VM.remote_checked(port, "printf immutable-bytes | sudo tee " + shlex.quote(immutable + '/exists') + " >/dev/null")
                VM.remote_checked(port, 'sudo mount -o remount,ro ' + shlex.quote(immutable))
                result = VM.remote_checked(port, 'sudo /tmp/topal-positional-file-control ' + shlex.quote(directory) + ' ' + shlex.quote(immutable))
                if not result.startswith('POSITIONAL_FILE_CONTROL_PASS '):
                    raise VM.LabError('control did not complete its value-law assertions')
                dispositions = json.loads(result.removeprefix('POSITIONAL_FILE_CONTROL_PASS '))
                expected = {'append_requested_overflow': 22, 'append_zero_max': 0, 'bad_descriptor': 9, 'negative_bad_descriptor': 22, 'negative_write_bad_descriptor': 22, 'pipe_negative': 22, 'pipe_read_bad_buffer': 29, 'pipe_read_zero': 29, 'pipe_write_zero': 29, 'pipe_wrong_direction': 29, 'read_bad_buffer_zero': 14, 'read_directory_bad_buffer': 14, 'read_directory_overflow': 22, 'read_directory_zero': 21, 'read_null_zero': 0, 'read_overflow': 22, 'read_write_only': 9, 'read_zero_max': 0, 'write_bad_buffer_zero': 14, 'write_directory': 9, 'write_null_zero': 0, 'write_overflow': 22, 'write_read_only': 9, 'write_zero_max': 0}
                if dispositions != expected:
                    raise VM.LabError('pinned positional-file dispositions changed: ' + repr(dispositions))
                print('Pinned Linux dispositions:', json.dumps(dispositions, sort_keys=True), flush=True)
                libc = VM.remote_checked(port, "dpkg-query -W -f='${Version}' libc6")
                VM.remote_checked(port, 'sudo umount ' + shlex.quote(directory))
                VM.remote_checked(port, 'sudo umount ' + shlex.quote(immutable))
                VM.remote_checked(port, 'sudo systemctl poweroff', timeout=30)
                process.wait(timeout=60)
                if process.returncode != 0:
                    raise VM.LabError('reference VM did not shut down cleanly')
                evidence = dict(schema='topal-positional-file-linux-control/1', result='pass',
                    harness_sha256=VM.digest(Path(__file__), 'sha256'),
                    dispositions=dispositions,
                    source_sha256=VM.digest(LAB / 'positional_file_control.c', 'sha256'),
                    executable_sha256=VM.digest(executable, 'sha256'),
                    manifest_sha256=VM.digest(VM.MANIFEST_PATH, 'sha256'),
                    reference_kernel_sha256=VM.digest(kernel, 'sha256'),
                    kernel_release=release, guest_libc=libc, needed_glibc=needed,
                    filesystem=mount, output=result,
                    scope='raw pread64/pwrite64 ordering, independent positions, append quirk, sparse bytes and unlinked lifetime; no full tmpfs-driver equivalence',
                    tcg_translation_cache_mib=32, qemu_command=command,
                    disposable_overlay=True)
            except BaseException:
                failure=Path(tempfile.mkdtemp(prefix='topal-positional-failure-',dir='/tmp'))
                for name in ('qemu.log','serial.log'):
                    if (root/name).exists():shutil.copy2(root/name,failure/name)
                (failure/'command.json').write_text(json.dumps(command,indent=2))
                print('Saved failed-control logs: '+str(failure),flush=True)
                raise
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=15)
    if write_evidence:
        (LAB / 'results/topal-positional-file-linux-control.json').write_text(json.dumps(evidence, indent=2, sort_keys=True) + '\n')
    print('PASS pinned Linux positional-file dispositions, bytes, independent positions and unlinked lifetime', flush=True)
    return evidence


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-evidence', action='store_true')
    args = parser.parse_args()
    verify(args.write_evidence)
