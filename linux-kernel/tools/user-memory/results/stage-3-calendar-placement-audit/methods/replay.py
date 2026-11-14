import gzip,hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
a=Path(sys.argv[1]).resolve();root=a.parents[4]
frozen=root/'linux-kernel/.cache/x86_64/calendar-placement-frozen'
report=json.loads((a/'report.json').read_text())
def fact(raw):return dict(bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
manifest=json.loads((a/'manifest.json').read_text())
assert {p.relative_to(a).as_posix() for p in a.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}
for name,row in manifest.items():
    p=(a/name).resolve();assert p.is_relative_to(a) and fact(p.read_bytes())==row
def raw(name):return gzip.decompress((a/(name+'.gz')).read_bytes())
base=root/report['dependency']['path']
assert fact((base/'manifest.json').read_bytes())==report['dependency']['manifest']
b=json.loads((base/'report.json').read_text());q=b['qualification']
for name,sha in q['source_hashes_before'].items():
    assert hashlib.sha256((frozen/name).read_bytes()).hexdigest()==sha
sys.path[:0]=[str(frozen/'linux-kernel/tools/user-memory'),str(frozen/'linux-kernel/tools/process-memory')]
from archive_publication import closed_archive
assert fact(closed_archive(base))==report['dependency']['manifest']
from inspect_call_inventory import Elf,verify_calls
from inventory_paging_controls import inventory
def native(cohort,name):
    data=gzip.decompress((base/('native/'+cohort+'/'+name+'.gz')).read_bytes())
    assert fact(data)==b['build_files'][cohort][name];return data
child=frozen/q['native_builds']['child-exec'];child.mkdir(parents=True,exist_ok=False)
for name in ['kernel.elf','process-mm-users/observations.json']:
    target=child/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(native('child-exec',name))
execution=frozen/'linux-kernel/.cache/x86_64/calendar-placement-stage-1/run.json'
execution.parent.mkdir(parents=True,exist_ok=True)
execution.write_bytes(gzip.decompress((base/'qualification/run.json.gz').read_bytes()))
method=frozen/'linux-kernel/.cache/x86_64/replay-mapping.py';method.write_bytes(raw('methods/mapping.py'))
result=subprocess.run(['rtk','proxy','python3','-B',str(method)],cwd=frozen,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),capture_output=True)
(frozen/'mapping-replay.stdout.log').write_bytes(result.stdout);(frozen/'mapping-replay.stderr.log').write_bytes(result.stderr)
assert result.returncode==0,result.stderr.decode()
actual=json.loads((frozen/'linux-kernel/.cache/x86_64/calendar-placement-mappings-stage-1.json').read_text())
expected=json.loads(raw('mapping/report.json'))
assert {k:v for k,v in actual.items() if k!='resources'}=={k:v for k,v in expected.items() if k!='resources'}
graph=json.loads(native('applications','process-access-inventory.json'))
paging=inventory(child/'kernel.elf',raw('followup/llvm.stdout.log'),raw('followup/gnu.stdout.log'),graph)
assert paging==json.loads(raw('followup/paging-controls.json'))
calls=json.loads(native('applications','extent-call-inventory.json'));verify_calls(Elf(child/'kernel.elf'),calls)
declared={row['offset']:row for row in calls['calls']};covered=set();cases={}
for name in q['applications']['summaries']:
    user=json.loads(native('applications','process-mm-applications/'+name+'/observations.json'))
    coverage=user['extent_call_coverage'];assert user['passed'] and coverage['observation_passed']
    assert coverage['kernel_sha256']==paging['kernel_sha256']
    assert coverage['inventory_sha256']==fact(native('applications','extent-call-inventory.json'))['sha256']
    offsets={row['offset'] for row in coverage['reached']};assert offsets and offsets<=set(declared)
    covered|=offsets;cases[name]=sorted(offsets)
    del user,coverage
saved=json.loads(raw('followup/helper-coverage.json'))
assert cases==saved['cases'] and len(covered)==saved['reached']
assert [declared[n] for n in sorted(set(declared)-covered)]==saved['unsampled']
for name,sha in q['source_hashes_before'].items():assert hashlib.sha256((frozen/name).read_bytes()).hexdigest()==sha
receipt=dict(passed=True,mapping_summary=actual['summary'],selected_controls=paging['selected_controls'],
    full_decoder_control_agreement=paging['full_decoder_control_agreement'],helper_reached=len(covered),helper_declared=len(declared),
    new_native_execution=False,qualification_complete=False)
(frozen/'calendar-audit-replayed.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
print(json.dumps(receipt))
