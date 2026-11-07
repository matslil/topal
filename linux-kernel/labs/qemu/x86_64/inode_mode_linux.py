#!/usr/bin/env python3
"""Execute raw chmod controls under the manifest-pinned Linux reference kernel."""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import time

import vm


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'fresh output required'
    vm.verify_host()
    manifest = vm.manifest()
    kernel = vm.BUILD / 'arch/x86/boot/bzImage'
    vm.verify_file(kernel, 'sha256', manifest['inputs']['linux']['bzimage_sha256'])
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    source = (vm.LAB / 'inode_mode_linux_guest.py').read_text()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='topal-inode-linux-') as temporary:
        work = Path(temporary)
        command = vm.qemu_command('reference', port, 'file')
        serial = command.index('file,id=serial0,path=' + str(vm.state_paths()['serial']))
        command[serial] = 'file,id=serial0,path=' + str(work / 'serial.log')
        command.append('-snapshot')
        with (work / 'qemu.log').open('wb') as log:
            process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 600
                while True:
                    assert process.poll() is None, (work / 'qemu.log').read_text()
                    probe = vm.ssh_command(port, 'true', timeout=10)
                    if probe.returncode == 0:
                        break
                    assert time.monotonic() < deadline, 'reference SSH timeout'
                    time.sleep(2)
                key = vm.state_paths()['key']
                completed = subprocess.run(['ssh', '-i', str(key), '-p', str(port),
                    '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=no',
                    '-o', 'UserKnownHostsFile=/dev/null', '-o', 'LogLevel=ERROR',
                    'topal@127.0.0.1', 'sudo python3 -'], input=source, text=True,
                    capture_output=True, timeout=180)
                assert completed.returncode == 0, (completed.stdout, completed.stderr)
                result = json.loads(completed.stdout)
                assert result['passed'] and result['kernel_release'] == '7.2.9'
                result['kernel_sha256'] = vm.digest(kernel, 'sha256')
                result['guest_source_sha256'] = hashlib.sha256(source.encode()).hexdigest()
                result['driver_sha256'] = vm.digest(Path(__file__), 'sha256')
                result['vm_manifest_sha256'] = vm.digest(vm.MANIFEST_PATH, 'sha256')
                result['qemu_snapshot'] = True
                args.output.write_text(json.dumps(result, indent=2) + '\n')
                print(f"INODE_LINUX_PASS cases={len(result['events'])} kernel={result['kernel_release']}", flush=True)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=15)


if __name__ == '__main__':
    main()
