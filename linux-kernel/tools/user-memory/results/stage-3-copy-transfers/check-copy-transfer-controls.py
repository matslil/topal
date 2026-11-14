"""Fault-inject the offline checker; never change or execute guest artifacts."""
import json
from pathlib import Path
import shutil
import subprocess
import sys

build, output = map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
results=[]
program = r'''
import sys
from pathlib import Path
sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
import audit_copy_transfers as a
name,build=sys.argv[1:3]
if name=='missing-shared-caller':
    a.CASES=tuple(r for r in a.CASES if r[0]!='clock-time-output')
elif name=='lost-helper-alias':
    a.PRESERVED=a.PRESERVED-{14}
elif name=='sampler-clobber':
    original=a.sampler_writes
    a.sampler_writes=lambda raw: original(raw)|{3}
else:
    original=a.step
    def changed(raw,registers,offset,address):
        result=original(raw,registers,offset,address)
        if name=='missing-cld' and result['kind']=='clear-direction':
            return dict(kind='register')
        if name=='copy-count-mismatch' and raw[:2]==bytes.fromhex('4c 89') and raw[-1]==0xf9:
            registers[1]=a.plus(registers[1],1)
        if name=='copy-pointer-mismatch' and result['kind']=='copy':
            registers[7]=a.constant(0)
        return result
    a.step=changed
a.audit(Path(build))
'''
for name in ['missing-shared-caller','lost-helper-alias','sampler-clobber',
             'missing-cld','copy-count-mismatch','copy-pointer-mismatch']:
    work=output/name
    work.mkdir()
    for file in ['provider.o','kernel.elf','extent-call-inventory.json',
                 'process-access-inventory.json','process-access-origins.json']:
        shutil.copyfile(build/file,work/file)
    result=subprocess.run([sys.executable,'-c',program,name,str(work)],
        stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode!=0 and b'AssertionError' in result.stdout,(name,result.stdout)
    for file in ['provider.o','kernel.elf','extent-call-inventory.json',
                 'process-access-inventory.json','process-access-origins.json']:
        assert (work/file).read_bytes()==(build/file).read_bytes()
    record=dict(name=name,rejected=True,exit_code=result.returncode,
        scope='Offline in-process checker configuration/effect fault injection; unchanged guest ELF/metadata and no VM execution or state mutation')
    (work/'rejection.json').write_text(json.dumps(record,indent=2)+'\n')
    results.append(record)
(output/'controls.json').write_text(json.dumps(dict(passed=True,rejections=results),indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
