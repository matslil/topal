"""Retain exact modified-source inputs for the first stat observer failure."""
import gzip
import hashlib
import json
from pathlib import Path

cache = Path('linux-kernel/.cache/x86_64')
run = cache/'tk283-staging-focused-1794448282205895303'
root = cache/'stat-first-reach-failure-inputs'
root.mkdir(exist_ok=False)
record = json.loads((run/'run.json').read_text())
assert record['exit_code'] != 0 and not record['source_changes_during_run']
digest = lambda data:hashlib.sha256(data).hexdigest()
current = '''        record = dict(offset=offset, target=self.row['target'],
            candidate=reg('r8'), length=reg('r9'), phase=memory(context+104,1)[0])
        if offset in extent_first_reach:
            # A source-scoped admission observer can record this same stopped
            # instruction first. Both read-only observations must agree.
            assert extent_first_reach[offset] == record
        else:
            extent_first_reach[offset] = record
'''
previous = '''        assert offset not in extent_first_reach
        extent_first_reach[offset] = dict(offset=offset, target=self.row['target'],
            candidate=reg('r8'), length=reg('r9'), phase=memory(context+104,1)[0])
'''
inputs = {}
for name,sha in record['modified_source_hashes_before'].items():
    data = Path(name).read_bytes()
    if name.endswith('/user_extent_call_coverage.py'):
        text = data.decode()
        assert text.count(current) == 1
        data = text.replace(current,previous,1).encode()
    assert digest(data) == sha,name
    snapshot = root/(name+'.gz')
    snapshot.parent.mkdir(parents=True,exist_ok=True)
    snapshot.write_bytes(gzip.compress(data,mtime=0))
    inputs[name] = dict(sha256=sha,snapshot=str(snapshot.relative_to(root)))
(root/'identity.json').write_text(json.dumps(dict(passed=True,terminal_status='failed',
    inputs=inputs,scope='Exact modified-source readback for failed first-reach observer bookkeeping; one source is reverse-derived and required to match its actual captured pre/post SHA. Not a passing native execution'),indent=2)+'\n')
print(json.dumps(dict(retained_failed_inputs=len(inputs),output=str(root))))
