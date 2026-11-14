"""Compose the six provider edits with the separately bound private predicate."""
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest

intermediate,current,foundation_path,binding_path,output=map(Path,sys.argv[1:6])
foundation=json.loads(foundation_path.read_text())
binding=json.loads(binding_path.read_text())
assert foundation['passed'] and binding['passed']
old_provider=Elf(intermediate/'provider.o')
provider=Elf(current/'provider.o')
kernel=Elf(current/'kernel.elf')
assert old_provider.data==provider.data
assert foundation['new_provider_sha256']==digest(provider.data)
assert binding['old_kernel_sha256']==digest((intermediate/'kernel.elf').read_bytes())
assert binding['kernel_sha256']==digest(kernel.data)
assert binding['target_sha256']==digest((current/'target.o').read_bytes())
name='topal_x86_systems_static_image_process_enter'
symbol,body=provider.function(name)
linked_symbol,linked=kernel.function(name)
reconstructed=bytearray(body)
relocations={at-symbol['value']:row for at,row in provider.relocations(symbol['section']).items()
             if symbol['value']<=at<symbol['value']+len(body)}
assert len(relocations)==19
fixups=[]
occupied=set()
for at,(target,kind,addend) in sorted(relocations.items()):
    assert kind in [2,4] and addend==-4
    assert 0<=at and at+4<=len(body)
    span=set(range(at,at+4))
    assert not occupied&span
    occupied.update(span)
    destination=kernel.symbols[target]
    assert destination['section'] and destination['value']
    value=destination['value']+addend-(linked_symbol['value']+at)
    assert -(1<<31)<=value<1<<31
    reconstructed[at:at+4]=struct.pack('<i',value)
    fixups.append(dict(offset=at,symbol=target,kind=kind,addend=addend,
                       linked_symbol_address=destination['value'],displacement=value))
assert bytes(reconstructed)==linked,'unexplained linked process bytes'
old_graph=json.loads((intermediate/'process-access-inventory.json').read_text())
graph=json.loads((current/'process-access-inventory.json').read_text())
assert graph['kernel_sha256']==digest(kernel.data)
assert {r['offset'] for r in old_graph['roots']}=={r['offset'] for r in graph['roots']}
assert old_graph['edges']==graph['edges']
assert [(r['offset'],r['size']) for r in old_graph['instructions']]==[
    (r['offset'],r['size']) for r in graph['instructions']]
old_census=json.loads((intermediate/'extent-call-inventory.json').read_text())
census=json.loads((current/'extent-call-inventory.json').read_text())
assert old_census['calls']==census['calls'] and len(census['calls'])==91
report=dict(passed=True,kernel_sha256=digest(kernel.data),provider_sha256=digest(provider.data),
    linked_function_sha256=digest(linked),function_bytes=len(linked),
    foundation_sha256=digest(foundation_path.read_bytes()),
    target_binding_sha256=digest(binding_path.read_bytes()),
    provider_rewrites=foundation['rewrites'],checked_internal_fixups=foundation['checked_internal_fixups'],
    unvisited_internal_fixups=foundation['unvisited_internal_fixups'],
    unchanged_other_provider_functions=foundation['unchanged_other_provider_functions'],
    current_linker_fixups=fixups,complete_linked_body_reconstructed=True,
    mapped_graph_nodes=len(graph['instructions']),mapped_graph_edges=len(graph['edges']),
    mapped_helper_calls=len(census['calls']),
    changed_target_function=binding['function'],
    unchanged_target_functions=binding['unchanged_target_functions'],
    unchanged_intermediate_objects=binding['unchanged_objects'],
    scope='Composed exact six-provider-edit proof plus one tested private predicate replacement. All current linked process bytes equal the unchanged intermediate provider after nineteen checked symbolic linker fixups; current graph/census offsets and all other target bodies/relocations match. Not a full producer reconstruction or ownership/lifetime proof.')
output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,kernel_sha256=report['kernel_sha256'],
    provider_rewrites=len(report['provider_rewrites']),internal_fixups=len(report['checked_internal_fixups']),
    linked_fixups=len(fixups),other_provider_functions=len(report['unchanged_other_provider_functions']),
    other_target_functions=len(report['unchanged_target_functions']),complete_linked_body_reconstructed=True)))
