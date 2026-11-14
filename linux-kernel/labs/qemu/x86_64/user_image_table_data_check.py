"""Run the source-built inactive-data test through the ordinary serial gate.

The guest application owns its heap byte buffer. Its numeric address inputs do
not designate live tables. The initializer writes application data; the kernel
follows its existing normal translation lifecycle. The test uses no debugger,
host executable mapping or physical-memory access.
"""
import hashlib
import json
from pathlib import Path
import sys

import topal_gate

MARKER = b'TOPAL_IMAGE_TABLE_DATA_OK\n'
FAILURE = b'TOPAL_IMAGE_TABLE_DATA_FAIL:'


def main():
    work = Path(sys.argv[1]).resolve()
    assert work.is_relative_to(topal_gate.LINUX_KERNEL.resolve())
    group = next(line.split(':', 2)[2] for line in
                 Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    resources = Path('/sys/fs/cgroup') / group.lstrip('/')
    assert 'topal-bounded-' in group
    assert (resources / 'memory.swap.max').read_text().strip() == '0'
    source = json.loads((work / 'source-component.json').read_text())
    assert source['image_sha256'] == topal_gate.digest(work / 'static-elf-abi-fixture.elf')
    binding = json.loads((work / 'private-image-binding.json').read_text())
    assert binding['production_qualified'] is False and binding['protected_context'] is False
    assert binding['layout']['kernel_sha256'] == topal_gate.digest(work / 'kernel.elf')
    result = dict(passed=False, native_execution=False, native_execution_attempted=False, protected_context=False,
                  scope='Ordinary user-mode execution of an inactive byte initializer; no live page tables, owner authority, publication or effective hardware protection',
                  source=source)
    try:
        result['qemu_version'] = topal_gate.verify_host()
        result['native_execution_attempted'] = True

        def source_failure():
            serial = work / 'serial.log'
            if serial.exists() and FAILURE in serial.read_bytes():
                return 'ordinary source buffer-preparation check failed; see serial.log'
            return None

        serial = topal_gate.observe_gate(work / 'bzImage', work / 'initramfs.cpio',
                                        work / 'serial.log', work / 'qemu.log', 120,
                                        failure_check=source_failure)
        completed = serial.count(MARKER)
        assert completed == 4, 'initial image and three replacements must each complete the source checks'
        result['native_execution'] = True
        # Preserve the complete original serial log. Remove only this test's
        # extension marker for the unchanged baseline serial-contract verifier.
        topal_gate.verify_process_serial(serial.replace(MARKER, b''), external=False,
                                          process_workload='true')
        result.update(passed=True, completed_calls=completed,
                      valid_requests=completed * source['valid_requests_per_call'],
                      refused_requests=completed * source['refused_requests_per_call'],
                      serial_sha256=hashlib.sha256(serial).hexdigest(),
                      kernel_sha256=topal_gate.digest(work / 'kernel.elf'),
                      private_binding_sha256=topal_gate.digest(work / 'private-image-binding.json'),
                      boot_image_sha256=topal_gate.digest(work / 'bzImage'))
    finally:
        result['resources'] = {name: (resources / name).read_text().strip() for name in
                               ['memory.max', 'memory.swap.max', 'memory.peak', 'memory.events']}
        (work / 'observations.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({name: result[name] for name in
                      ['passed', 'completed_calls', 'valid_requests', 'refused_requests', 'protected_context']}))


if __name__ == '__main__':
    main()
