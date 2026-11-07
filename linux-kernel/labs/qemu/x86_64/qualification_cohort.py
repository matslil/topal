#!/usr/bin/env python3
"""Run a fresh, retained VM qualification cohort through the normal host wrapper."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

LAB=Path(__file__).resolve().parent
REPOSITORY=LAB.parents[3]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(args):
    root=args.output.resolve()
    root.mkdir(parents=True)
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPOSITORY,text=True).strip()
    record=dict(schema='topal.retained-vm-cohort/1',head=head,phase=args.phase,
        complete=False,passed=False,steps=[],harness_sha256=digest(Path(__file__)),
        retention_sha256=digest(LAB/'qualification_gate.py'),
        bounded_wrapper_sha256=digest(REPOSITORY/'scripts/run_bounded.py'),
        environment={name:os.environ.get(name) for name in ('TMPDIR','RUST_MIN_STACK',
            'RUST_TEST_THREADS','CARGO_BUILD_JOBS','CARGO_PROFILE_TEST_DEBUG',
            'CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_CODEGEN_UNITS',
            'CARGO_PROFILE_DEV_CODEGEN_UNITS','CARGO_INCREMENTAL','MALLOC_ARENA_MAX','MALLOC_CONF')})
    sources=root/'sources'
    sources.mkdir()
    record['sources']=[]
    for source in sorted([*LAB.glob('*.py'),*LAB.glob('*.c')]):
        target=sources/source.name
        shutil.copyfile(source,target)
        record['sources'].append(dict(source=str(source.relative_to(REPOSITORY)),
            snapshot=str(target.relative_to(root)),sha256=digest(target)))
    path=root/'cohort.json'

    def save():
        path.write_text(json.dumps(record,indent=2)+'\n')

    def step(name,gate,arguments):
        assert all(digest(REPOSITORY/entry['source'])==entry['sha256'] for entry in record['sources']), \
            'lab sources changed after the cohort snapshot; retain this attempt and use fresh evidence'
        index=len(record['steps'])
        directory=root/('s'+str(index).zfill(3))
        command=[sys.executable,str(REPOSITORY/'scripts/run_bounded.py'),'--',
            sys.executable,str(LAB/'qualification_gate.py'),'--output',str(directory),
            '--gate',gate,'--',*map(str,arguments)]
        item=dict(name=name,command=command,output=str(directory),exit_code=None,
            stdout='s'+str(index).zfill(3)+'.stdout.log',stderr='s'+str(index).zfill(3)+'.stderr.log')
        record['steps'].append(item)
        save()
        print('RETAINED_COHORT_START '+name,flush=True)
        started=time.monotonic()
        with (root/item['stdout']).open('wb') as out,(root/item['stderr']).open('wb') as err:
            run=subprocess.run(command,cwd=REPOSITORY,stdout=out,stderr=err)
        item.update(exit_code=run.returncode,elapsed_seconds=time.monotonic()-started,
            stdout_sha256=digest(root/item['stdout']),stderr_sha256=digest(root/item['stderr']))
        assert run.returncode!=0 or (directory/'retention.json').exists(), 'successful step lost its retention record'
        if (directory/'retention.json').exists():
            retention=json.loads((directory/'retention.json').read_text())
            assert retention['head']==head and retention['complete']
            item['retention_sha256']=digest(directory/'retention.json')
            assert retention['passed']==(run.returncode==0)
        save()
        print('RETAINED_COHORT_DONE '+name+' '+str(run.returncode),flush=True)
        if run.returncode:
            print((root/item['stdout']).read_text(errors='replace')[-10000:],flush=True)
            print((root/item['stderr']).read_text(errors='replace')[-10000:],flush=True)
            raise SystemExit(run.returncode)
        assert all(digest(REPOSITORY/entry['source'])==entry['sha256'] for entry in record['sources']), \
            'lab sources changed while a gate was running'

    save()
    if args.phase=='baseline':
        for name in ('ram_page','file_region','region','positional','named','file_open','volatile',
                     'header','dispatch','identity','retained','structured','numeric','library'):
            step(name,name+'_gate.py',['--write-evidence'])
        for name in ('realtime','time','children','pair','pipes'):
            step(name,name+'_gate.py',['--external-static-user-image',args.external_static_user_image,'--write-evidence'])
        if args.predecessor_artifact is None:
            raise ValueError('baseline qualification requires the explicit deadline predecessor artifact')
        step('deadline','deadline_gate.py',['--external-static-user-image',args.external_static_user_image,
            '--predecessor-artifact',args.predecessor_artifact,'--write-evidence'])
    elif args.phase=='corpus':
        step('complete-busybox-corpus','workload_gate.py',['--output',root/'corpus',
            '--external-static-user-image',args.external_static_user_image,'--timeout',args.timeout])
    else:
        normal=root/'n0'
        fork=root/'n2'
        step('normal','namespace_gate.py',['--output',normal,'--dispositions','--physical','--timeout',args.timeout])
        for index,name in enumerate(('directory','fork','exec','cancel'),1):
            step(name,'namespace_'+name+'_gate.py',['--output',root/('n'+str(index)),'--timeout',args.timeout])
        step('utilities','namespace_utility_gate.py',['--output',root/'n5',
            '--external-static-user-image',args.external_static_user_image,'--timeout',args.timeout])
        modes=('source-false','source-wrong-association','runtime-abort')
        operations=('link','linkat','move','replace','exchange')
        for mi,mode in enumerate(modes):
            for oi,operation in enumerate(operations):
                step('fatal-'+mode+'-'+operation,'namespace_fault_gate.py',
                    ['--output',root/('f'+str(mi)+str(oi)),'--mode',mode,'--operation',operation,'--timeout',args.timeout])
        step('clock-observe','namespace_clock_gate.py',['--output',root/'c0','--fixture',normal,'--timeout',args.timeout])
        for index,operation in enumerate(operations,1):
            step('clock-error-'+operation,'namespace_clock_gate.py',
                ['--output',root/('c'+str(index)),'--operation',operation,'--timeout',args.timeout])
        for index,mode in enumerate(('bad-link-count','overlap-namespace')):
            step('inode-admission-'+mode,'namespace_inode_guard_gate.py',
                ['--fixture',fork,'--output',root/('g'+str(index)),'--mode',mode,'--timeout',args.timeout])
        for index,mode in enumerate(('normal','runtime-abort','post-frame','post-control','source-false',
            'source-wrong-association','pre-owner','pre-view','pre-input','pre-generation',
            'pre-overlap-position','pre-overlap-page','pre-overlap-inode')):
            fixture=root/'f00' if mode=='source-false' else root/'f10' if mode=='source-wrong-association' else normal
            step('controller-'+mode,'namespace_controller_gate.py',
                ['--fixture',fixture,'--output',root/('k'+str(index)),'--mode',mode,'--timeout',args.timeout])
    record.update(complete=True,passed=True)
    save()
    print('RETAINED_COHORT_PASS '+args.phase+' steps='+str(len(record['steps'])),flush=True)
    return record


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--phase',required=True,choices=('baseline','namespace','corpus'))
    parser.add_argument('--external-static-user-image',required=True,type=Path)
    parser.add_argument('--predecessor-artifact',type=Path)
    parser.add_argument('--timeout',type=float,default=180)
    args=parser.parse_args()
    if args.phase=='baseline' and args.predecessor_artifact is None:
        parser.error('baseline qualification requires --predecessor-artifact')
    verify(args)
