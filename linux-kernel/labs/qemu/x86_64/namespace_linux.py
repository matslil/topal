#!/usr/bin/env python3
"""Capture raw namespace controls in the disposable pinned Linux reference VM."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import time

import vm


def observe(output):
    vm.verify_host()
    manifest = vm.manifest()
    kernel = vm.BUILD / 'arch/x86/boot/bzImage'
    vm.verify_file(kernel, 'sha256', manifest['inputs']['linux']['bzimage_sha256'])
    output.mkdir(parents=True)
    source = (vm.LAB / 'namespace_linux_guest.py').read_bytes()
    (output / 'guest.py').write_bytes(source)
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    command = vm.qemu_command('reference', port, 'file')
    serial = command.index('file,id=serial0,path=' + str(vm.state_paths()['serial']))
    command[serial] = 'file,id=serial0,path=' + str(output / 'serial.log')
    command.append('-snapshot')
    (output / 'command.json').write_text(json.dumps(command, indent=2) + '\n')
    with (output / 'qemu.log').open('wb') as log:
        process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 600
            with (output / 'ready.stdout.log').open('w') as stdout, (output / 'ready.stderr.log').open('w') as stderr:
                while True:
                    assert process.poll() is None, (output / 'qemu.log').read_text()
                    probe = vm.ssh_command(port, 'true', timeout=10)
                    stdout.write(probe.stdout); stdout.flush()
                    stderr.write(probe.stderr); stderr.flush()
                    if probe.returncode == 0:
                        break
                    assert time.monotonic() < deadline, 'reference SSH timeout'
                    time.sleep(2)
            key = vm.state_paths()['key']
            completed = subprocess.run(['ssh', '-i', str(key), '-p', str(port),
                '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=no',
                '-o', 'UserKnownHostsFile=/dev/null', '-o', 'LogLevel=ERROR',
                'topal@127.0.0.1', 'sudo env TOPAL_NAMESPACE_LINUX_GUEST=1 python3 -'],
                input=source, capture_output=True, timeout=180)
            (output / 'guest.stdout.log').write_bytes(completed.stdout)
            (output / 'guest.stderr.log').write_bytes(completed.stderr)
            assert completed.returncode == 0, (completed.stdout.decode(errors='replace'), completed.stderr.decode(errors='replace'))
            result = json.loads(completed.stdout)
            assert result['passed'] and result['kernel_release'] == manifest['inputs']['linux']['release']
            assert result['events'] and all(event['asserted_errno'] for event in result['events'])
            result.update(kernel_sha256=vm.digest(kernel, 'sha256'),
                guest_source_sha256=hashlib.sha256(source).hexdigest(),
                driver_sha256=vm.digest(Path(__file__), 'sha256'),
                vm_manifest_sha256=vm.digest(vm.MANIFEST_PATH, 'sha256'), qemu_snapshot=True,
                guest_stdout_sha256=vm.digest(output / 'guest.stdout.log', 'sha256'),
                guest_stderr_sha256=vm.digest(output / 'guest.stderr.log', 'sha256'))
            (output / 'observation.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
            print(f"NAMESPACE_LINUX_PASS cases={len(result['events'])} kernel={result['kernel_release']}", flush=True)
            return result
        finally:
            process.terminate()
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=15)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    observe(args.output.resolve())
