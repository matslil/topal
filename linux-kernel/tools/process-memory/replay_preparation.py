#!/usr/bin/env python3
"""Replay archived native records from their selected frozen Python inputs.

No guest or native rebuild. This does not re-establish artifact/load authority.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    archive, output = args.archive.resolve(), args.output.resolve()
    root = Path(__file__).resolve().parents[3]
    if not output.is_relative_to(root / 'linux-kernel'):
        raise ValueError('keep Linux replay under linux-kernel')
    manifest = json.loads((archive / 'manifest.json').read_text())
    expected = set(manifest) | {'manifest.json'}
    actual = {p.relative_to(archive).as_posix() for p in archive.rglob('*') if p.is_file()}
    if actual != expected:
        raise ValueError('extra or missing archive files')
    for name, fact in manifest.items():
        raw = (archive / name).read_bytes()
        if len(raw) != fact['bytes'] or hashlib.sha256(raw).hexdigest() != fact['sha256']:
            raise ValueError('archive identity differs: ' + name)
    report = json.loads((archive / 'report.json').read_text())
    if not report['passed']:
        raise ValueError('incomplete native archive')
    output.mkdir(parents=True, exist_ok=False)
    for source, fact in report['selected_inputs'].items():
        raw = gzip.decompress((archive / ('inputs/' + source + '.gz')).read_bytes())
        if len(raw) != fact['bytes'] or hashlib.sha256(raw).hexdigest() != fact['sha256']:
            raise ValueError('selected input differs: ' + source)
        target = (output / source).resolve()
        if not target.is_relative_to(output):
            raise ValueError('input path escapes frozen root')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
    raw = gzip.decompress((archive / 'native-observations.json.gz').read_bytes())
    if hashlib.sha256(raw).hexdigest() != report['observation']['sha256']:
        raise ValueError('observation identity differs')
    (output / 'observations.json').write_bytes(raw)
    (output / 'expected.json').write_text(json.dumps(report))
    for name in ['target', 'dispatch']:
        raw = gzip.decompress((archive / ('generated-' + name + '.ll.gz')).read_bytes())
        if hashlib.sha256(raw).hexdigest() != report['artifacts'][name + '.ll']['sha256']:
            raise ValueError('generated native-source identity differs')
    script = '''import json,sys
from pathlib import Path
sys.path.insert(0,'linux-kernel/tools/process-memory')
from readback_preparation import readback
actual=json.loads(json.dumps(readback(json.loads(Path('observations.json').read_text()))))
expected=json.loads(Path('expected.json').read_text())
assert all(actual[k]==expected[k] for k in actual)
Path('replayed.json').write_text(json.dumps(actual,indent=2,sort_keys=True)+'\\n')
print(json.dumps(dict(passed=True,preparations=actual['preparations'],controls=len(actual['controls']),metadata_syscalls=actual['metadata_syscalls'])))
'''
    subprocess.run(['rtk', 'proxy', 'python3', '-c', script], cwd=output, check=True)
    print('Frozen saved-record replay passed:', output)


if __name__ == '__main__':
    main()
