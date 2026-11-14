"""Host-file GNU decoding of the new graph; no guest connection or execution."""
import json
from pathlib import Path
import subprocess
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
program=Path('linux-kernel/.cache/x86_64/pipe-loop-graph-program.py').read_text()
script=('set pagination off\nset confirm off\nfile '+str((build/'kernel.elf').resolve())+'\npython\n'
    +'graph_path='+repr(str((build/'process-access-inventory.json').resolve()))+'\n'
    +'kernel_path='+repr(str((build/'kernel.elf').resolve()))+'\n'
    +'result_path='+repr(str((output/'gnu-verification.json').resolve()))+'\n'+program+'\nend\nquit\n')
(output/'verify-graph.gdb').write_text(script)
command=['gdb','-nx','-batch','-x',str(output/'verify-graph.gdb')]
result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
for name,data in [('gdb.stdout.log',result.stdout),('gdb.stderr.log',result.stderr),
    ('gdb.command.json',(json.dumps(dict(argv=command,exit_code=result.returncode),indent=2)+'\n').encode())]:
    (output/name).write_bytes(data)
assert result.returncode==0,result.stdout.decode(errors='replace')+result.stderr.decode(errors='replace')
print((output/'gnu-verification.json').read_text())
