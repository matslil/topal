"""Reject corrupted first-party artifacts offline; never boot these copies."""
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf, digest
from inspect_callback_bindings import relocation_entries, SLOTS

build, output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
bindings = json.loads((build/'callback-bindings.json').read_text())
elf = Elf(build/'kernel.elf')
def file_offset(address):
    owners = [s for s in elf.sections if s[1]!=8 and s[2]&2 and s[3]<=address<s[3]+s[5]]
    assert len(owners)==1
    section, = owners
    return section[4]+address-section[3]

cases = []
for callback in bindings['callbacks']:
    cases.append((callback['role']+'-instruction',file_offset(callback['target']),
                  'linked allocated payload differs'))
literal = None
pointer = None
for callback in bindings['callbacks']:
    obj = Elf(build/'baseline'/(callback['role'].replace('_','-')+'.o'))
    for section in callback['sections']:
        if section['executable']:
            continue
        relocations = relocation_entries(obj,section['object_section'])
        reserved = set()
        for offset,(_,kind,_) in relocations.items():
            reserved.update(range(offset,offset+(8 if kind==1 else 4)))
            if pointer is None and kind==1:
                pointer = file_offset(section['address']+offset)
        free = next((n for n in range(section['bytes']) if n not in reserved),None)
        if literal is None and free is not None:
            literal = file_offset(section['address']+free)
assert literal is not None and pointer is not None
cases += [('literal',literal,'linked allocated payload differs'),
          ('relocated-table',pointer,'linked allocated payload differs')]
root_symbol, _ = elf.function('_topal_kernel_entry')
process, _ = elf.function('topal_x86_systems_static_image_process_enter')
first = bindings['callbacks'][0]
cases += [('root-code-argument',file_offset(root_symbol['value']+first['root_lea_offset']+3),
           "assert linked_root['value']+field+4"),
          ('registration-slot',file_offset(process['value']+first['registration_store_offset']+3),
           'assert len(stores) == 1')]
tool = json.loads((build/'extent-call-disassembly.command.json').read_text())['argv'][0]
records = []
for name, offset, expected in cases:
    work = output/name
    work.mkdir()
    (work/'baseline').mkdir()
    for filename in ['root.o','process-access-inventory.json']:
        shutil.copyfile(build/filename,work/filename)
    shutil.copyfile(build/'baseline/provenance.json',work/'baseline/provenance.json')
    for _,_,key in SLOTS:
        stem = key.replace('_','-')
        for suffix in ['o','t','modules.json']:
            shutil.copyfile(build/'baseline'/(stem+'.'+suffix),work/'baseline'/(stem+'.'+suffix))
    changed = bytearray(elf.data)
    changed[offset] ^= 1
    (work/'kernel.elf').write_bytes(changed)
    graph = json.loads((work/'process-access-inventory.json').read_text())
    graph['kernel_sha256'] = digest(changed)
    (work/'process-access-inventory.json').write_text(json.dumps(graph,indent=2)+'\n')
    command = [sys.executable,'linux-kernel/tools/user-memory/inspect_callback_bindings.py',str(work),tool]
    result = subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode != 0 and expected in result.stdout.decode(),(name,result.stdout.decode())
    record = dict(name=name,kernel_file_offset=offset,original_byte=elf.data[offset],
        changed_byte=changed[offset],corrected_outer_kernel_sha256=digest(changed),
        exit_code=result.returncode,rejected=True,expected_gate=expected,
        scope='Offline corrupted first-party ELF with corrected outer graph/kernel hash; never loaded into a VM, executed, or used to modify a pinned application')
    (work/'rejection.json').write_text(json.dumps(record,indent=2)+'\n')
    records.append(record)
report = dict(passed=True,rejections=records,scope='Nine offline byte-binding counterexamples; not guest fault injection or complete hostile-provenance authentication')
(output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(records))))
