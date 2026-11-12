"""Bind tested predicate bytes to the actual target and linked guest kernel."""
import json
from pathlib import Path
import sys

sys.path.insert(0,str(Path('linux-kernel/tools/user-memory').resolve()))
from inspect_call_inventory import Elf,digest

old=Path(sys.argv[1])
new=Path(sys.argv[2])
component=Path(sys.argv[3])
output=Path(sys.argv[4])
before=Elf(old/'target.o')
after=Elf(new/'target.o')
probe=Elf(component/'clean.o')
_,expected=probe.function('topal_test_clean_wide')
_,expected_old=probe.function('topal_test_clean_old')
old_symbol,actual_old=before.function('t.clean')
new_symbol,actual=after.function('t.clean')
_,linked=Elf(new/'kernel.elf').function('t.clean')
assert actual_old==expected_old
assert actual==linked==expected
assert not before.relocations(old_symbol['section'])
assert not after.relocations(new_symbol['section'])
def functions(elf):
    return {name:s for name,s in elf.symbols.items()
            if s['section'] and s['info']&15==2 and s['size']}
old_functions,new_functions=functions(before),functions(after)
assert set(old_functions)==set(new_functions)
unchanged=[]
for name in sorted(old_functions):
    if name=='t.clean':continue
    a,abody=before.function(name)
    b,bbody=after.function(name)
    assert abody==bbody,('other target function changed',name)
    def relocations(elf,symbol,body):
        return {at-symbol['value']:row for at,row in elf.relocations(symbol['section']).items()
                if symbol['value']<=at<symbol['value']+len(body)}
    assert relocations(before,a,abody)==relocations(after,b,bbody),name
    unchanged.append(name)
objects=['root.o','source.o','runtime.o','owner.o','controller.o','dispatch.o',
         'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o',
         'provider.o','predecessor-provider.o','predecessor-gateway.o']
for name in objects:
    assert (old/name).read_bytes()==(new/name).read_bytes(),('other object changed',name)
report=dict(passed=True,old_kernel_sha256=digest((old/'kernel.elf').read_bytes()),
    kernel_sha256=digest((new/'kernel.elf').read_bytes()),
    old_target_sha256=digest(before.data),target_sha256=digest(after.data),
    old_function=dict(bytes=len(actual_old),sha256=digest(actual_old)),
    function=dict(bytes=len(actual),sha256=digest(actual)),
    component_object_sha256=digest((component/'clean.o').read_bytes()),
    unchanged_target_functions=unchanged,unchanged_objects=objects,
    scope='Exact tested predicate bytes in actual target and linked kernel; other target function bodies/relocations and fourteen component/provider/predecessor objects unchanged. Not full image reconstruction or native ownership/lifetime qualification.')
output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k not in ['unchanged_target_functions','unchanged_objects']}
    |dict(unchanged_target_function_count=len(unchanged),unchanged_object_count=len(objects))))
