"""Bind pre-format regression/external execution to the exact rebuilt kernel."""
import hashlib
import json
from pathlib import Path
import sys

build,output=map(Path,sys.argv[1:3])
output.mkdir(parents=True,exist_ok=True)
root=Path('linux-kernel/.cache/x86_64')
prior=[root/name for name in [
    'user-extent-source-poll-2263956-1794395971230199800',
    'user-extent-source-2264433-1794396085975218237',
    'user-extent-application-2264804-1794396372582602747']]
digest=lambda data:hashlib.sha256(data).hexdigest()
names=['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
       'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
       'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage']
comparisons=[]
for name in names:
    data=(build/name).read_bytes()
    for other in prior: assert data==(other/name).read_bytes(),(str(other),name)
    comparisons.append(dict(path=name,bytes=len(data),sha256=digest(data),matching_preformat_builds=3))
assert (build/'static-elf-abi-fixture.elf').read_bytes()==(prior[0]/'static-elf-abi-fixture.elf').read_bytes()
old=Path('linux-kernel/tools/user-memory/results/stage-3-owned-poll-format-failure/before-format.rs').read_bytes()
new=Path('src/topal-compiler/src/systems_artifact_static_image.rs').read_bytes()
single=b'fn static_user_image_fixture_with_source(component: Option<(u32, Option<u32>, Option<u32>, &[u8])>) -> Vec<u8> {'
wrapped=b'fn static_user_image_fixture_with_source(\n    component: Option<(u32, Option<u32>, Option<u32>, &[u8])>,\n) -> Vec<u8> {'
assert old.count(single)==1 and old.replace(single,wrapped)==new
external=(prior[2]/'static-elf-abi-fixture.elf').read_bytes()
assert digest(external)=='7ae95c260622412ceba6eb9f45d68f616cdc091a1d71d6c0a08b5875bb2c31dd'
raw=(prior[2]/'initramfs.cpio').read_bytes()
members={}
offset=0
while True:
    assert raw[offset:offset+6]==b'070701'
    length=int(raw[offset+54:offset+62],16)
    size=int(raw[offset+94:offset+102],16)
    name=raw[offset+110:offset+110+size-1].decode()
    offset=(offset+110+size+3)&-4
    if name=='TRAILER!!!':
        assert length==0 and not any(raw[offset:])
        break
    assert name not in members
    members[name]=raw[offset:offset+length]
    offset=(offset+length+3)&-4
assert set(members)=={'init','static-user.elf'} and members['static-user.elf']==external
assert json.loads((prior[2]/'first-image-exit/observations.json').read_text())['passed']
record=dict(passed=True,comparisons=comparisons,preformat_source_sha256=digest(old),
    formatted_source_sha256=digest(new),formatting_change='Exact single-signature line wrapping; no other byte change',
    external_image_sha256=digest(external),external_resource_sha256=digest(raw),
    scope='Three prior native runs used the pre-format producer. Their exact kernel/boot bytes match the fresh formatted-producer native run; they are not reported as executions from the formatted source.')
(output/'rebuild-equivalence.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(dict(passed=True,objects=len(names),matching_preformat_builds=3,external_image_sha256=digest(external))))
