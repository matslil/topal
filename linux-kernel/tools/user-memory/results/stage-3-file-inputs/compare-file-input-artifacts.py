"""Compare unchanged core kernel artifacts across current file-input cohorts."""
import hashlib
import json
from pathlib import Path

cache=Path('linux-kernel/.cache/x86_64')
baseline=cache/'user-extent-source-pipes-2379391-1794452959731792752'
builds=dict(storage=cache/'user-extent-source-storage-2382778-1794455130144072641',
    source=cache/'user-extent-source-2383420-1794455566798945157',
    pinned=cache/'user-extent-application-2383173-1794455447746904465')
core=['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o','gateway.o',
    'boundary.o','extent.o','extent-callers.o','extent-bridge.o','predecessor-provider.o',
    'predecessor-gateway.o','provider.o','kernel.elf','bzImage']
metadata=['extent-call-inventory.json','process-access-inventory.json','process-access-origins.json',
    'process-byte-origins.json','fixed-transfer-audit.json','copy-transfer-audit.json','staged-transfer-audit.json',
    'additional-scalar-audit.json','staging-copy-audit.json','pipe-fixed-output-audit.json',
    'stat-output-audit.json','serial-prefix-audit.json','pipe-loop-audit.json']
digest=lambda data:hashlib.sha256(data).hexdigest()
comparisons={}
for label,build in builds.items():
    comparisons[label]=[]
    for name in core+metadata:
        raw=(build/name).read_bytes()
        assert raw==(baseline/name).read_bytes(),(label,name)
        comparisons[label].append(dict(path=name,sha256=digest(raw),bytes=len(raw)))
old_source=cache/'user-extent-source-2380225-1794453340010915677'
source_images=[]
for name in ['static-elf-abi-fixture.elf','initramfs.cpio']:
    raw=(builds['source']/name).read_bytes()
    assert raw==(old_source/name).read_bytes()
    source_images.append(dict(path=name,sha256=digest(raw),bytes=len(raw)))
report=dict(passed=True,baseline_build=str(baseline),core_objects_images_per_cohort=len(core),
    metadata_reports_per_cohort=len(metadata),comparisons=comparisons,default_source_images_equal=source_images,
    scope='Actual byte comparisons of seventeen core objects/images and thirteen metadata/reports in each current native cohort, plus unchanged default source application/boot bytes; not a complete clean producer or ownership/lifetime qualification')
(cache/'file-input-artifact-comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,core_per_cohort=len(core),metadata_per_cohort=len(metadata),cohorts=3,default_images_equal=2)))
