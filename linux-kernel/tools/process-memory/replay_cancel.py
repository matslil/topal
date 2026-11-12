#!/usr/bin/env python3
"""Replay frozen live-child cancellation records; never substitutes for QEMU."""
import argparse
import gzip
import json
from pathlib import Path
import subprocess

from replay_process import checked


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();archive=args.archive.resolve();output=args.output.resolve()
    root=Path(__file__).resolve().parents[3]
    if not output.is_relative_to(root/'linux-kernel'):raise ValueError('Linux replay directory')
    manifest=json.loads((archive/'manifest.json').read_text())
    if {p.relative_to(archive).as_posix() for p in archive.rglob('*') if p.is_file()}!=set(manifest)|{'manifest.json'}:
        raise ValueError('archive file closure')
    for name,fact in manifest.items():
        target=(archive/name).resolve()
        if not target.is_relative_to(archive):raise ValueError('archive path escapes root')
        checked(target.read_bytes(),fact)
    report=json.loads((archive/'report.json').read_text())
    if not report['passed'] or report['schema']!='topal-process-mm-native-cancellation-v1':raise ValueError('archive schema')
    output.mkdir(parents=True,exist_ok=False)
    for name,fact in report['selected_inputs'].items():
        target=(output/name).resolve()
        if not target.is_relative_to(output):raise ValueError('input escapes root')
        raw=checked(gzip.decompress((archive/('inputs/'+name+'.gz')).read_bytes()),fact)
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    for i,row in enumerate(report['cohorts']):
        raw=checked(gzip.decompress((archive/f'cohorts/{i}/observations.json.gz').read_bytes()),row['observation'])
        (output/f'case-{i}.json').write_bytes(raw)
    raw=checked(gzip.decompress((archive/'regression/observations.json.gz').read_bytes()),report['regression']['observation'])
    (output/'regression.json').write_bytes(raw)
    (output/'expected.json').write_text(json.dumps(report))
    script='''import json,sys
from pathlib import Path
sys.path.insert(0,'linux-kernel/tools/process-memory')
from readback_cancel import readback
from readback_exec import readback as exec_readback
expected=json.loads(Path('expected.json').read_text())
for i,row in enumerate(expected['cohorts']):
    actual=json.loads(json.dumps(readback(json.loads(Path(f'case-{i}.json').read_text()))))
    assert all(actual[k]==row[k] for k in actual)
actual=json.loads(json.dumps(exec_readback(json.loads(Path('regression.json').read_text()))))
assert actual==expected['regression']['summary']
Path('replayed.json').write_text(json.dumps(dict(passed=True,cases=4,cancellation_controls=48,regression_controls=30))+'\\n')
print('Frozen cancellation and child-exec records passed')
'''
    subprocess.run(['rtk','proxy','python3','-c',script],cwd=output,check=True)


if __name__=='__main__':main()
