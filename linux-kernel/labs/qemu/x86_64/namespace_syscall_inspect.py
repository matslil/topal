"""Observe actual user syscall returns; no injection or kernel-state changes."""
import hashlib
import json
from pathlib import Path
import traceback

import gdb


def install(config):
    output = Path(config['output'])
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == config['inspector_sha256']
    seen = []
    filesystem_snapshots = {}
    physical = None
    if config.get('physical_sha256'):
        import namespace_syscall_physical
        physical = namespace_syscall_physical.Observer(config)

    class Returned(gdb.Breakpoint):
        def __init__(self, case):
            self.case = case
            super().__init__('*' + hex(case['returned']), internal=True)

        def stop(self):
            try:
                result = int(gdb.parse_and_eval('$rax')) & ((1 << 64) - 1)
                assert int(gdb.parse_and_eval('$cs')) & 3 == 3, 'caller must return in user mode'
                assert result == self.case['expected'] & ((1 << 64) - 1), (self.case, result)
                if 'namespace_epoch' in self.case:
                    history = next(s for s in config['artifact']['namespace_runtime']['invocation']['storage']
                                   if s['symbol'].endswith('_history'))
                    actual = int.from_bytes(gdb.selected_inferior().read_memory(history['address'],8),'little')
                    assert actual == self.case['namespace_epoch'], ('namespace epoch checkpoint',self.case,actual)
                if 'filesystem_checkpoint' in self.case:
                    owner = config['artifact']['namespace_runtime']['invocation']['controller_owner']['address']
                    profile = config['artifact']['volatile_files']
                    value = bytes(gdb.selected_inferior().read_memory(owner+profile['filesystem_offset'],profile['filesystem_bytes']))
                    name,action = self.case['filesystem_checkpoint']
                    if action == 'save':
                        assert name not in filesystem_snapshots
                        filesystem_snapshots[name] = value
                    else:
                        assert action == 'compare' and filesystem_snapshots[name] == value, 'exec changed the shared filesystem'
                    (output/('filesystem-'+name+'-'+action+'.bin')).write_bytes(value)
                if physical:
                    physical.returned(self.case)
                seen.append(dict(self.case, actual=result))
                (output / 'syscall-returns.json').write_text(json.dumps(seen, indent=2) + '\n')
                self.enabled = False
                # Detach successfully after the last observed return. The
                # independent caller then finishes its own assertions before
                # the serial completion markers can be accepted.
                return len(seen) == len(config['cases'])
            except BaseException:
                (output / 'failed.txt').write_text(traceback.format_exc())
                return True

    for case in config['cases']:
        Returned(case)
        if physical:
            physical.watch(case)
