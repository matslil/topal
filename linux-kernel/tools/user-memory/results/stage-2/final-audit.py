from pathlib import Path
import gzip,hashlib,json,re,subprocess,urllib.parse
root=Path.cwd();out=root/'linux-kernel/tools/user-memory/results/stage-2'
r=json.loads((out/'qualification.json').read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
assert r['complete'] and r['passed'] and not r['stage_complete']
for name,digest in r['sources'].items():assert sha((root/name).read_bytes())==digest,name
for name in ['independent-readback.json','replay.json']:
    assert json.loads((out/name).read_text())['passed']
checked=0
def audit(node,parent):
    global checked
    if isinstance(node,dict):
        if {'file','sha256','uncompressed_sha256','bytes'}<=node.keys():
            packed=(parent/node['file']).read_bytes();assert sha(packed)==node['sha256']
            raw=gzip.decompress(packed);assert sha(raw)==node['uncompressed_sha256'] and len(raw)==node['bytes']
            checked+=1
        else:
            for value in node.values():audit(value,parent)
    elif isinstance(node,list):
        for value in node:audit(value,parent)
for directory in [out,out.with_name('stage-2-review-stop'),out.with_name('stage-2-initial-interrupt-failure')]:
    for path in directory.glob('*.json'):audit(json.loads(path.read_text()),directory)
names=subprocess.check_output(['git','diff','--name-only'],text=True).splitlines()
names += [str(p.relative_to(root)) for p in (root/'linux-kernel/tools/user-memory/results').glob('stage-2*/README.md')]
links=0
for name in sorted(set(names)):
    if not name.endswith('.md'):continue
    path=root/name
    for target in re.findall(r'\[[^\]]*\]\(([^\s)]+)(?:\s+[^)]*)?\)',path.read_text()):
        target=target.strip('<>');parsed=urllib.parse.urlparse(target)
        if parsed.scheme or not parsed.path:continue
        candidate=(path.parent/urllib.parse.unquote(parsed.path)).resolve()
        assert candidate.exists(),(name,target)
        links+=1
record=dict(passed=True,scope='Frozen-source equality, compressed archive identities and changed-document local file links; no additional guest execution or full-stage completion',
    frozen_sources=len(r['sources']),compressed_records_checked=checked,local_markdown_targets=links)
(out/'final-audit.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record,indent=2))
