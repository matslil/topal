import gzip,json,os,subprocess,sys
from pathlib import Path
a=Path(sys.argv[1]).resolve();work=Path(sys.argv[2]).resolve();root=a.parents[4]
assert work.is_relative_to(root/'linux-kernel');work.mkdir(parents=True,exist_ok=False)
report=json.loads((a/'report.json').read_text())
import hashlib
def load(row):
    p=(a if row['location']=='archive' else root)/row['path'];data=p.read_bytes()
    assert hashlib.sha256(data).hexdigest()==row['packed_sha256']
    raw=gzip.decompress(data);assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'];return raw
manifest=json.loads((a/'manifest.json').read_text())
assert {p.relative_to(a).as_posix() for p in a.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}
for name,row in manifest.items():
    p=(a/name).resolve();assert p.is_relative_to(a);raw=p.read_bytes();assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
for name,row in report['sources'].items():
    p=(work/name).resolve();assert p.is_relative_to(work);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(load(row))
for name,row in report['artifacts'].items():
    p=work/'evidence'/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(load(row))
for case in ['historical','current']:
    for decoder in ['llvm','gnu']:
        name=case+'/'+decoder+'-explicit.stdout.log'
        (work/'evidence'/name).write_bytes(gzip.decompress((a/('observations/'+name+'.gz')).read_bytes()))
sys.path[:0]=[str(work/'linux-kernel/tools/user-memory'),str(work/'linux-kernel/tools/process-memory')]
from run_calendar_data import inspect_case,controls,verify_dependency
from inspect_calendar_data import context,FUNCTION
from inspect_call_inventory import Elf
for p,h in report['dependencies'].items():verify_dependency(root/p,h)
actual={}
for name,address in [('historical',0x2bbaa5d),('current',45861106)]:
    folder=work/'evidence'/name;value=inspect_case(folder,address,name=='current')
    expected=report['qualification']['cases'][name]
    assert value=={k:v for k,v in expected.items() if k!='explicit_start_decoders'}
    e=Elf(folder/'kernel.elf');s,b=e.function(FUNCTION)
    for decoder in ['llvm','gnu']:
        row=context((folder/(decoder+'-explicit.stdout.log')).read_bytes(),b,s['value'],address)
        assert row==[expected['explicit_start_decoders'][decoder]]
    actual[name]=value
assert controls(work/'evidence/current')==report['qualification']['controls']
from archive_publication import closed_archive
try:closed_archive(root/'linux-kernel/tools/user-memory/results/stage-3-paging-controls')
except ValueError as error:assert str(error)=='archive file closure'
else:raise AssertionError('original flat-manifest refusal not reproduced')
namespace={'__name__':'retained_callback_reader'}
raw=gzip.decompress((a/'history/callback-shape/calendar_data.py.gz').read_bytes());exec(compile(raw,'retained-calendar-data.py','exec'),namespace)
graph=json.loads((work/'evidence/current/graph.json').read_text())
try:namespace['guarded_entries'](graph,actual['current']['tables']['year_consumer'])
except TypeError as error:assert 'string indices must be integers' in str(error)
else:raise AssertionError('original callback-shape error not reproduced')
environment=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
for directory in ['user-memory','process-memory']:
    result=subprocess.run(['rtk','proxy','python3','-m','unittest','discover','-s','linux-kernel/tools/'+directory,'-p','test_*.py'],cwd=work,env=environment,capture_output=True)
    (work/(directory+'.stdout.log')).write_bytes(result.stdout);(work/(directory+'.stderr.log')).write_bytes(result.stderr)
    assert result.returncode==0,(directory,result.stderr.decode())
(work/'replayed.json').write_text(json.dumps(dict(passed=True,cases=actual,controls=report['qualification']['controls'],original_failures_reproduced=2,models=337,new_native_execution=False),indent=2)+'\n')
print('Frozen calendar data, graph, source binding, four controls, original reader failures and337models passed')
