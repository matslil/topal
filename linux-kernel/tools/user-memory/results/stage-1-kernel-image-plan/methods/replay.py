import gzip,hashlib,json,os,subprocess,sys
from pathlib import Path
a=Path(sys.argv[1]).resolve();work=Path(sys.argv[2]).resolve();root=a.parents[4]
assert work.is_relative_to(root/'linux-kernel');work.mkdir(parents=True,exist_ok=False)
report=json.loads((a/'report.json').read_text());manifest=json.loads((a/'manifest.json').read_text())
def fact(raw):return dict(bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
assert {p.relative_to(a).as_posix() for p in a.rglob('*') if p.is_file()}==set(manifest)|{'manifest.json'}
for name,row in manifest.items():
    p=(a/name).resolve();assert p.is_relative_to(a) and fact(p.read_bytes())==row
for name,row in report['sources'].items():
    base=a if row['location']=='archive' else root;p=(base/row['path']).resolve();assert p.is_relative_to(base)
    compressed=p.read_bytes();assert fact(compressed)['sha256']==row['packed_sha256']
    raw=gzip.decompress(compressed);assert fact(raw)=={k:row[k] for k in ['bytes','sha256']}
    target=(work/name).resolve();assert target.is_relative_to(work)
    target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
sys.path.insert(0,str(work/'linux-kernel/tools/process-memory'))
from archive_publication import closed_archive
assert fact(closed_archive(root/report['dependency']['path']))==report['dependency']['manifest']
for label,count in [('user-memory',271),('process-memory',82)]:
    command=['rtk','proxy','python3','-B','-m','unittest','discover','-s','linux-kernel/tools/'+label,'-p','test_*.py']
    result=subprocess.run(command,cwd=work,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),capture_output=True)
    (work/(label+'.stdout.log')).write_bytes(result.stdout);(work/(label+'.stderr.log')).write_bytes(result.stderr)
    assert result.returncode==0 and ('Ran '+str(count)+' tests').encode() in result.stderr,result.stderr.decode()
(work/'replayed.json').write_text(json.dumps(dict(passed=True,models=353,new_native_execution=False,qualification_complete=False),indent=2)+'\n')
print('Frozen kernel-image planning sources and353models passed; no guest execution')
