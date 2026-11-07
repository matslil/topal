#!/usr/bin/env python3
"""Repeat all inode machine oracles with one frozen driver and fresh outputs."""
import argparse
import json
from pathlib import Path

import inode_mode_gate as MODE


def existing(root, lifetime=False):
    root = root.resolve()
    image, calls = MODE.CHECKS.fixture(MODE.GATE.static_user_fixture(), lifetime)
    values = (root/'boot/bzImage', root/'boot/initramfs.cpio', root/'boot/boot-provenance.json',
        root/'linked/provenance.json', root/'generated/initial-user.elf', root/'linked/static-user.elf')
    assert values[5].read_bytes() == image, 'existing caller does not match this source'
    return values, calls


def native_inputs(values):
    artifact = json.loads(values[3].read_text())
    entries = {e['identity']: e['sha256'] for e in artifact['inputs']
        if e['identity'].startswith(('topal.provider-object.', 'topal.systems-root-object.'))}
    assert len(entries) == 2
    return entries


def verify(args):
    root = args.output.resolve()
    root.mkdir(parents=True)
    MODE.GATE.verify_host()
    baseline, calls = existing(args.primary)
    lifetime, lifetime_calls = existing(args.lifetime, True)
    kernel = MODE.GATE.digest(baseline[3].parent/'kernel.elf')
    assert MODE.GATE.digest(lifetime[3].parent/'kernel.elf') == kernel
    records = {}
    records['primary'] = MODE.inspect(root/'primary', baseline, calls, args.timeout)
    records['lifetime'] = MODE.inspect(root/'lifetime', lifetime, lifetime_calls, args.timeout)
    records['inactive_terminal'] = MODE.inspect(root/'inactive-terminal', lifetime, lifetime_calls,
        args.timeout, terminate_inactive=True)
    for fault in ('foreign-owner', 'record-stride', 'interrupts-enabled', 'permissions-range',
                  'dirty-arena', 'generation-exhausted', 'bad-type'):
        records[fault] = MODE.inspect(root/fault, baseline, calls, args.timeout, fault=fault)
    for name, path in [('false-after-mode', args.false_after_mode), ('wrong-mode', args.wrong_mode)]:
        values, altered_calls = existing(path)
        assert native_inputs(values) == native_inputs(baseline), 'source control changed the native provider'
        assert MODE.GATE.digest(values[3].parent/'kernel.elf') != kernel
        records[name] = MODE.inspect(root/name, values, altered_calls, args.timeout, bad_action=name)
        records[name]['native_root_and_provider_unchanged'] = True
    record = dict(schema='topal-inode-mode-qualification/1', result='pass', cases=records,
        linked_kernel_sha256=kernel,
        suite_sha256=MODE.GATE.digest(Path(__file__)))
    (root/'qualification.json').write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    print(f'PASS all {len(records)} inode physical dispositions', flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--primary', type=Path, required=True)
    parser.add_argument('--lifetime', type=Path, required=True)
    parser.add_argument('--false-after-mode', type=Path, required=True)
    parser.add_argument('--wrong-mode', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--timeout', type=float, default=180)
    verify(parser.parse_args())
