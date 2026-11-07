#!/usr/bin/env python3
"""Run unchanged, pinned BusyBox chmod/stat/shell workloads on one kernel."""
import argparse
import json
from pathlib import Path
import tempfile

import topal_gate as GATE


def verify(root, busybox, timeout):
    root.mkdir(parents=True, exist_ok=True)
    GATE.checked_static_user_image(busybox)
    GATE.verify_host()
    values = GATE.build_image(root/'artifact', busybox)
    build = GATE.build_image
    observe = GATE.observe_gate
    records = {}
    names = ('mode-numeric', 'mode-symbolic', 'mode-directory',
             'mode-retained-reader', 'mode-umask-independent')
    try:
        GATE.build_image = lambda *_a, **_k: values
        for name in names:
            case = root/name
            case.mkdir()

            def copied_observation(image, resource, _serial, _qemu, *args, **kwargs):
                return observe(image, resource, case/'serial.log', case/'qemu.log', *args, **kwargs)

            GATE.observe_gate = copied_observation
            record = GATE.verify(timeout, False, busybox, name)
            assert record['linked_kernel_sha256'] == GATE.digest(values[3].parent/'kernel.elf')
            (case/'evidence.json').write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
            records[name] = record
            print('PASS unchanged BusyBox '+name, flush=True)
    finally:
        GATE.build_image = build
        GATE.observe_gate = observe
    result = dict(schema='topal-inode-mode-busybox/1', result='pass', workloads=records,
        harness_sha256=GATE.digest(Path(__file__)), gate_sha256=GATE.digest(Path(GATE.__file__)),
        busybox_sha256=GATE.digest(busybox))
    (root/'utility-observation.json').write_text(json.dumps(result, indent=2, sort_keys=True)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--busybox', type=Path, default=Path('/usr/bin/busybox'))
    parser.add_argument('--keep', type=Path)
    parser.add_argument('--timeout', type=float, default=120)
    args = parser.parse_args()
    if args.keep:
        verify(args.keep.resolve(), args.busybox.resolve(), args.timeout)
    else:
        with tempfile.TemporaryDirectory(prefix='topal-inode-mode-busybox-') as directory:
            verify(Path(directory), args.busybox.resolve(), args.timeout)
