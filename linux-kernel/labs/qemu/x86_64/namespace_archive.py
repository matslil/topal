#!/usr/bin/env python3
"""Archive a completed namespace cohort verbatim, deduplicating exact bytes."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import stat

import topal_gate as GATE

LAB=Path(__file__).resolve().parent
OPERATIONS=('link','linkat','move','replace','exchange')
MODES=('source-false','source-wrong-association','runtime-abort')
CONTROLS=('normal','runtime-abort','post-frame','post-control','source-false',
    'source-wrong-association','pre-owner','pre-view','pre-input','pre-generation',
    'pre-overlap-position','pre-overlap-page','pre-overlap-inode')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify(cohort,output,rust_record):
    record=json.loads((cohort/'cohort.json').read_text())
    assert record['schema']=='topal.retained-vm-cohort/1' and record['phase']=='namespace'
    assert record['complete'] and record['passed']
    expected={'normal','directory','fork','exec','cancel','utilities','clock-observe'}
    expected.update('fatal-'+mode+'-'+operation for mode in MODES for operation in OPERATIONS)
    expected.update('clock-error-'+operation for operation in OPERATIONS)
    expected.update('inode-admission-'+mode for mode in ('bad-link-count','overlap-namespace'))
    expected.update('controller-'+mode for mode in CONTROLS)
    assert len(record['steps'])==42 and {step['name'] for step in record['steps']}==expected
    vms=0
    for step in record['steps']:
        assert step['exit_code']==0
        for stream in ('stdout','stderr'):
            assert sha((cohort/step[stream]).read_bytes())==step[stream+'_sha256']
        vms+=sum(line.startswith(b'+ qemu-system-x86_64 ') for line in (cohort/step['stdout']).read_bytes().splitlines())
        retained=Path(step['output'])/'retention.json'
        assert sha(retained.read_bytes())==step['retention_sha256']
        state=json.loads(retained.read_text())
        assert state['complete'] and state['passed'] and state['head']==record['head']
    assert vms==47
    for entry in record['sources']:
        assert GATE.digest(GATE.REPOSITORY/entry['source'])==entry['sha256']
        assert GATE.digest(cohort/entry['snapshot'])==entry['sha256']
    rust=json.loads(rust_record.read_text())
    assert rust['complete'] and rust['passed'] and rust['head']==record['head']
    normal=json.loads((cohort/'n0/caller-observation.json').read_text())
    primary=json.loads((LAB/'results/topal-toolchain-gate.json').read_text())
    assert normal['result']=='pass' and normal['independent_calls']==465
    assert normal['kernel_sha256']==primary['linked_kernel_sha256']
    proofs=dict(normal='n0/caller-observation.json',directory='n1/caller-observation.json',
        fork='n2/fork-qualification.json',exec='n3/exec-qualification.json',
        cancel='n4/physical-observation.json',utilities='n5/utility-qualification.json')
    proofs.update({'fatal-'+mode+'-'+operation:'f'+str(mi)+str(oi)+'/physical-observation.json'
        for mi,mode in enumerate(MODES) for oi,operation in enumerate(OPERATIONS)})
    proofs['clock-observe']='c0/physical-observation.json'
    proofs.update({'clock-error-'+operation:'c'+str(i)+'/physical-observation.json'
        for i,operation in enumerate(OPERATIONS,1)})
    proofs.update({'inode-admission-'+mode:'g'+str(i)+'/physical-observation.json'
        for i,mode in enumerate(('bad-link-count','overlap-namespace'))})
    proofs.update({'controller-'+mode:'k'+str(i)+'/physical-observation.json'
        for i,mode in enumerate(CONTROLS)})
    assert set(proofs)==expected
    for name,path in proofs.items():
        data=json.loads((cohort/path).read_text())
        if not name.startswith('controller-'):
            assert data['result']=='pass',(name,data.get('result'))
        # Physical drivers bind the actual source, commands, loaded artifacts,
        # snapshots and debugger/serial logs. Verify each before archiving.
        for path,digest in data.get('hashes',{}).items():
            assert GATE.digest(Path(path))==digest,path
    output.mkdir(parents=True)
    blobs=output/'blobs'
    blobs.mkdir()
    files={}
    written=set()
    for path in sorted(cohort.rglob('*')):
        if not stat.S_ISREG(path.stat().st_mode):
            continue
        data=path.read_bytes()
        digest=sha(data)
        target=blobs/(digest+'.gz')
        if digest not in written:
            target.write_bytes(gzip.compress(data,mtime=0))
            assert gzip.decompress(target.read_bytes())==data
            written.add(digest)
        files[str(path.relative_to(cohort))]=dict(sha256=digest,bytes=len(data),
            blob=str(target.relative_to(output)))
    index=dict(schema='topal.namespace-verbatim-file-index/1',files=files)
    (output/'files.json').write_text(json.dumps(index,indent=2)+'\n')
    result=dict(schema='topal.namespace-machine-qualification.x86_64/1',result='pass',
        qualified_code_commit=record['head'],qualified_code_tree=rust['tree'],
        rust_record_sha256=GATE.digest(rust_record),primary_kernel_sha256=normal['kernel_sha256'],
        cohort_sha256=GATE.digest(cohort/'cohort.json'),
        archive_index_sha256=GATE.digest(output/'files.json'),
        source_snapshots=record['sources'],proofs=proofs,vm_observations=vms,
        steps=42,unique_blobs=len(written),archived_files=len(files),
        archiver_sha256=GATE.digest(Path(__file__)),
        scope='fresh committed shared-namespace source/admission/lifetime/utility qualification; complete distribution, containers, remaining UAPI and VM hosting remain required')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print('NAMESPACE_ARCHIVE_PASS files='+str(len(files))+' blobs='+str(len(written)),flush=True)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cohort',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--rust-record',required=True,type=Path)
    args=parser.parse_args()
    verify(args.cohort.resolve(),args.output.resolve(),args.rust_record.resolve())
