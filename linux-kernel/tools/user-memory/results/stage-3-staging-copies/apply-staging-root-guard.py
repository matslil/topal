"""Apply the reviewed guard only after the preceding bounded run releases its lock."""
import ast
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

cache=Path('linux-kernel/.cache/x86_64')
audit=Path('linux-kernel/tools/user-memory/audit_staging_copies.py')
tests=Path('linux-kernel/tools/user-memory/test_staging_copies.py')
identity=json.loads((cache/'staging-copy-before-root-guard-inputs/identity.json').read_text())
assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in identity.items())
old=audit.read_text()
tree=ast.parse(old)
function=next(node for node in tree.body if isinstance(node,ast.FunctionDef)
              and node.name=='bounded_count_initializers')
original=ast.get_source_segment(old,function)
candidate=(cache/'bounded-count-root-guard-candidate.py').read_text().rstrip()
assert old.count(original)==1
updated=old.replace(original,candidate)
call='bounded_count_initializers(nodes,predecessors,start)'
assert updated.count(call)==1
updated=updated.replace(call,'bounded_count_initializers(nodes,predecessors,start,roots)')
test_source=tests.read_text()
call='bounded_count_initializers(nodes,preds,42)'
assert test_source.count(call)==3
test_source=test_source.replace(call,'bounded_count_initializers(nodes,preds,42,set())')
marker='    def observer(self):'
assert test_source.count(marker)==1
additional='''    def test_a_root_with_predecessors_does_not_inherit_their_count(self):
        nodes,preds = self.count_graph()
        for entry in [10,17,33,36]:
            with self.subTest(entry=entry), self.assertRaises(AssertionError):
                bounded_count_initializers(nodes,preds,42,{entry})

    def test_a_root_at_the_consumer_cannot_inherit_an_earlier_count(self):
        nodes,preds = self.count_graph()
        with self.assertRaises(AssertionError):
            bounded_count_initializers(nodes,preds,42,{42})

    def test_a_root_which_sets_a_bounded_count_remains_safe(self):
        nodes,preds = self.count_graph()
        definitions = bounded_count_initializers(nodes,preds,42,{0,23})
        self.assertEqual({row['value'] for row in definitions},{2,5})

'''
test_source=test_source.replace(marker,additional+marker)
ast.parse(updated)
ast.parse(test_source)
work=cache/'staging-root-guard-preflight'
work.mkdir()
audit.write_text(updated)
tests.write_text(test_source)
command=[sys.executable,'-m','unittest','discover','-s',str(audit.parent),'-p',tests.name,'-v']
with (work/'unit-tests.log').open('w') as log:
    result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
assert result.returncode==0,(work/'unit-tests.log').read_text()
original_build=cache/'user-extent-application-2334987-1794428811789108512'
build=work/'positive-copy'
build.mkdir()
names=['provider.o','kernel.elf','extent-call-inventory.json','process-access-inventory.json',
       'process-access-origins.json']
for name in names:shutil.copyfile(original_build/name,build/name)
command=[sys.executable,str(audit),str(build)]
with (work/'positive-audit.log').open('w') as log:
    result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
assert result.returncode==0,(work/'positive-audit.log').read_text()
new=(build/'staging-copy-audit.json').read_bytes()
assert new==(original_build/'staging-copy-audit.json').read_bytes()
report=dict(passed=True,unit_tests=7,positive_report_byte_identical=True,
    positive_report_sha256=hashlib.sha256(new).hexdigest(),
    source_sha256={str(path):hashlib.sha256(path.read_bytes()).hexdigest() for path in [audit,tests]},
    scope='Pure reader strengthening, synthetic graph controls and current-image report equivalence; no new native execution claim')
(work/'preflight.json').write_text(json.dumps(report,indent=2)+'\n')
catalog_path=cache/'staging-copies-catalog.json'
catalog=json.loads(catalog_path.read_text())
catalog['root_guard_preflight']=str(work)
catalog['notes'].append('Root guard was applied after preceding bounded run released exclusive lock and before poll retry source capture. Seven controls PASS and current actual-kernel staging report remains byte-identical. Pre-guard reader/test snapshots retained for earlier native/library versions. Post suite must now count52 user-memory Python tests plus4 decoder tests. Full library/native pass status remains separate.')
catalog_path.write_text(json.dumps(catalog,indent=2)+'\n')
print(json.dumps(report),flush=True)
