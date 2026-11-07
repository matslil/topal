"""Bind post-run verifier corrections without relabeling experiment inputs."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
PREFIX='linux-kernel/labs/qemu/x86_64/'
ALLOWED={PREFIX+'test_namespace_gate.py':'publication contract',
    PREFIX+'test_topal_gate.py':'publication contract',
    PREFIX+'qualification_archive.py':'post-run archive verifier'}


def sha(data): return hashlib.sha256(data).hexdigest()


def source_set(entries):
    return sha(json.dumps(sorted((e['source'],e['sha256']) for e in entries),separators=(',',':')).encode())


def validate(entries,repository,read_snapshot,manifest_path):
    manifest=json.loads(manifest_path.read_text())
    assert manifest['schema']=='topal.post-run-verification-revision/1'
    assert manifest['experiment_source_set_sha256']==source_set(entries)
    assert manifest['verifier_sha256']==sha(Path(__file__).read_bytes())
    base=manifest_path.parent
    assert sha((base/'verification-sources/verification_revision.py').read_bytes())==manifest['verifier_sha256']
    changes=manifest['changes']
    assert set(changes)<=set(ALLOWED)
    changed=set()
    for entry in entries:
        path=entry['source']
        assert sha(read_snapshot(entry['snapshot']))==entry['sha256'],path
        current=sha((repository/path).read_bytes())
        if path in changes:
            change=changes[path]
            assert change['role']==ALLOWED[path]
            assert change['experiment_sha256']==entry['sha256']
            assert current==change['verification_sha256'] and current!=entry['sha256']
            assert sha((base/change['verification_snapshot']).read_bytes())==current
            changed.add(path)
        else:
            assert current==entry['sha256'],('experiment input changed',path)
    assert changed==set(changes)
    return manifest


def create(cohort,archive,failed_log):
    record=json.loads((cohort/'cohort.json').read_text())
    assert record['complete'] and record['passed']
    assert not (archive/'verification-revision.json').exists()
    directory=archive/'verification-sources'
    directory.mkdir()
    changes={}
    for entry in record['sources']:
        original=(cohort/entry['snapshot']).read_bytes()
        assert sha(original)==entry['sha256']
        current=(ROOT/entry['source']).read_bytes()
        if sha(current)==entry['sha256']: continue
        assert entry['source'] in ALLOWED,('runtime input changed',entry['source'])
        target=directory/Path(entry['source']).name
        target.write_bytes(current)
        changes[entry['source']]=dict(role=ALLOWED[entry['source']],
            experiment_sha256=entry['sha256'],verification_sha256=sha(current),
            verification_snapshot=str(target.relative_to(archive)))
    assert changes
    (directory/'verification_revision.py').write_bytes(Path(__file__).read_bytes())
    (archive/'verification-failures.log').write_bytes(failed_log.read_bytes())
    manifest=dict(schema='topal.post-run-verification-revision/1',
        qualified_code_commit=record['head'],origin_cohort_sha256=sha((cohort/'cohort.json').read_bytes()),
        experiment_source_set_sha256=source_set(record['sources']),changes=changes,
        failed_verification_log_sha256=sha(failed_log.read_bytes()),
        verifier_sha256=sha(Path(__file__).read_bytes()),
        reason='Correct exact V6 publication schemas and distinguish unentered source frames from admitted clock/source frames; preserve all experiment inputs and outcomes')
    path=archive/'verification-revision.json'
    path.write_text(json.dumps(manifest,indent=2)+'\n')
    validate(record['sources'],ROOT,lambda name:(cohort/name).read_bytes(),path)
    print('VERIFICATION_REVISION_BOUND files='+str(len(changes)),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('cohort','archive','failed-log'): parser.add_argument('--'+name,type=Path,required=True)
    args=parser.parse_args()
    create(args.cohort.resolve(),args.archive.resolve(),args.failed_log.resolve())
