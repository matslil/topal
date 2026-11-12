#!/usr/bin/env python3
"""Replay archived process/MM observations from frozen inputs; no native run."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess


def checked(raw, fact):
    if len(raw)!=fact['bytes'] or hashlib.sha256(raw).hexdigest()!=fact['sha256']:
        raise ValueError('archive identity differs')
    return raw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive',type=Path)
    parser.add_argument('output',type=Path)
    args = parser.parse_args()
    archive, output = args.archive.resolve(), args.output.resolve()
    root = Path(__file__).resolve().parents[3]
    if not output.is_relative_to(root/'linux-kernel'):
        raise ValueError('Linux replay directory')
    manifest = json.loads((archive/'manifest.json').read_text())
    if {p.relative_to(archive).as_posix() for p in archive.rglob('*') if p.is_file()} != set(manifest)|{'manifest.json'}:
        raise ValueError('missing or extra archive files')
    for name,fact in manifest.items():
        target=(archive/name).resolve()
        if not target.is_relative_to(archive):
            raise ValueError('archive path escapes root')
        checked(target.read_bytes(),fact)
    report=json.loads((archive/'report.json').read_text())
    if not report['passed']:
        raise ValueError('incomplete native archive')
    output.mkdir(parents=True,exist_ok=False)
    for name,fact in report['selected_inputs'].items():
        target=(output/name).resolve()
        if not target.is_relative_to(output):
            raise ValueError('selected path escapes root')
        raw=checked(gzip.decompress((archive/('inputs/'+name+'.gz')).read_bytes()),fact)
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(raw)
    raw=checked(gzip.decompress((archive/'native-observations.json.gz').read_bytes()),report['observation'])
    (output/'observations.json').write_bytes(raw)
    (output/'expected.json').write_text(json.dumps(report))
    script='''import json,sys
from pathlib import Path
sys.path.insert(0,'linux-kernel/tools/process-memory')
from readback_process import readback
actual=json.loads(json.dumps(readback(json.loads(Path('observations.json').read_text()))))
expected=json.loads(Path('expected.json').read_text())
assert all(actual[k]==expected[k] for k in actual)
Path('replayed.json').write_text(json.dumps(actual,indent=2,sort_keys=True)+'\\n')
print(json.dumps(dict(passed=True,cases=actual['cases'],controls=len(actual['controls']),operations=actual['operations'])))
'''
    subprocess.run(['rtk','proxy','python3','-c',script],cwd=output,check=True)
    print('Frozen process/MM saved-record replay passed:',output)


if __name__=='__main__':
    main()
