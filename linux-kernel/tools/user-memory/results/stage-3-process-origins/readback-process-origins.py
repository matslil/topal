"""Expand origin spans independently and review unchanged native artifacts."""
from array import array
from collections import Counter
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import digest

build,baseline,output=map(Path,sys.argv[1:4])
output.mkdir(parents=True,exist_ok=True)
origins=json.loads((build/'process-byte-origins.json').read_text())
joined=json.loads((build/'process-access-origins.json').read_text())
graph=json.loads((build/'process-access-inventory.json').read_text())
assert joined['passed'] and joined['function_bytes']==231663
assert len(origins['spans'])==45005 and len(origins['locations'])==9380
assert len(joined['source_hashes'])==50 and joined['access_instructions']==14584
assert len(joined['process_relocations'])==19
assert digest((build/'process-byte-origins.json').read_bytes())==joined['origin_sha256']
assert digest((build/'process-access-inventory.json').read_bytes())==joined['process_graph_sha256']
unset=(1<<32)-1
owner=array('I',[unset])*joined['function_bytes']
for start,end,index in origins['spans']:
    assert start<end and len(owner[start:end])==end-start and all(n==unset for n in owner[start:end])
    owner[start:end]=array('I',[index])*(end-start)
assert all(n!=unset for n in owner)
source_lines={}
for path,sha in joined['source_hashes'].items():
    data=Path(path).read_bytes()
    assert digest(data)==sha
    source_lines[path]=data.decode().splitlines()
for original,row in zip(origins['locations'],joined['locations'],strict=True):
    assert {key:row[key] for key in ['file','line','column']}==original
    assert row['source_line']==source_lines[row['file']][row['line']-1]
expected={row['offset']:row for row in graph['instructions'] if row['access_kinds']}
assert {row['offset'] for row in joined['accesses']}==set(expected)
primary=Counter()
for row in joined['accesses']:
    node=expected[row['offset']]
    assert (row['size'],row['bytes'],row['instruction'],row['access_kinds'])==(
        node['size'],node['bytes'],node['instruction'],node['access_kinds'])
    expanded=[]
    for span in row['byte_origins']:
        expanded.extend([span['location']]*(span['end']-span['start']))
    assert list(owner[row['offset']:row['offset']+row['size']])==expanded
    path=origins['locations'][expanded[0]]['file']
    assert not path.endswith(('systems_x86_function.rs','systems_provider_emit_translation.rs'))
    primary[path]+=1
assert primary=={row['file']:row['primary_access_instructions'] for row in joined['groups']}
comparisons=[]
for name in ['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
             'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
             'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']:
    data=(build/name).read_bytes()
    assert data==(baseline/name).read_bytes(),name
    comparisons.append(dict(path=name,bytes=len(data),sha256=digest(data),unchanged=True))
for name in ['static-elf-abi-fixture.elf','initramfs.cpio']:
    assert (build/name).read_bytes()==(baseline/name).read_bytes()
serial=(build/'first-image-exit/serial.log').read_bytes()
assert serial==(baseline/'first-image-exit/serial.log').read_bytes()
assert json.loads((build/'extent-call-inventory.json').read_text())==json.loads((baseline/'extent-call-inventory.json').read_text())
observed=json.loads((build/'first-image-exit/observations.json').read_text())
assert observed['passed'] and observed['callback_observation']['passed']
assert sum(row['count'] for row in observed['callback_observation']['counts'])==77
assert observed['extent_call_coverage']['reached_sites']==38
leaf_rows=[row for row in joined['accesses']
    if any(joined['locations'][span['location']]['file'].endswith('systems_process_entry_completion.rs')
        and joined['locations'][span['location']]['line'] in [1682,1685] for span in row['byte_origins'])]
assert len(leaf_rows)==1006
report=dict(passed=True,function_bytes=231663,contiguous_spans=45005,source_locations=9380,
    source_files=50,potential_access_instructions=14584,whole_function_relocations=19,
    expanded_byte_origin_equal=True,all_graph_accesses_joined=True,
    generic_encoder_primary_locations=0,leaf_status_instruction_instances=1006,
    kernel_comparisons=comparisons,unchanged_pinned_image=True,unchanged_boot_resource=True,
    pinned_image_sha256=digest((build/'static-elf-abi-fixture.elf').read_bytes()),
    serial_bytes=len(serial),serial_sha256=digest(serial),native_extent_first_reach=38,native_callback_returns=77,
    scope='Selected exact emission/source/access review index and unchanged native witness; not pointer classification, ownership/lifetime, every-path admission dominance or production qualification')
(output/'readback.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='kernel_comparisons'}))
