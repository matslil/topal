"""Publish completed VM proof with external payloads kept outside Git."""
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT=Path('/home/mats/git/topal')
CACHE=ROOT/'linux-kernel/.cache'
LAB=ROOT/'linux-kernel/labs/qemu/x86_64'
OUT=CACHE/'tk281-current-publication-2'
OUT.mkdir()
record=dict(complete=False,passed=False,steps=[])


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def save(): (OUT/'publication.json').write_text(json.dumps(record,indent=2)+'\n')


def run(name,args):
    command=[sys.executable,str(ROOT/'scripts/run_bounded.py'),'--',sys.executable,*map(str,args)]
    log=OUT/(name+'.log')
    step=dict(name=name,command=command,returncode=None,log=log.name)
    record['steps'].append(step)
    save()
    print('PUBLICATION_START',name,flush=True)
    with log.open('wb') as stream:
        result=subprocess.run(command,cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT)
    step.update(returncode=result.returncode,log_sha256=sha(log))
    save()
    print('PUBLICATION_DONE',name,result.returncode,flush=True)
    if result.returncode:
        print(log.read_text(errors='replace')[-14000:],flush=True)
        raise SystemExit(result.returncode)


try:
    save()
    proof=json.loads((LAB/'results/tk281/final/qualification.json').read_text())
    assert (proof['total_vm_observations'],proof['unchanged_busybox_workloads'],proof['lab_publication_contracts'])==(230,51,45)
    assert proof['verification_revision_sha256']==sha(LAB/'results/tk281/namespace-consumer/verification-revision.json')
    tool=ROOT/'linux-kernel/tools/archive-publication/archive_publication.py'
    corpus=json.loads((CACHE/'tk281-s4-corpus-2/cohort.json').read_text())
    arguments=corpus['steps'][0]['command']
    payload=Path(arguments[arguments.index('--external-static-user-image')+1])
    current=[LAB/'results/tk281/namespace-consumer',LAB/'results/tk281/final']
    args=[tool,'publish','--payload',payload]
    for archive in current: args+=['--archive',archive]
    run('publish-current',args)
    args=[tool,'hydrate','--payload',payload]
    for archive in current+[
        LAB/'results/tk281/before-v6-consumer-fixture-refresh/namespace-consumer',
        LAB/'results/tk281/before-v6-consumer-fixture-refresh/baseline-and-observer-preflight']:
        args+=['--archive',archive]
    run('hydrate-all',args)
    run('lab-contracts-after-publication',['-m','unittest','discover','-s',LAB,'-p','test_*.py','-v'])
    contracts=(OUT/'lab-contracts-after-publication.log').read_bytes()
    assert b'Ran 45 tests in ' in contracts and contracts.rstrip().endswith(b'OK')
    # Scan every prospective published gzip object, including historical
    # evidence. Retain the audit driver and actual complete command log.
    audit=OUT/'audit_external_payload.py'
    audit.write_text('''from pathlib import Path
import gzip
import hashlib
import json
root=Path('''+repr(str(LAB/'results/tk281'))+''')
payload=Path('''+repr(str(payload))+''').read_bytes()
count=0
for path in root.rglob('*.gz'):
    assert payload not in gzip.decompress(path.read_bytes()),str(path)
    count+=1
print('PUBLISHED_BINARY_BOUNDARY_PASS objects='+str(count)+' dependency_sha256='+hashlib.sha256(payload).hexdigest(),flush=True)
''')
    run('binary-boundary-audit',[audit])
    final=current[1]
    for name in ('lab-contracts-after-publication.log','hydrate-all.log','binary-boundary-audit.log'):
        shutil.copyfile(OUT/name,final/name)
    shutil.copyfile(audit,final/audit.name)
    shutil.copyfile(Path(__file__),final/'publication-finalizer.py')
    proof=json.loads((final/'qualification.json').read_text())
    proof.update(publication_contracts=45,
        publication_contract_log_sha256=sha(final/'lab-contracts-after-publication.log'),
        publication_hydration_log_sha256=sha(final/'hydrate-all.log'),
        publication_binary_audit_log_sha256=sha(final/'binary-boundary-audit.log'),
        publication_finalizer_sha256=sha(final/'publication-finalizer.py'))
    (final/'qualification.json').write_text(json.dumps(proof,indent=2)+'\n')
    record.update(complete=True,passed=True)
    save()
    print('CURRENT_PUBLICATION_PASS vm_observations=230 workloads=51 contracts=45',flush=True)
except BaseException:
    record['complete']=True
    save()
    raise
