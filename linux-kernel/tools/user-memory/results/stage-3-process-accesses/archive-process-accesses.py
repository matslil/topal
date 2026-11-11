"""Freeze selected linked-graph, GNU re-decode and unchanged VM evidence."""
import gzip
import json
from pathlib import Path
import shutil
import struct
import sys
sys.path.insert(0, str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf, digest

build, run, post, readback = map(Path, sys.argv[1:5])
root = Path('linux-kernel/tools/user-memory/results/stage-3-process-accesses')
root.mkdir(parents=True, exist_ok=True)
cache = Path('linux-kernel/.cache/x86_64')
def copy(source, name):
    target = root/name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
def write(name, data):
    (root/name).write_text(json.dumps(data, indent=2)+'\n')

native = json.loads((run/'run.json').read_text())
assert native['exit_code'] == 0 and not native['source_changes_during_run']
assert json.loads((post/'validation.json').read_text())['passed']
assert json.loads((readback/'readback.json').read_text())['passed']
graph = json.loads((build/'process-access-inventory.json').read_text())
for name in ['extent-call-inventory.json','extent-call-inspection.json','transfer-guard-audit.json',
             'process-access-coverage.json']:
    copy(build/name, name)
for stem in ['extent-call-disassembly','extent-call-inspection','transfer-guard-audit',
             'process-access-inventory','process-access-coverage','process-access-disassembly',
             'process-access-recovery-0','process-access-recovery-1']:
    for suffix in ['command.json','stderr.log']:
        copy(build/f'{stem}.{suffix}', f'audit/{stem}.{suffix}')
    if stem not in ['extent-call-disassembly','process-access-disassembly']:
        copy(build/f'{stem}.stdout.log', f'audit/{stem}.stdout.log')
artifacts = {}
for name in ['provider.o','kernel.elf','process-access-inventory.json','process-access-disassembly.stdout.log']:
    data = (build/name).read_bytes()
    path = root/'artifacts'/(name+'.gz')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(gzip.compress(data, mtime=0))
    artifacts[name] = dict(bytes=len(data), sha256=digest(data),
        snapshot=str(path.relative_to(root)), snapshot_sha256=digest(path.read_bytes()))
write('artifact-identity.json', artifacts)
copy(run/'run.json','native-run.json')
copy(run/'test.log','native-test.log')
for name in ['inspect.gdb','observations.json','gdb.stdout.log','gdb.stderr.log',
             'serial.log','qemu.command.json','qemu.stdout.log','qemu.stderr.log']:
    copy(build/'first-image-exit'/name, 'guest/'+name)
copy(post/'validation.json', 'post-validation.json')
for path in post.glob('*.log'):
    copy(path, 'post/'+path.name)
copy(readback/'readback.json', 'readback.json')
for path in readback.iterdir():
    if path.name != 'readback.json' and path.is_file():
        copy(path, 'gnu/'+path.name)
for name in ['readback-process-accesses','archive-process-accesses','check-process-accesses']:
    copy(cache/(name+'.py'), name+'.py')
copy(cache/'check-startup-source.py', 'post-runner-base.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'), 'native-runner.py')

# Record the local origin of each indirect target. This is a load provenance
# observation, not proof of registration, callee ownership or target effects.
elf = Elf(build/'kernel.elf')
symbol, body = elf.function(graph['function'])
names = {952:'policy',960:'numeric',976:'header',984:'dispatch',992:'identity'}
bindings = []
for row in graph['indirect_transfers']:
    at = row['offset']
    lo = at-47
    assert body[lo:lo+5] == bytes.fromhex('65 48 8b 04 25')
    slot = struct.unpack_from('<I', body,lo+5)[0]
    assert slot in names
    assert body[lo+9:lo+18] == bytes.fromhex('48 81 f8 00 00 00 00 0f 84')
    assert body[lo+22:at+2] == bytes.fromhex(
        '49 89 e3 48 81 e4 f0 ff ff ff 48 81 c4 f0 ff ff ff 4c 89 9c 24 00 00 00 00 ff d0')
    assert not any(e['kind']=='direct-transfer' and lo < e['target'] <= at for e in graph['edges'])
    bindings.append(dict(offset=at, target_register='rax',gs_slot=slot,
        source_role=names[slot],local_load_offset=lo,
        source='src/topal-compiler/src/systems_process_'+names[slot]+'.rs',
        scope='Exact local GS load/null guard/alignment/call slice and no decoded direct entry into its interior; registration, callee and asynchronous reachability remain unqualified here'))
write('indirect-context-loads.json',dict(passed=True,bindings=bindings))
summary = {k:v for k,v in graph.items() if k not in ['instructions','edges','terminals']}
write('graph-summary.json',summary)
write('exploration.json',dict(
    scope='Offline exploratory assumptions, separate from the passing native run and from AI safeguard notices',
    initial_all_emitted_calls_assumption='An exploratory assertion requiring all 91 emitted calls in the direct graph failed for offset 88236. Emission does not imply reachability. The final inventory reports this call outside its graph and separately requires every observed native call to be included; no security acceptance gate is removed.',
    linear_decode_gap='Two real branch destinations, 162020 and 185049, required targeted decoding after linear decoding from embedded-data regions missed their instruction boundaries.',
    remaining='Outside this selected graph is not a whole-program unreachability proof; direct source routing, external/indirect callees and alternate roots remain explicit obligations'))
previous = json.loads((root.parent/'stage-3-transfer-guards/source-identity.json').read_text())
sources = set(previous['selected_inputs']) | set(native['modified_source_hashes_before']) | {
    'src/topal-compiler/src/systems_process_entry_start.rs',
    'src/topal-compiler/src/systems_provider_emit_process_entry.rs',
} | {'src/topal-compiler/src/systems_process_'+name+'.rs' for name in names.values()}
identities = {}
for source in sorted(sources):
    data = Path(source).read_bytes()
    if source in native['modified_source_hashes_before']:
        assert digest(data) == native['modified_source_hashes_before'][source], source
    target = root/'inputs'/(source+'.gz')
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(gzip.compress(data,mtime=0))
    identities[source] = dict(sha256=digest(data), snapshot=str(target.relative_to(root)))
write('source-identity.json',dict(base_commit='8c0c0ab922299fb89d70e765a3b3a5a3b4b06f77',
    selected_inputs=identities,
    scope='Selected process/control/callee-boundary/observer/authority snapshots; not complete producer or production cold reconstruction'))
write('scope.json',dict(stage_complete=False,production_activation=False,
    remaining=['complete all-kernel/callee/exception entry graph and body/owner bindings',
        'classify potential memory operations by pointer origin and transfer direction',
        'complete admission dominance, length/error/staging and stable access lifetime proof',
        'three fallback native coverage/reachability obligations',
        'native ownership/mode/stale-state/stability qualification',
        'canonical production binding and exact committed cold reconstruction',
        'all remaining original MM/distribution/driver/container/VM-host acceptance']))
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root),selected_inputs=len(identities))))
