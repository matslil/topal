"""Reconcile saved volatile input admission/copy records and source requests."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import runpy
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
raw=(build/'source-user-extents/observations.json').read_bytes()
obs=json.loads(raw)
audit_bytes=(build/'file-input-audit.json').read_bytes()
audit=json.loads(audit_bytes)
copies=obs['copy_transfer_observation']
assert obs['passed'] and audit['passed'] and copies['passed']
assert copies['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
assert copies['file_input_audit_sha256']==digest(audit_bytes)
admissions,completed=iter(copies['retained_admissions']),iter(copies['retained_completions'])
events,refusals,unused=[],[],[]
variants=Counter()
for execution in obs['executions']:
    assert len(execution['calls'])==62 and execution['storage_retirement_verified']
    for call in execution['calls']:
        n,args,result=call['number'],call['args'],call['result']
        if n not in [1,18]: continue
        assert call['complete_storage_state_verified'] and call['complete_table_verified'] and call['copy_window_verified']
        assert call['phase_before']==call['phase_after']
        name='volatile-write-input' if n==1 else 'volatile-pwrite-input'
        if 'file_input_admission' not in call:
            assert (args[2]==0 and result==0) or (args[1]==(1<<64)-1 and result==-14)
            unused.append(dict(name=name,args=args,result=result))
            continue
        context=call['file_input_admission']
        admission=next(admissions)
        expected=min(args[2],256,4096-context['position']%4096,(1<<63)-1-context['position'])
        assert context['name']==name and context['candidate']==args[1] and context['length']==expected
        assert (admission['name'],admission['candidate'],admission['size'])==(name,args[1],expected)
        assert admission['helper_registers_preserved'] and admission['captured_arguments_preserved']
        if result<0:
            assert result==-14 and not admission['accepted']
            refusals.append(dict(name=name,candidate=args[1],size=expected))
            continue
        assert result==expected and admission['accepted']
        event=next(completed)
        assert (event['name'],event['candidate'],event['size'])==(name,args[1],expected)
        assert event['live_stack_words']==1 and all(event[k] for k in [
            'live_stack_preserved','captured_arguments_preserved','translation_observations_equal','exact_payload_equal'])
        payload=bytes.fromhex(event['payload_hex'])
        assert payload==bytes.fromhex(call['buffer_before'])[:expected]
        if n==1 and result==16: assert payload==bytes(range(0x10,0x20))
        elif n==1:
            assert args[1]==0x635ff8 and result==256
            assert payload==bytes((i*19+7)&255 for i in range(256))
        elif result==8: assert payload==bytes(range(0xa0,0xa8))
        else:
            assert args[1]==0x639ffc and args[2]==512 and context['position']==8188 and result==4
            assert payload==bytes(range(0xd0,0xd4))
        if n==18: assert call['positional_position_preserved']
        variants[(name,context['page_present'])]+=1
        events.append(dict(name=name,candidate=args[1],size=result,position=context['position'],
            page_present=context['page_present'],payload_sha256=digest(payload)))
assert len(obs['executions'])==4 and next(admissions,None) is next(completed,None) is None
assert len(events)==16 and len(refusals)==24 and len(unused)==16
assert sum(r['size'] for r in events)==1136
assert set(variants.values())=={4} and len(variants)==4
for name in ['volatile-write-input','volatile-pwrite-input']:
    counts=copies['counts'][name]
    assert counts==dict(attempts=20,admitted=8,refused=12,completed=8,
        bytes=1088 if name=='volatile-write-input' else 48)
program=(build/'source-user-extents/inspect.gdb').read_text()
for filename in ['user_extent_copy_observer.py','user_extent_storage_observer.py']:
    assert runpy.run_path(str(Path('linux-kernel/labs/qemu/x86_64')/filename))['PROGRAM'] in program
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(raw),source_syscalls=248,completed_inputs=events,refused_inputs=refusals,
    unused_early_results=unused,total_input_bytes=1136,
    variants=[dict(name=name,page_present=present,count=count) for (name,present),count in sorted(variants.items())],
    saved_observer_programs_present=True,
    scope='Independent saved source/admission/completion ordering, clipped counts, deterministic payloads and resident/missing-page variants. Runtime live-stack and boundary mapping checks retain their native observer evidence; not independent private-read reconstruction, a second native run, complete owners/nonalias/lifetime/retired-spill cleanup or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,syscalls=248,admissions=40,completed=16,refused=24,bytes=1136,variants=4)))
