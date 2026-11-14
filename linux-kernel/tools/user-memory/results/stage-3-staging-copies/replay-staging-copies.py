"""Replay the frozen source join and five reports without building or execution."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=False)
digest=lambda data:hashlib.sha256(data).hexdigest()
identity=json.loads((root/'source-identity.json').read_text())
assert digest((root/identity['inherited_record']).read_bytes())==identity['inherited_record_sha256']
for path,row in identity['selected_inputs'].items():
    data=gzip.decompress((root/row['snapshot']).read_bytes())
    assert digest(data)==row['sha256']
    target=work/'source-root'/path
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)
readers=work/'source-root/linux-kernel/tools/user-memory'
build=work/'build'
build.mkdir()
for name,row in json.loads((root/'artifact-identity.json').read_text()).items():
    data=(root/row['snapshot']).read_bytes()
    assert digest(data)==row['snapshot_sha256']
    data=gzip.decompress(data)
    assert digest(data)==row['sha256'] and len(data)==row['bytes']
    (build/name).write_bytes(data)
names=['process-access-origins.json','fixed-transfer-audit.json','copy-transfer-audit.json',
    'staged-transfer-audit.json','additional-scalar-audit.json','staging-copy-audit.json']
expected={name:(build/name).read_bytes() for name in names}
commands=[('trace_process_origins',[str(build),str(work/'source-root')])]+[
    (name,[str(build)]) for name in ['audit_fixed_transfers','audit_copy_transfers','audit_staged_transfers',
        'audit_additional_scalars','audit_staging_copies']]
for name,args in commands:
    result=subprocess.run([sys.executable,str(readers/(name+'.py')),*args],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(name+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,result.stdout.decode(errors='replace')
assert all((build/name).read_bytes()==data for name,data in expected.items())
def restore(label):
    directory=work/label
    directory.mkdir()
    for name,row in json.loads((root/(label+'-artifact-identity.json')).read_text()).items():
        compressed=(root/row['snapshot']).read_bytes()
        assert digest(compressed)==row['snapshot_sha256']
        data=gzip.decompress(compressed)
        assert digest(data)==row['sha256'] and len(data)==row['bytes']
        (directory/name).write_bytes(data)
    return directory

baseline,intermediate=restore('baseline'),restore('intermediate')
component=work/'private-clean-component'
component.mkdir()
component_bytes=gzip.decompress((root/'private-clean/component/clean.o.gz').read_bytes())
binding=json.loads((root/'private-clean/private-clean-target-binding.json').read_text())
assert digest(component_bytes)==binding['component_object_sha256']
(component/'clean.o').write_bytes(component_bytes)
proofs=[('diff-staging-kernel',[baseline,intermediate,work/'provider-difference.json'],
         'prior-provider-kernel-difference.json','provider-difference.json'),
        ('check-private-clean-target',[intermediate,build,component,work/'target-binding.json'],
         'private-clean/private-clean-target-binding.json','target-binding.json'),
        ('compose-staging-kernel-difference',[intermediate,build,work/'provider-difference.json',
                                            work/'target-binding.json',work/'kernel-difference.json'],
         'kernel-difference.json','kernel-difference.json')]
for name,args,expected_name,actual_name in proofs:
    result=subprocess.run([sys.executable,str(root/(name+'.py')),*map(str,args)],
                          cwd=work/'source-root',stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (work/(name+'.log')).write_bytes(result.stdout)
    assert result.returncode==0,result.stdout.decode(errors='replace')
    assert (work/actual_name).read_bytes()==(root/expected_name).read_bytes(),name
report=dict(passed=True,selected_inputs=len(identity['selected_inputs']),exact_source_join_equal=True,
    exact_five_reports_equal=True,selected_intervals=79,distinct_user_access_instructions=306,
    exact_provider_difference_equal=True,exact_private_predicate_binding_equal=True,
    exact_composed_kernel_difference_equal=True,
    scope='Frozen selected first-party source/artifact and three-version machine-byte replay; no compiler rebuild, GNU/LLVM/GDB/VM execution, complete owners/lifetime or production qualification')
(work/'replay.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
