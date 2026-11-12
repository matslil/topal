"""Freeze guarded pipe-loop sources and actual validation artifacts."""
import gzip
import hashlib
import json
from pathlib import Path

cache = Path('linux-kernel/.cache/x86_64')
root = Path('linux-kernel/tools/user-memory/results/stage-3-pipe-loops')
prior = root.parent/'stage-3-serial-prefix'
builds = dict(pipes=cache/'user-extent-source-pipes-2379391-1794452959731792752',
    source=cache/'user-extent-source-2380225-1794453340010915677',
    pinned=cache/'user-extent-application-2379978-1794453220767025660')
runs = dict(pipes=cache/'tk283-staging-focused-1794452950707407241',
    source=cache/'tk283-staging-focused-1794453338998087040',
    pinned=cache/'tk283-staging-focused-1794453219740910708',
    provider=cache/'tk283-staging-focused-1794453197818739514',
    fixtures=cache/'tk283-staging-focused-1794453616456833800')
digest = lambda data:hashlib.sha256(data).hexdigest()
frozen = None
for label,path in runs.items():
    run = json.loads((path/'run.json').read_text())
    assert run['exit_code']==0 and not run['source_changes_during_run']
    assert run['modified_source_hashes_before']==run['modified_source_hashes_after']
    assert run['selected_tests']==dict(provider=3,fixtures=2).get(label,1)
    assert run['resources']['memory.swap.max']=='0'
    assert all(int(line.split()[1])==0 for line in run['resources']['memory.events'].splitlines())
    if frozen is None: frozen=run['modified_source_hashes_before']
    assert frozen==run['modified_source_hashes_before']
assert len(frozen)==9 and not root.exists()
root.mkdir()
def save(name,data):
    path=root/name
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(data)
def write(name,value): save(name,(json.dumps(value,indent=2)+'\n').encode())
def zipped(name,data):
    packed=gzip.compress(data,mtime=0)
    save(name,packed)
    return dict(snapshot=name,sha256=digest(data),bytes=len(data),snapshot_sha256=digest(packed))
old_inputs=(prior/'source-identity.json').read_bytes()
inputs={name:dict(row,snapshot='../stage-3-serial-prefix/'+row['snapshot'])
    for name,row in json.loads(old_inputs)['selected_inputs'].items()}
for name,sha in frozen.items():
    data=Path(name).read_bytes()
    assert digest(data)==sha
    inputs[name]=zipped('inputs/'+name+'.gz',data)
assert len(inputs)==152
write('source-identity.json',dict(base_commit='c5a59dcd8849ed83fa7de67408c95b59ddd0c753',
    inherited_record='../stage-3-serial-prefix/source-identity.json',inherited_record_sha256=digest(old_inputs),
    selected_inputs=inputs,scope='Selected frozen sources only; not a complete clean producer or production qualification'))
fields=[('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset'),('pipe-fixed-output-audit.json','access_instructions'),
    ('stat-output-audit.json','copy_offset'),('serial-prefix-audit.json','copy_offset'),('pipe-loop-audit.json','user_accesses')]
names=['provider.o','kernel.elf','extent-call-inventory.json','process-access-inventory.json',
    'process-access-origins.json','process-byte-origins.json','static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']
names += [name for name,_ in fields]
artifacts={name:zipped('artifacts/'+name+'.gz',(builds['pipes']/name).read_bytes()) for name in names}
comparison_bytes=(cache/'pipe-loop-emission-comparison/comparison.json').read_bytes()
comparison=json.loads(comparison_bytes)
assert comparison['passed']
for row in comparison['unchanged_component_objects']:
    name=row['path']
    data=(builds['pipes']/name).read_bytes()
    assert digest(data)==row['sha256'] and len(data)==row['bytes']
    artifacts[name]=zipped('artifacts/'+name+'.gz',data)
old_artifacts=json.loads((prior/'artifact-identity.json').read_text())
for name in ['provider.o','extent-call-inventory.json','process-access-inventory.json']:
    row=old_artifacts[name]
    artifacts['before/'+name]=dict(row,snapshot='../stage-3-serial-prefix/'+row['snapshot'])
assert len(artifacts)==35
write('artifact-identity.json',artifacts)
save('emission-comparison.json',comparison_bytes)
kernel_sha=artifacts['kernel.elf']['sha256']
selected,accesses={},set()
for name,field in fields:
    report=json.loads((builds['pipes']/name).read_text())
    assert report['passed'] and report['kernel_sha256']==kernel_sha
    for row in report['cases']:
        assert row['call_offset'] not in selected
        selected[row['call_offset']]=dict(name=row['name'],call_offset=row['call_offset'],audit=artifacts[name]['snapshot'])
        accesses.update([row[field]] if field=='copy_offset' else [r['offset'] for r in row[field]])
census=json.loads((builds['pipes']/'extent-call-inventory.json').read_text())
remaining=[r for r in census['calls'] if r['offset'] not in selected]
assert (len(selected),len(accesses),len(remaining))==(86,314,5)
write('caller-access-index.json',dict(complete=False,kernel_sha256=kernel_sha,emitted_helper_calls=91,
    calls_with_selected_local_intervals=86,distinct_user_access_instructions=314,
    selected_intervals=[selected[at] for at in sorted(selected)],calls_without_selected_local_intervals=remaining,
    scope='Nine current-kernel conditional local reports; not complete owners/lifetime/callers/fallbacks or production qualification'))
