"""Verify selected saved-code inventory evidence without claiming qualification."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

from replay import digest,read,root,unpack

repo=root.parents[4]
meta=read(root/'inputs.json')
sources={name:unpack(row) for name,row in meta['sources'].items()}
artifacts={name:unpack(row) for name,row in meta['artifacts'].items()}
records={name:unpack(row) for name,row in meta['records'].items()}
assert (len(sources),len(artifacts),len(records))==(3,2,2)
for name,raw in sources.items():assert (repo/name).read_bytes()==raw
base=meta['base_artifact_metadata']
base_path=(root/base['path']).resolve()
assert base_path.is_relative_to(repo)
raw=base_path.read_bytes()
assert len(raw)==base['bytes'] and digest(raw)==base['sha256']
base_rows=json.loads(raw)
for name,row in meta['artifacts'].items():
    old=base_rows[name]
    assert {k:v for k,v in row.items() if k!='snapshot'}=={k:v for k,v in old.items() if k!='snapshot'}
    assert (root/row['snapshot']).resolve()==(base_path.parent/old['snapshot']).resolve()
report=read(root/'inventory.json')
assert report['passed'] and report['kernel_sha256']==digest(artifacts['kernel.elf'])
assert report['kernel_sha256']=='47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f'
assert report['process_graph_sha256']==digest(artifacts['process-access-inventory.json'])
assert report['inventory_sha256']==digest(sources['linux-kernel/tools/user-memory/inventory_paging_controls.py'])
for label in ['llvm','gnu']:
    assert report[label+'_listing_sha256']==digest(records[label+'.stdout.log'])
    command=read(root/(label+'.command.json'))
    assert command['exit_code']==0 and '-d' in command['argv'] and '-z' in command['argv']
    assert (root/(label+'.stderr.log')).read_bytes()==b''
assert (report['selected_controls'],report['process_graph_controls'],report['executable_section']['bytes'])==(847,523,1267581)
assert (report['llvm_unknown_rows'],report['gnu_bad_rows'],report['llvm_overlap_bytes'],report['gnu_overlap_bytes'])==(63,60,153,0)
assert (report['llvm_prefix_fragment_rows'],report['gnu_prefix_fragment_rows'])==(3,1)
assert not report['full_decoder_control_agreement']
assert report['unmatched_control_addresses']==[0x2bbaa5d]
unmatched=[r for r in report['controls'] if not r['decoder_agreement']]
assert len(unmatched)==1 and not unmatched[0]['in_selected_process_graph']
assert report['embedded_or_alternate_candidates']==1
assert len(report['raw_opcode_candidates'])==len(report['gnu_raw_opcode_candidates'])==67
assert [(r['address'],r['opcode_bytes'],r['kind']) for r in report['raw_opcode_candidates']]==[(r['address'],r['opcode_bytes'],r['kind']) for r in report['gnu_raw_opcode_candidates']]
qualification=read(root/'qualification.json')
assert qualification['passed'] and not qualification['new_native_execution']
assert qualification['source_hashes_before']==qualification['source_hashes_after']=={name:digest(raw) for name,raw in sources.items()}
assert qualification['input_hashes_before']==qualification['input_hashes_after']=={name:digest(raw) for name,raw in dict(artifacts,**records).items()}
assert qualification['resources']['memory.swap.max']=='0'
assert all(int(line.split()[1])==0 for line in qualification['resources']['memory.events'].splitlines())
assert [row['label'] for row in qualification['commands']]==['models','inventory']
for row in qualification['commands']:
    assert row['exit_code']==0 and digest((root/(row['label']+'.log')).read_bytes())==row['log_sha256']
assert b'Ran 9 tests' in (root/'models.log').read_bytes()
for ordinal in ['initial','second','third','fourth']:
    failure=read(root/(ordinal+'-failure.json'))
    assert not failure['passed'] and not failure['native_execution'] and not failure['exact_original_stderr_retained']
    assert digest(gzip.decompress((root/(ordinal+'-inventory.py.gz')).read_bytes()))==failure['source_sha256']
replay=read(root/'replay.json')
assert replay['passed'] and replay['exact_inventory_equal'] and replay['reader_tests']==9
assert not replay['new_native_execution'] and not replay['full_decoder_control_agreement']
assert replay['selected_sources']==3 and replay['inherited_artifacts']==2
assert b'Ran 9 tests' in (root/'replay-models.log').read_bytes()
links=[]
for doc in [root/'README.md',repo/'linux-kernel/README.md',repo/'linux-kernel/step-4-status.md',
        repo/'linux-kernel/step-4-owned-user-extents.md',repo/'linux-kernel/tools/user-memory/caller-routing.md']:
    for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',doc.read_text()):
        if '://' in target or target.startswith('#'):continue
        assert (doc.parent/target.split('#',1)[0]).resolve().is_file(),(doc,target)
        links.append(dict(document=str(doc.relative_to(repo)),target=target))
files={}
for path in sorted(root.rglob('*')):
    if not path.is_file() or path.name=='manifest.json':continue
    assert '__pycache__' not in path.parts and path.suffix not in ['.pyc','.pyo']
    data=path.read_bytes()
    files[str(path.relative_to(root))]=dict(sha256=digest(data),bytes=len(data))
manifest=dict(passed=True,files=files,source_count=3,inherited_artifacts=2,listing_records=2,local_links=links,
    full_decoder_control_agreement=False,
    scope='Selected bytes, inventory bindings, failure summaries and exact frozen reader replay. Not resolved interpretation/entry/mode/TLB or kernel production qualification')
if '--check' in sys.argv[1:]:assert read(root/'manifest.json')==manifest
else:(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(passed=True,archive_files=len(files),selected_sources=3,inherited_artifacts=2,
    listed_controls=847,unmatched_controls=1,reader_tests=9)))
