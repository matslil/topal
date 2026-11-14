"""Offline malformed diagnostic controls; no changed artifact is executed."""
import copy
import hashlib
import json
from pathlib import Path
import runpy
import sys
import traceback
from unittest.mock import patch

cache = Path('linux-kernel/.cache/x86_64')
catalog = json.loads((cache / 'staged-refusals-catalog.json').read_text())
build = Path(catalog['build'])
path = build / 'source-user-extents/observations.json'
native = json.loads(path.read_text())
reader = cache / 'readback-staged-refusals.py'
real_read = Path.read_text
controls = []
for label in ['protected-before-key','protected-after-key','syscall-result',
              'refusal-result','refusal-pointer','counterpart-preservation']:
    diagnostic = copy.deepcopy(native)
    if label.startswith('protected-'):
        row = diagnostic['shell_executions'][0]['calls'][0]
        data = bytearray.fromhex(row['protected_before'])
        data[0 if label == 'protected-before-key' else 72] ^= 1
        row['protected_before'] = data.hex()
    elif label == 'syscall-result':
        call = next(c for row in diagnostic['executions'] for c in row['calls'] if c['result'] == -14)
        call['result'] = 0
    else:
        event = diagnostic['staged_transfer_observation']['source_syscall_refusals'][0]
        if label == 'refusal-result': event['result'] = -22
        elif label == 'refusal-pointer': event['candidate'] ^= 1
        else: event['counterpart_unchanged'] = False
    def read(candidate, *args, **kwargs):
        return json.dumps(diagnostic) if candidate == path else real_read(candidate,*args,**kwargs)
    sys.argv = [str(reader),str(build),str(cache/'user-extent-source-2264433-1794396085975218237'),
        str(cache/'user-extent-source-storage-2296061-1794411555659865899'),str(cache/'unused-control-result.json')]
    try:
        with patch.object(Path,'read_text',read):
            runpy.run_path(str(reader),run_name='__main__')
    except AssertionError:
        controls.append(dict(label=label,rejected=True,exception=traceback.format_exc()))
    else:
        raise AssertionError(('malformed diagnostic accepted',label))
report = dict(passed=True,reader_sha256=hashlib.sha256(reader.read_bytes()).hexdigest(),controls=controls,
    scope='Six offline malformed observation controls with native ELF/image/guest state unchanged; not evidence from running malformed states')
(cache/'staged-refusal-readback-controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(controls))))
