"""GNU host-file linear decode of the complete preceding process byte span."""
import json
from pathlib import Path
import subprocess

base=Path('linux-kernel/.cache/x86_64')
old=base/'user-extent-source-storage-2391049-1794459629440117879'
output=base/'stack-retirement-readbacks/baseline-linear'
output.mkdir(exist_ok=False)
program=r'''
import gdb,json,hashlib
from pathlib import Path
graph=json.loads(Path(graph_path).read_text())
assert hashlib.sha256(Path(kernel_path).read_bytes()).hexdigest()==graph['kernel_sha256']
base=graph['function_address']
rows=gdb.selected_inferior().architecture().disassemble(base,base+graph['function_bytes'])
transcript=[dict(offset=row['addr']-base,size=row['length'],assembly=row['asm']) for row in rows]
raw=(json.dumps(transcript,sort_keys=True)+'\n').encode()
Path(output_path).write_bytes(raw)
print(json.dumps(dict(passed=True,rows=len(transcript),sha256=hashlib.sha256(raw).hexdigest())))
'''
script='set pagination off\nfile '+str((old/'kernel.elf').resolve())+'\npython\n'
script+='graph_path='+repr(str((old/'process-access-inventory.json').resolve()))+'\n'
script+='kernel_path='+repr(str((old/'kernel.elf').resolve()))+'\n'
script+='output_path='+repr(str((output/'transcript.json').resolve()))+'\n'+program+'\nend\nquit\n'
(output/'decode.gdb').write_text(script)
command=['gdb','-nx','-batch','-x',str(output/'decode.gdb')]
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
(output/'stdout.log').write_bytes(result.stdout)
(output/'stderr.log').write_bytes(result.stderr)
(output/'command.json').write_text(json.dumps(dict(command=command,exit_code=result.returncode),indent=2)+'\n')
assert result.returncode==0,result.stderr[-2000:]
print(result.stdout.decode())
