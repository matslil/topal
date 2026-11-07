#!/usr/bin/env python3
"""Reconcile completed VM cohorts and retain their exact artifacts and logs."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import stat
import subprocess
import sys

import topal_gate as GATE

LAB=Path(__file__).resolve().parent
sys.path.insert(0,str(GATE.LINUX_KERNEL/'tools/archive-publication'))
import verification_revision
BASELINE_COUNTS=dict(ram_page=21,file_region=13,region=9,positional=4,named=4,
    file_open=5,volatile=4,header=4,dispatch=9,identity=5,retained=5,structured=6,
    numeric=4,library=3,realtime=5,time=3,children=3,pair=4,pipes=4,deadline=2)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify(args):
    rust_path=GATE.LINUX_KERNEL/'tools/rust-qualification/results/tk281/final/qualification.json'
    rust=json.loads(rust_path.read_text())
    assert rust['complete'] and rust['passed']
    head=rust['head']
    bounded_sha=GATE.digest(GATE.REPOSITORY/'scripts/run_bounded.py')
    assert rust['bounded_wrapper_sha256']==bounded_sha
    primary=json.loads((LAB/'results/topal-toolchain-gate.json').read_text())
    namespace_archive=LAB/'results/tk281/namespace-consumer'
    namespace=json.loads((namespace_archive/'qualification.json').read_text())
    inode_archive=LAB/'results/tk281/inode-consumer'
    inode=json.loads((inode_archive/'archive.json').read_text())
    assert namespace['qualified_code_commit']==inode['qualified_code_commit']==head
    assert namespace['rust_record_sha256']==inode['rust_record_sha256']==GATE.digest(rust_path)
    assert namespace['primary_kernel_sha256']==primary['linked_kernel_sha256']
    commands=[]
    totals={}
    roots=dict(baseline=args.baseline.resolve(),corpus=args.corpus.resolve(),
        namespace=args.namespace.resolve(),inode=args.inode.resolve(),
        inode_command=args.inode_command.resolve())
    for name in ('baseline','corpus','namespace'):
        root=roots[name]
        record=json.loads((root/'cohort.json').read_text())
        assert record['complete'] and record['passed'] and record['head']==head
        assert record['phase']==name
        assert record['bounded_wrapper_sha256']==bounded_sha
        assert record['harness_sha256']==GATE.digest(LAB/'qualification_cohort.py')
        assert record['retention_sha256']==GATE.digest(LAB/'qualification_gate.py')
        if name=='baseline':
            assert {entry['name'] for entry in record['steps']}==set(BASELINE_COUNTS)
        verification_revision.validate(record['sources'],GATE.REPOSITORY,
            lambda name:(root/name).read_bytes(),namespace_archive/'verification-revision.json')
        count=0
        for step in record['steps']:
            assert step['exit_code']==0
            stdout=(root/step['stdout']).read_bytes()
            stderr=(root/step['stderr']).read_bytes()
            assert sha(stdout)==step['stdout_sha256'] and sha(stderr)==step['stderr_sha256']
            retained=Path(step['output'])/'retention.json'
            assert retained.resolve().is_relative_to(root)
            assert GATE.digest(retained)==step['retention_sha256']
            state=json.loads(retained.read_text())
            assert state['complete'] and state['passed'] and state['exit_code']==0
            assert (state['head'],state['tree'])==(head,rust['tree'])
            assert state['wrapper_sha256']==record['retention_sha256']
            assert state['gate_sha256']==GATE.digest(GATE.REPOSITORY/state['gate'])
            vms=sum(line.startswith(b'+ qemu-system-x86_64 ') for line in stdout.splitlines())
            if name=='baseline':
                assert vms==BASELINE_COUNTS[step['name']],(step['name'],vms)
            count+=vms
            commands.append(dict(cohort=name,name=step['name'],vm_observations=vms,
                stdout=name+'/'+step['stdout'],stderr=name+'/'+step['stderr'],
                stdout_sha256=sha(stdout),stderr_sha256=sha(stderr)))
        totals[name]=count
    assert totals==dict(baseline=117,corpus=54,namespace=47)
    physical=json.loads((roots['inode']/'physical/qualification.json').read_text())
    assert physical['result']=='pass' and len(physical['cases'])==12
    assert physical['linked_kernel_sha256']==primary['linked_kernel_sha256']
    command=json.loads((roots['inode_command']/'retention.json').read_text())
    assert command['complete'] and command['passed'] and command['head']==head
    assert command['tree']==rust['tree'] and command['exit_code']==0
    assert command['wrapper_sha256']==GATE.digest(LAB/'qualification_gate.py')
    assert command['gate_sha256']==GATE.digest(LAB/'inode_mode_refresh.py')
    assert GATE.digest(roots['inode_command']/'retention.json')==inode['command_record_sha256']
    totals['inode']=len(physical['cases'])
    assert sum(totals.values())==230
    corpus=json.loads((roots['corpus']/'corpus/qualification.json').read_text())
    assert corpus['result']=='pass' and len(corpus['workloads'])==51
    assert set(corpus['workloads'])==set(GATE.PROCESS_WORKLOADS)
    assert corpus['busybox_sha256']==GATE.PINNED_STATIC_BUSYBOX_SHA256
    assert corpus['primary_kernel_sha256']==primary['linked_kernel_sha256']
    assert GATE.digest(roots['corpus']/'corpus/qualification.json')==inode['corpus_record_sha256']
    executions_path=roots['corpus']/'corpus/execution-logs.json'
    assert GATE.digest(executions_path)==corpus['execution_logs_sha256']
    executions=json.loads(executions_path.read_text())
    for name,entry in corpus['workloads'].items():
        path=roots['corpus']/'corpus'/entry['path']
        assert GATE.digest(path)==entry['sha256']
        workload=json.loads(path.read_text())
        assert workload['observations']['result']=='pass'
        assert workload['linked_kernel_sha256']==primary['linked_kernel_sha256']
        assert workload['static_user_image_sha256']==corpus['busybox_sha256']
        assert workload['process_output_utf8'].encode()==GATE.PROCESS_WORKLOADS[name][1]
        for stream,key in (('serial','serial_sha256'),('qemu_log','qemu_log_sha256')):
            log=Path(executions[name][stream])
            assert log.resolve().is_relative_to(roots['corpus'])
            assert GATE.digest(log)==executions[name][key]
        assert executions[name]['serial_sha256']==workload['observations']['serial_sha256']
    changed=subprocess.check_output(['git','diff','--name-only'],cwd=GATE.REPOSITORY,text=True).splitlines()
    assert not any(path.startswith(('src/','spec/','docs/','linux-kernel/kernel/'))
        or path.endswith('.rs') for path in changed),'production or Rust source changed after qualification'
    contracts=args.contracts.resolve().read_bytes()
    assert b'Ran 45 tests in ' in contracts and contracts.rstrip().endswith(b'OK')
    output=args.output.resolve()
    output.mkdir(parents=True)
    blobs=output/'blobs'
    blobs.mkdir()
    files={}
    written=set()

    def store(path,key):
        data=path.read_bytes()
        digest=sha(data)
        target=blobs/(digest+'.gz')
        if digest not in written:
            target.write_bytes(gzip.compress(data,mtime=0))
            assert gzip.decompress(target.read_bytes())==data
            written.add(digest)
        files[key]=dict(sha256=digest,bytes=len(data),blob=str(target.relative_to(output)))

    # Namespace already has a complete, separately tested blob archive. Other
    # cohorts retain every regular artifact here; equal bytes share one blob.
    for name in ('baseline','corpus','inode','inode_command'):
        root=roots[name]
        for path in sorted(root.rglob('*')):
            if not stat.S_ISREG(path.stat().st_mode):
                continue
            assert not path.is_symlink() or path.resolve().is_relative_to(root)
            store(path,name+'/'+str(path.relative_to(root)))
    store(args.contracts.resolve(),'lab-publication-contracts.log')
    for path in sorted(LAB.glob('results/topal-*.json')):
        store(path,'publication/'+path.name)
    for path in (Path(__file__),LAB/'qualification_gate.py',LAB/'qualification_cohort.py',
                 LAB/'namespace_archive.py',LAB/'inode_mode_archive.py',LAB/'workload_gate.py',
                 LAB/'inode_mode_refresh.py'):
        store(path,'runner-sources/'+path.name)
    (output/'files.json').write_text(json.dumps(dict(schema='topal.full-vm-verbatim-index/1',
        files=files),indent=2)+'\n')
    result=dict(schema='topal.namespace-full-affected-qualification.x86_64/1',
        qualified_code_commit=head,qualified_code_tree=rust['tree'],
        primary_kernel_sha256=primary['linked_kernel_sha256'],
        vm_observations=totals,total_vm_observations=sum(totals.values()),
        unchanged_busybox_workloads=51,lab_publication_contracts=45,
        lab_contract_log_sha256=sha(contracts),commands=commands,
        rust_record_sha256=GATE.digest(rust_path),
        namespace_record_sha256=GATE.digest(namespace_archive/'qualification.json'),
        verification_revision_sha256=GATE.digest(namespace_archive/'verification-revision.json'),
        verification_revision_validator_sha256=GATE.digest(Path(verification_revision.__file__)),
        inode_record_sha256=GATE.digest(inode_archive/'archive.json'),
        archived_files=len(files),unique_blobs=len(written),
        archive_index_sha256=GATE.digest(output/'files.json'),
        archiver_sha256=GATE.digest(Path(__file__)),
        scope='complete current shared-namespace affected qualification; full initial Linux/distribution/container/VM-host acceptance remains required')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print('FULL_VM_ARCHIVE_PASS observations=230 workloads=51 contracts=45 files='+str(len(files)),flush=True)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('baseline','corpus','namespace','inode','inode-command','contracts','output'):
        parser.add_argument('--'+name,required=True,type=Path)
    verify(parser.parse_args())
