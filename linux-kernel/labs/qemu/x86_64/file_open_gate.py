#!/usr/bin/env python3
"""Execute versioned ordinary Topal open plans before real file publication."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time
from volatile_gate import GATE, CHECKS, build as build_files, observe as observe_files

LAB = Path(__file__).resolve().parent
CHOICES = 'OpenDescription, NotDirectory, DirectoryWrite, ReadOnlyView, Missing, AlreadyExists, CreateDescription, TruncateDescription'

def observe(root, values, timeout, wrong=False):
    root.mkdir(parents=True)
    artifact=json.loads(values[3].read_text())
    binding=artifact['kernel_library']
    if binding['interface']!='file-open-decision-v1' or binding['native_export']!='topal_kernel_file_open_policy_v1':
        raise GATE.GateError('open plan did not bind the selected version')
    helper=next(f['symbol'] for f in binding['functions'] if 'missing_2dkind' in f['symbol'])
    debug,failed,result,script=(root/name for name in ('debug.sock','failed.txt','observation.json','open.gdb'))
    config=dict(helper=helper,wrong=wrong,failed=str(failed),result=str(result))
    script.write_text(f"set pagination off\nfile {values[3].parent/'kernel.elf'}\ntarget remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\nimport file_open_inspect\nfile_open_inspect.install({config!r})\nend\ncontinue\ndetach\n")
    outcome=[]
    def inspect():
        try:
            until=time.monotonic()+10
            while not debug.exists() and time.monotonic()<until: time.sleep(.02)
            run=subprocess.run(['gdb','-q','-batch','-x',str(script)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=timeout+10)
            (root/'debug.log').write_bytes(run.stdout);outcome.append(run.returncode)
        except BaseException as error: failed.write_text(repr(error))
    thread=threading.Thread(target=inspect,daemon=True)
    original_build,original_command,original_observe=GATE.build_image,GATE.qemu_command,GATE.observe_gate
    def command(*args):
        thread.start();return original_command(*args)+['-S','-gdb',f'unix:{debug},server=on,wait=off']
    def failure():
        if failed.exists():return failed.read_text()
        if outcome and outcome!=[0]:return 'open debugger failed: '+repr(outcome)
        if wrong and result.exists() and outcome==[0]:return 'controlled wrong plan refused'
        return None
    GATE.build_image=lambda *_args,**_kwargs:values
    GATE.qemu_command=command
    GATE.observe_gate=lambda *args,**kwargs:original_observe(*args,**kwargs,failure_check=failure)
    runtime=None
    try:
        try:runtime=GATE.verify(timeout,False)
        except GATE.GateError:
            if not wrong or failed.exists() or not result.exists():raise
        else:
            if wrong:raise GATE.GateError('wrong plan escaped native prepublication refusal')
    finally:
        GATE.build_image,GATE.qemu_command,GATE.observe_gate=original_build,original_command,original_observe
    thread.join(timeout=5)
    if outcome!=[0] or failed.exists() or not result.exists():
        raise GATE.GateError('open inspector failed: '+repr(outcome)+'\n'+(root/'debug.log').read_text())
    observation=json.loads(result.read_text())
    expected='expected-wrong-create-plan-refusal' if wrong else 'pass'
    if observation['result']!=expected:raise GATE.GateError('wrong physical open disposition')
    return dict(runtime=runtime,observation=observation,library=binding,
        linked_kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
        static_user_image_sha256=GATE.digest(values[5]))

def verify(timeout=120,write_evidence=False):
    original_source,original_build=GATE.SOURCE_PATH,GATE.build_image
    source=(GATE.LINUX_KERNEL/'kernel/policy/file-open-access.t').read_text()
    with tempfile.TemporaryDirectory(prefix='topal-file-open-',dir='/tmp') as temporary:
        root=Path(temporary)
        record=dict(schema='topal-file-open-plan-qemu/1',harness_sha256=GATE.digest(Path(__file__)),
            inspector_sha256=GATE.digest(LAB/'file_open_inspect.py'),source_sha256=GATE.hashlib.sha256(source.encode()).hexdigest())
        native_inputs=None
        try:
            for name,text in [('normal',source),('reversed',source.replace(CHOICES,', '.join(reversed(CHOICES.split(', '))))),
                              ('wrong',source.replace('false then CreateDescription','false then OpenDescription'))]:
                case=root/name;case.mkdir()
                libraries=case/'library';shutil.copytree(GATE.LINUX_KERNEL/'kernel',libraries)
                (libraries/'policy/file-open-access.t').write_text(text)
                selected=case/'root.t'
                selected.write_text(original_source.read_text().replace('policy/boot-open-access.t','policy/file-open-access.t')
                    .replace('export is boot-open-decision','export is file-open-decision')
                    .replace('interface is boot-open-decision-v1','interface is file-open-decision-v1'))
                GATE.SOURCE_PATH=selected
                GATE.build_image=lambda destination,**kwargs:original_build(destination,library_root=libraries,**kwargs)
                values,calls=build_files(case/'artifact',False)
                artifact=json.loads(values[3].read_text())
                owned=[i for i in artifact['inputs'] if i['identity'].startswith(('topal.systems-root-object','topal.provider-object'))]
                if len(owned)!=2 or (native_inputs is not None and owned!=native_inputs):
                    raise GATE.GateError('source plan variant changed native root/provider')
                native_inputs=owned
                record[name]=observe(case/'observe',values,timeout,name=='wrong')
                record[name]['actual_file_syscall_returns']=len(calls) if name!='wrong' else None
                record[name]['root_and_provider_inputs']=owned
                if name=='normal':
                    lifetime,lifetime_calls=build_files(case/'lifetime-artifact',True)
                    if GATE.digest(lifetime[3].parent/'kernel.elf')!=record[name]['linked_kernel_sha256']:
                        raise GATE.GateError('lifetime fixture changed selected open-plan kernel')
                    record['lifetime']=observe_files(case/'lifetime',lifetime,lifetime_calls,timeout,lifetime=True)
                    record['lifetime']['actual_file_syscall_returns']=len(lifetime_calls)
                print('PASS source open plan',name,flush=True)
        finally:GATE.SOURCE_PATH,GATE.build_image=original_source,original_build
        # The legacy open profile remains an explicitly selected compatibility run.
        legacy=root/'frozen-root.t'
        frozen_source=original_source.read_text().replace('static-linux-dispatch-v6','static-linux-dispatch-v5')
        if '  namespace-library is' in frozen_source:
            start=frozen_source.index('  namespace-library is')
            end=frozen_source.index('  ),\n',start)+len('  ),\n')
            frozen_source=frozen_source[:start]+frozen_source[end:]
        legacy.write_text(frozen_source.replace('policy/file-open-access.t','policy/boot-open-access.t')
            .replace('export is file-open-decision','export is boot-open-decision')
            .replace('interface is file-open-decision-v1','interface is boot-open-decision-v1'))
        GATE.SOURCE_PATH=legacy
        try:
            frozen=original_build(root/'frozen-artifact')
            GATE.build_image=lambda *_args,**_kwargs:frozen
            runtime=GATE.verify(timeout,False)
        finally:GATE.build_image,GATE.SOURCE_PATH=original_build,original_source
        record['frozen']=dict(runtime=runtime,library=json.loads(frozen[3].read_text())['kernel_library'],
            linked_kernel_sha256=GATE.digest(frozen[3].parent/'kernel.elf'))
    if write_evidence:(LAB/'results/topal-file-open-plan-gate.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    print('PASS current/frozen open plans, ordinary helper frames, private ABI, lifetimes and wrong-plan refusal',flush=True)
    return record

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout',type=float,default=120)
    parser.add_argument('--write-evidence',action='store_true')
    args=parser.parse_args();verify(args.timeout,args.write_evidence)
