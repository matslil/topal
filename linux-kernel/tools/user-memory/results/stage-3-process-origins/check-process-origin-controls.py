"""Reject malformed diagnostic metadata; never execute changed guest code."""
import copy
import json
from pathlib import Path
import shutil
import subprocess
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
origin=json.loads((build/'process-byte-origins.json').read_text())
graph=json.loads((build/'process-access-inventory.json').read_text())
results=[]
for name in ['missing-span','overlap','foreign-source','function-digest','decoded-bytes']:
    work=output/name
    work.mkdir()
    for file in ['provider.o','kernel.elf']:
        shutil.copyfile(build/file,work/file)
    changed=copy.deepcopy(origin)
    changed_graph=copy.deepcopy(graph)
    if name=='missing-span':
        changed['spans'].pop(0)
    elif name=='overlap':
        changed['spans'][1][0]-=1
    elif name=='foreign-source':
        changed['locations'][0]['file']='outside_compiler/not-a-source.rs'
    elif name=='function-digest':
        changed['function_sha256']='0'*64
    else:
        first=changed_graph['instructions'][0]
        raw=bytearray.fromhex(first['bytes'])
        raw[0]^=1
        first['bytes']=raw.hex()
    (work/'process-byte-origins.json').write_text(json.dumps(changed,indent=2)+'\n')
    (work/'process-access-inventory.json').write_text(json.dumps(changed_graph,indent=2)+'\n')
    command=[sys.executable,'linux-kernel/tools/user-memory/trace_process_origins.py',str(work),str(Path.cwd())]
    result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/'rejection.log').write_bytes(result.stdout)
    assert result.returncode!=0 and b'AssertionError' in result.stdout,(name,result.stdout)
    record=dict(name=name,rejected=True,exit_code=result.returncode,
        scope='Offline JSON origin/graph diagnostic mutation only; unchanged first-party ELF and no VM execution or guest-state mutation')
    (work/'rejection.json').write_text(json.dumps(record,indent=2)+'\n')
    results.append(record)
report=dict(passed=True,rejections=results,scope='Malformed diagnostic metadata rejection, not source-provenance authentication or native memory-safety qualification')
(output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,rejected=len(results))))
