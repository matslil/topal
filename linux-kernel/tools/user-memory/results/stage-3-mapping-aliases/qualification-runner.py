import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

repo=Path.cwd()
base=repo/'linux-kernel/.cache/x86_64'
output=base/'mapping-alias-final-checks'
output.mkdir(exist_ok=False)
build=base/'user-extent-source-metadata-2408345-1794470644696506965'
readers=repo/'linux-kernel/tools/user-memory'
digest=lambda data:hashlib.sha256(data).hexdigest()
names=['alias_snapshot.py','audit_mapping_aliases.py','test_alias_snapshot.py',
    'readback_mapping_aliases.py','control_mapping_aliases.py']
before={name:digest((readers/name).read_bytes()) for name in names}
commands=[]
def run(label,args):
    start=time.monotonic()
    result=subprocess.run([sys.executable]+list(map(str,args)),cwd=repo,
        env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (output/(label+'.log')).write_bytes(result.stdout)
    commands.append(dict(label=label,command=[sys.executable]+list(map(str,args)),
        exit_code=result.returncode,seconds=time.monotonic()-start,log_sha256=digest(result.stdout)))
    (output/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
    assert result.returncode==0,(label,result.stdout[-4000:].decode(errors='replace'))
    print(label+' passed',flush=True)
run('models',['-m','unittest','discover','-s',readers,'-v'])
assert b'Ran 164 tests' in (output/'models.log').read_bytes()
report=output/'mapping-aliases.json'
run('aliases',[readers/'audit_mapping_aliases.py',build,report])
run('independent',[readers/'readback_mapping_aliases.py',build,report,output/'independent.json'])
run('controls',[readers/'control_mapping_aliases.py',build,report,output/'controls'])
after={name:digest((readers/name).read_bytes()) for name in names}
assert before==after
result=dict(passed=True,model_tests=164,rejected_saved_data_controls=20,
    source_hashes_before=before,source_hashes_after=after,commands=commands,
    scope='Bounded Python-only saved-data/model checks; no new native execution')
(output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