for label,path in runs.items():
    save('native/'+label+'/run.json',(path/'run.json').read_bytes())
    zipped('native/'+label+'/test.log.gz',(path/'test.log').read_bytes())
coverage={}
cohort_identities={}
for label,build in builds.items():
    folders=[p for p in build.iterdir() if p.is_dir() and (p/'observations.json').is_file()]
    assert len(folders)==1
    obs=json.loads((folders[0]/'observations.json').read_text())
    assert obs['passed'] and obs['extent_call_coverage']['kernel_sha256']==kernel_sha
    coverage[label]={r['offset'] for r in obs['extent_call_coverage']['reached']}
    for path in folders[0].iterdir():
        if path.is_file() and path.suffix in ['.json','.log','.gdb']:
            zipped('native/'+label+'/observation/'+path.name+'.gz',path.read_bytes())
    if (build/'source-component.json').is_file():
        zipped('native/'+label+'/source-component.json.gz',(build/'source-component.json').read_bytes())
    cohort_identities[label]={}
    for name in names:
        if name in ['static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']:
            if not (build/name).exists(): continue
        data=(build/name).read_bytes()
        cohort_identities[label][name]=dict(sha256=digest(data),bytes=len(data))
        if name not in ['static-elf-abi-fixture.elf','source-user.elf','initramfs.cpio']:
            assert digest(data)==artifacts[name]['sha256']
    for row in comparison['unchanged_component_objects']:
        assert digest((build/row['path']).read_bytes())==row['sha256']
    for name in ['kernel.elf','bzImage','provider.o']:
        assert (build/name).read_bytes()==(builds['pipes']/name).read_bytes()
write('native-input-identities.json',cohort_identities)
union=set().union(*coverage.values())
write('native-coverage.json',dict(passed=True,kernel_sha256=kernel_sha,
    fresh_first_reach_sites={k:len(v) for k,v in coverage.items()},matching_kernel_union=len(union),
    reached=sorted(union),unreached=[r for r in census['calls'] if r['offset'] not in union],
    scope='Only three actual current-kernel first-reach cohorts; earlier kernel coverage is not relabeled'))
save('native-readback.json',(cache/'pipe-loop-native-readback/readback.json').read_bytes())
for path in (cache/'pipe-loop-graph-readback').iterdir():
    if path.is_file(): zipped('gnu-graph/'+path.name+'.gz',path.read_bytes())
save('gnu-operands.json',(cache/'pipe-loop-operand-readback/readback.json').read_bytes())
controls=json.loads((cache/'pipe-loop-controls/controls.json').read_text())
assert controls['passed'] and len(controls['controls'])==17
save('controls/controls.json',(cache/'pipe-loop-controls/controls.json').read_bytes())
save('controls/positive.log',(cache/'pipe-loop-controls/positive.log').read_bytes())
for row in controls['controls']:
    data=(cache/'pipe-loop-controls'/row['name']/'rejection.log').read_bytes()
    assert digest(data)==row['log_sha256']
    save('controls/'+row['name']+'.log',data)
for filename in ['pipe-cases.su','source-component.json']:
    zipped('fixture/'+filename+'.gz',(builds['pipes']/filename).read_bytes())
for name in ['archive-pipe-loops.py','compare-pipe-loop-emission.py','readback-pipe-loop-native.py',
             'check-pipe-loop-controls.py','readback-pipe-loop-operands.py','verify-pipe-loop-graph.py',
             'pipe-loop-graph-program.py','run-staging-focused-tests.py']:
    save(name,(cache/name).read_bytes())
write('review.json',dict(passed=True,stable_ids=['TK-DEC-283','TOPAL-SYSTEMS-USER-EXTENT-001',
    'TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-TRANSFER-001','TOPAL-REQ-SYSTEMS-USER-EXTENT-001',
    'TOPAL-COMP-SYSTEMS-USER-EXTENT-001'],
    risk='High: protects the positive count required by do-before-test byte loops; no core or authoritative meaning changes, no predecessor-byte changes, no owned production activation',
    review='Single-agent higher-authority/source/full-diff review, complete body and shifted-relocation reconstruction, GNU whole-graph and selected operands/cuts, seventeen offline ELF controls, 94 models/readers, three native regressions and targeted provider/fixture tests',
    selected_inputs=152,changed_inputs_stable_in_all_five_commands=9,model_reader_tests=94,
    body_bytes_added=52,adjusted_relative_displacements=219,unchanged_component_objects=14,
    source_syscalls=176,loop_transfers=40,loop_bytes=65808,loop_refusals=28,wrapping_transfers=16,zero_length_calls=8,
    user_stack_bound_bytes=1368,user_stack_limit_bytes=2048,
    conditional_sites=86,user_access_instructions=314,remaining_sites=5,
    kernel_sha256=kernel_sha,failures=[],safeguard_notices=[],core_language_or_authority_meaning_changed=False,
    remaining=['Actual owners/nonaliasing and continuous lifetime',
        'Five sites, all helper/entry/callee/return/fatal/exception and fallback obligations',
        'Further native profiles on this changed kernel and canonical clean production reconstruction',
        'Complete original Linux-compatible kernel acceptance']))
print(json.dumps(dict(output=str(root),selected_inputs=len(inputs),artifacts=len(artifacts),
    conditional_sites=86,user_access_instructions=314,current_kernel_first_reach_union=len(union))))
