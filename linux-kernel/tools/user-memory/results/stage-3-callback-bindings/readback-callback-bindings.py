"""Review final ordinary callback observations and unchanged artifact identities."""
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf, digest

source, application, source_baseline, app_baseline, controls, output = map(Path,sys.argv[1:7])
output.mkdir(parents=True,exist_ok=True)
bindings = json.loads((source/'callback-bindings.json').read_text())
app_bindings = json.loads((application/'callback-bindings.json').read_text())
assert [key for key in bindings if bindings[key]!=app_bindings[key]] == ['baseline_provenance_sha256']
for directory, record in [(source,bindings),(application,app_bindings)]:
    assert digest((directory/'baseline/provenance.json').read_bytes()) == record['baseline_provenance_sha256']
assert bindings['passed'] and len(bindings['callbacks'])==5 and len(bindings['sites'])==6
assert sum(len(r['functions']) for r in bindings['callbacks'])==389
assert sum(len(r['sections']) for r in bindings['callbacks'])==357
assert sum(r['allocation_relocations_verified'] for r in bindings['callbacks'])==978
kernel = Elf(source/'kernel.elf')
assert digest(kernel.data)==bindings['kernel_sha256']
comparisons = []
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data = (source/name).read_bytes()
    assert data==(application/name).read_bytes()==(source_baseline/name).read_bytes()==(app_baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,bytes=len(data),sha256=digest(data),unchanged=True))
for build,baseline,work in [(source,source_baseline,'source-user-extents'),
                            (application,app_baseline,'first-image-exit')]:
    for name in ['static-elf-abi-fixture.elf','initramfs.cpio']:
        assert (build/name).read_bytes()==(baseline/name).read_bytes(),name
    assert (build/work/'serial.log').read_bytes()==(baseline/work/'serial.log').read_bytes()
rows = {r['gs_slot']:r for r in bindings['callbacks']}
expected_slots = set(rows)
def review(build,work):
    data = json.loads((build/work/'observations.json').read_text())
    native = data['callback_observation']
    assert data['passed'] and native['passed']
    assert {r['gs_slot'] for r in native['registered']}==expected_slots
    for row in native['registered']:
        assert row['target']==rows[row['gs_slot']]['target'] and row['argument']==rows[row['gs_slot']]['argument']
    counts = {r['offset']:r['count'] for r in native['counts']}
    samples = {r['offset']:r for r in native['samples']}
    assert set(counts)==set(samples) and all(n>0 for n in counts.values())
    maps = []
    for offset,sample in samples.items():
        assert sample['target']==rows[sample['gs_slot']]['target']
        assert all(sample[k] for k in ['callee_registers_preserved','captured_arguments_preserved',
                                      'callback_registry_preserved','complete_process_table_preserved'])
        translation = sample['supervisor_translation']
        user,writable,nx = True,True,False
        path = translation['path']
        assert [r['shift'] for r in path]==[39,30,21]
        for index,row in enumerate(path):
            entry = row['entry']
            assert entry&1 and row['table']%4096==0 and row['table']<0x40000000
            user = user and bool(entry&4)
            writable = writable and bool(entry&2)
            nx = nx or bool(entry&(1<<63))
            if index+1<len(path):
                assert entry&0x000ffffffffff000 == path[index+1]['table']
        leaf = path[-1]
        assert leaf['entry']&128
        physical = (leaf['entry']&0x000fffffffe00000)|(sample['target']&0x1fffff)
        assert physical==sample['target']==translation['physical']
        assert not user and not nx
        assert translation['effective_user']==user and translation['effective_write']==writable
        assert translation['effective_execute']==(not nx)
        maps.append(dict(offset=offset,effective_user=user,effective_write=writable,effective_execute=not nx))
    section_count=section_bytes=0
    for role in native['live_sections']:
        expected = {r['address']:r for r in rows[role['gs_slot']]['sections']}
        assert len(role['sections'])==len(expected)
        for record in role['sections']:
            section = expected[record['address']]
            assert record['bytes']==section['bytes'] and record['sha256']==section['linked_sha256']
            marker = kernel.symbols[section['marker']+'.begin']
            owner = kernel.sections[marker['section']]
            at = marker['value']-owner[3]
            raw = kernel.section_bytes(marker['section'])[at:at+record['bytes']]
            assert digest(raw)==record['sha256']
            section_count+=1
            section_bytes+=len(raw)
    return dict(passed=True,counts=counts,total_returns=sum(counts.values()),
        observed_sites=len(counts),unreached=native['unreached'],
        live_sections=section_count,live_bytes=section_bytes,hardware_mapping_samples=maps)
source_review = review(source,'source-user-extents')
app_review = review(application,'first-image-exit')
assert source_review['observed_sites']==6 and not source_review['unreached']
assert source_review['total_returns']==1168 and source_review['live_sections']==357 and source_review['live_bytes']==38541
assert app_review['observed_sites']==1 and app_review['total_returns']==77
observed = json.loads((source/'source-user-extents/observations.json').read_text())
assert sum(len(r['calls']) for r in observed['executions'])==208
assert next(r for r in observed['callback_observation']['samples'] if r['offset']==226667)['arguments'][0]==41
assert next(r for r in observed['callback_observation']['samples'] if r['offset']==226667)['result']==6
negative = json.loads((controls/'controls.json').read_text())
assert negative['passed'] and len(negative['rejections'])==9 and all(r['rejected'] for r in negative['rejections'])
report = dict(passed=True,kernel_comparisons=comparisons,controllers=5,functions=389,allocated_sections=357,
    resolved_relocations=978,source=source_review,application=app_review,
    source_syscalls=208,pinned_image_sha256=digest((application/'static-elf-abi-fixture.elf').read_bytes()),
    unchanged_source_image=True,unchanged_pinned_image=True,unchanged_boot_resources=True,
    source_serial_sha256=digest((source/'source-user-extents/serial.log').read_bytes()),
    application_serial_sha256=digest((application/'first-image-exit/serial.log').read_bytes()),
    source_serial_bytes=len((source/'source-user-extents/serial.log').read_bytes()),
    application_serial_bytes=len((application/'first-image-exit/serial.log').read_bytes()),
    offline_counterexamples=9,
    scope='Selected exact source-controller/root registration and observed target/ABI/private-table preservation. Supervisor write permission is observed, not hardware read-only code. Complete callee effects/provenance/lifetime and every caller/user-access obligation remain required')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['kernel_comparisons','source','application']}))
