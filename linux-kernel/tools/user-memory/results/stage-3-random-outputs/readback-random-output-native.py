"""Join actual bounded-random records to source syscalls and returned opaque bytes."""
import hashlib
import json
from pathlib import Path
import runpy
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest=lambda raw:hashlib.sha256(raw).hexdigest()
raw=(build/'source-user-extents/observations.json').read_bytes()
obs=json.loads(raw)
audit_bytes=(build/'random-output-audit.json').read_bytes()
audit=json.loads(audit_bytes)
native=obs['random_output_observation']
assert obs['passed'] and native['passed'] and native['audit_sha256']==digest(audit_bytes)
assert obs['extent_call_coverage']['kernel_sha256']==audit['kernel_sha256']==digest((build/'kernel.elf').read_bytes())
records=iter(native['records'])
wrappers=[e for e in obs['startup_executions'] if e['number']==318]
assert len(wrappers)==8
events=[]
early=[]
for ordinal,wrapper in enumerate(wrappers):
    calls=[c for c in wrapper['calls'] if c['number']==318]
    assert len(calls)==9 and wrapper['args'][1]==8
    for call,(size,flags,result) in zip(calls[:3],[(0,1,0),(8,8,-22),(17,1,-22)]):
        assert call['args'][:3]==[(1<<64)-1,size,flags] and call['result']==result
        assert call['phase_before']==call['phase_after']==6
        early.append(dict(size=size,flags=flags,result=result))
    expected_marker=1 if ordinal==0 else 2 if ordinal==1 else 3
    bad=[0x63a000,(1<<64)-1,0x400000,0x800000,0x639fff]
    for index,call in enumerate(calls[3:]):
        record=next(records)
        assert call['args'][:3]==[record['candidate'],record['size'],1]
        assert record['size']==8 and record['marker']==expected_marker
        assert call['phase_before']==call['phase_after']==6
        if index<5:
            assert call['result']==-14 and not record['accepted']
            assert record['candidate']==bad[index] and record['stores']==record['readbacks']==[]
            assert all(record[k] for k in ['service_preserved','helper_registers_preserved','captured_arguments_preserved'])
        else:
            assert call['result']==8 and record['accepted'] and record['candidate']==0x638ff8
            assert record['source_offset']==(168 if expected_marker==1 else 224)
            assert record['stores']==record['readbacks']==list(range(8))
            assert record['payload_sha256']==digest(bytes.fromhex(wrapper['output_after']))
            assert all(record[k] for k in ['exact_payload_equal','captured_arguments_preserved',
                'stack_restored','suspended_stack_field_cleared','translation_observations_equal',
                'only_admitted_destination_leaf_ad_sets'])
            for change in record['table_status_sets']:
                before,after=change['before'],change['after']
                assert before&7==7 and not before&~after
                assert change['changed_bits']==before^after and 0<change['changed_bits']<=0x60
                assert not change['changed_bits']&~0x60
                assert change['table']%4096==0 and 0<=change['index']<512
        events.append(dict(candidate=record['candidate'],size=record['size'],marker=record['marker'],
            result=call['result'],accepted=record['accepted'],stores=record['stores'],readbacks=record['readbacks'],
            payload_sha256=record.get('payload_sha256'),table_status_sets=record.get('table_status_sets',[])))
assert next(records,None) is None and len(events)==48 and len(early)==24
assert sum(e['accepted'] for e in events)==8
assert sum(e['size'] for e in events if e['accepted'])==64
program=(build/'source-user-extents/inspect.gdb').read_text()
observer=runpy.run_path('linux-kernel/labs/qemu/x86_64/user_extent_random_observer.py')['PROGRAM']
assert observer in program
report=dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(raw),events=events,early_results_without_admission=early,
    admissions=48,refusals=40,completions=8,bytes=64,initial_completions=1,reusable_completions=7,
    native_lengths=[8],indexed_stores=64,indexed_readbacks=64,
    source_syscall_order_results_and_opaque_output_hashes_equal=True,saved_observer_program_present=True,
    scope='Independent saved record/source syscall ordering, actual candidate/count/results, 24 early no-admission results, complete byte indices and opaque returned-output hashes. Private/helper/mapping assertions retain their native observer evidence. No new native execution, cryptographic proof, full 1..16 native length coverage, extraction scratch/retired outer spill/owner/lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,admissions=48,refusals=40,completions=8,bytes=64,early_results=24)))
