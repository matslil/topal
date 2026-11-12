"""Independent saved FIFO/source/event ordering and byte-sequence model."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import runpy
import sys

build,output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=False)
digest = lambda data:hashlib.sha256(data).hexdigest()
obs_bytes = (build/'source-user-extents/observations.json').read_bytes()
obs = json.loads(obs_bytes)
audit_bytes = (build/'pipe-loop-audit.json').read_bytes()
audit = json.loads(audit_bytes)
loops = obs['pipe_loop_observation']
assert obs['passed'] and audit['passed'] and loops['passed']
assert loops['kernel_sha256'] == audit['kernel_sha256'] == digest((build/'kernel.elf').read_bytes())
assert loops['audit_sha256'] == digest(audit_bytes)
records,refusals = iter(loops['records']),iter(loops['refusals'])
counts = {name:Counter() for name in ['pipe-read-loop','pipe-write-loop']}
events,errors,zero = [],[],[]
for execution in obs['executions']:
    assert len(execution['calls'])==44 and execution['pipe_retirement_verified']
    queue,cursor = bytearray(),0
    for call in execution['calls']:
        n,args,result = call['number'],call['args'],call['result']
        if n not in [0,1]: continue
        assert call['complete_fifo_state_verified'] and call['complete_table_verified'] and call['copy_window_verified']
        assert call['phase_before']==call['phase_after']
        name = 'pipe-read-loop' if n==0 else 'pipe-write-loop'
        if result>0:
            event = next(records)
            assert (event['name'],event['number'],event['args'],event['candidate'],event['size'],event['result']) == (
                name,n,args,args[1],result,result)
            assert all(event[k] for k in ['helper_registers_preserved','captured_arguments_preserved',
                'exact_user_and_ring_bytes','boundary_mappings_equal'])
            expected_count = min(args[2],len(queue) if n==0 else 4096-len(queue))
            assert result==expected_count and 1<=result<=4096
            initial = cursor if n==0 else (cursor+len(queue))%4096
            assert event['initial_cursor']==initial and event['ring_wrap']==(initial+result>4096)
            payload = bytes.fromhex(event['payload_hex'])
            if n==0:
                assert payload==bytes(queue[:result]) and payload.hex()==call['output_hex']
                del queue[:result]
                cursor = (cursor+result)%4096
            else:
                assert payload==bytes.fromhex(call['buffer_before'])[:result]
                if args[1]==0x635000:
                    assert payload==bytes((i*37+11)&255 for i in range(result))
                elif result==16: assert payload==b'owned-pipe-data!'
                else: assert result==8 and payload==b'owned-pi'
                queue.extend(payload)
            counts[name].update(attempts=1,admitted=1,completed=1,bytes=result)
            events.append(dict(name=name,candidate=args[1],size=result,initial_cursor=initial,
                wrap=event['ring_wrap'],payload_sha256=digest(payload)))
        elif result==-14 and args[1]!=(1<<64)-1:
            event = next(refusals)
            assert (event['name'],event['number'],event['args'],event['candidate'],event['result']) == (name,n,args,args[1],result)
            assert event['size']==min(args[2],len(queue) if n==0 else 4096-len(queue))
            counts[name].update(attempts=1,refused=1)
            errors.append(dict(name=name,candidate=args[1],size=event['size']))
        else:
            assert result in [-14,-11,0]
            if result==0 and args[2]==0: zero.append(dict(name=name,candidate=args[1]))
    assert not queue
assert len(obs['executions'])==4
assert next(records,None) is next(refusals,None) is None
for name,row in counts.items():
    expected = dict(row)
    expected.setdefault('refused',0)
    assert expected == loops['counts'][name]
assert len(events)==40 and sum(e['size'] for e in events)==65808
assert len(errors)==28 and len(zero)==8 and sum(e['wrap'] for e in events)==16
program = (build/'source-user-extents/inspect.gdb').read_text()
for filename in ['user_extent_pipe_loop_observer.py','user_extent_pipe_observer.py']:
    selected = runpy.run_path(str(Path('linux-kernel/labs/qemu/x86_64')/filename))['PROGRAM']
    assert selected in program
report = dict(passed=True,kernel_sha256=audit['kernel_sha256'],audit_sha256=digest(audit_bytes),
    observation_sha256=digest(obs_bytes),source_syscalls=176,loop_completions=events,refusals=errors,
    zero_length_calls_without_transfer=zero,counts={k:dict(v) for k,v in counts.items()},
    wrapping_transfers=16,saved_observer_programs_present=True,
    scope='Independent saved source/event correspondence and FIFO queue/cursor/pattern model. Runtime complete ring-erasure/payload and boundary mapping checks retain their live-observer evidence; no per-byte native transcript, second native run, complete owners/nonalias/lifetime or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,syscalls=176,completions=40,bytes=65808,refusals=28,wraps=16,zero_calls=8)))
