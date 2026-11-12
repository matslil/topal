#!/usr/bin/env python3
"""Archive a completed experimental process/MM run with exact inputs."""
import argparse
import gzip
import json
from pathlib import Path

from readback_process import digest, readback
from process_native import require

ROOT = Path(__file__).resolve().parents[3]
SELECTED = [
    'AGENTS.md',
    'docs/systems-profile.md',
    'se/requirements.md',
    'spec/systems-profile.md',
    'src/topal-compiler/se-requirements.md',
    'linux-kernel/design/process-memory-ownership.md',
    'linux-kernel/design/decision-register.md',
    'linux-kernel/step-4-process-memory-ownership.md',
    'linux-kernel/step-4-status.md',
    'linux-kernel/tools/process-memory/README.md',
    'src/topal-compiler/src/systems_process_mm.rs',
    'src/topal-compiler/src/runtime/kernel_process_mm.ll',
    'src/topal-compiler/src/runtime/kernel_process_mm_context.ll',
    'src/topal-compiler/src/runtime/kernel_process_mm_owners.ll',
    'src/topal-compiler/src/systems_frame_gateway_process_memory.rs',
    'src/topal-compiler/src/systems_frame_gateway_memory.rs',
    'src/topal-compiler/src/systems_frame_gateway.rs',
    'src/topal-compiler/src/systems_frame_root_tests.rs',
    'src/topal-compiler/src/systems_frame_target.rs',
    'src/topal-compiler/src/systems_provider_emit_process_entry.rs',
    'src/topal-compiler/src/systems_provider_emit_publication.rs',
    'src/topal-compiler/src/systems_provider_emit_translation.rs',
    'src/topal-compiler/src/systems_provider_object.rs',
    'src/topal-compiler/src/systems_process_children.rs',
    'src/topal-compiler/src/systems_process_pair.rs',
    'src/topal-compiler/src/systems_user_source_fixture.rs',
    'src/topal-compiler/src/systems_frame_inspection.rs',
    'src/topal-compiler/src/runtime/kernel_frame_memory.ll',
    'src/topal-compiler/src/runtime/kernel_frame_memory_lease.ll',
    'src/topal-compiler/src/runtime/kernel_frame_controller.ll',
    'src/topal-compiler/src/runtime/kernel_frame_target.ll',
    'src/topal-compiler/src/runtime/kernel_frame_gateway_lease.ll',
    'src/topal-compiler/src/systems_user_extent.rs',
    'src/topal-compiler/src/systems_user_extent_clean.rs',
    'src/topal-compiler/src/runtime/kernel_user_extent_target.ll',
    'src/topal-compiler/src/runtime/kernel_user_extent_clean.ll',
    'linux-kernel/kernel/memory/frame-metadata.t',
    'linux-kernel/labs/qemu/x86_64/process_mm_process_inspect.py',
    'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_cases.c',
    'linux-kernel/labs/qemu/x86_64/frame_root_inspect.py',
    'linux-kernel/tools/process-memory/process_native.py',
    'linux-kernel/tools/process-memory/readback_process.py',
    'linux-kernel/tools/process-memory/archive_process.py',
    'linux-kernel/tools/process-memory/replay_process.py',
]
PRESERVED = [
    'source.o', 'runtime.o', 'owner.o', 'controller.o', 'boundary.o', 'root.o',
    'extent.o', 'extent-bridge.o', 'extent-callers.o',
    'predecessor-provider.o', 'predecessor-gateway.o',
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build', type=Path)
    parser.add_argument('--reference', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--attempt', action='append',default=[],type=Path)
    args = parser.parse_args()
    build, reference, output = args.build.resolve(), args.reference.resolve(), args.output.resolve()
    require(output.is_relative_to(ROOT/'linux-kernel'), 'Linux output directory')
    raw = (build/'process-mm-users/observations.json').read_bytes()
    report = readback(json.loads(raw))
    frame = json.loads((build/'frame-inspection.json').read_text())
    for key in ['complete_stack_bytes','complete_physical_bytes']:
        require(frame['user_extent_callers'][key] <= 4096, 'complete native bound')
    require({'p.prepare','p.commit','p.switch','p.child_exit','p.cancel_inactive'} <= set(frame['calls']), 'native caller closure')
    preserved = {}
    pairs = [(name,name) for name in PRESERVED] + [
        ('retained-owned-provider.o','provider.o'), ('retained-owned-gateway.o','gateway.o')]
    for actual, old in pairs:
        payload = (build/actual).read_bytes()
        require(payload == (reference/old).read_bytes(), 'changed predecessor artifact: '+actual)
        preserved[actual] = dict(reference=old, **digest(payload))
    component = json.loads((build/'source-component.json').read_text())
    source = ROOT/'linux-kernel/labs/qemu/x86_64/user_extent_process_mm_cases.c'
    require(component['process_mm_source_sha256'] == digest(source.read_bytes())['sha256'], 'application source binding')
    require((build/'process-mm-cases.c').read_bytes()==source.read_bytes(), 'compiled application source differs')
    inspector=(build/'process-mm-users/inspect.gdb').read_text()
    model=(ROOT/'linux-kernel/tools/process-memory/process_native.py').read_text()
    require(inspector.count('exec('+repr(model)+',model)')==1, 'actual debugger model differs')
    require(component['image_sha256'] == digest((build/'static-elf-abi-fixture.elf').read_bytes())['sha256'], 'application image binding')
    resources = json.loads((build/'resource-observation.json').read_text())
    require(resources['swap_max_bytes']==0 and all(v==0 for v in resources['events'].values()), 'observed resource failure')
    verification = json.loads((build/'llvm-verification.json').read_text())
    require(set(verification)=={'source','runtime','owner','controller','target','dispatch','extent'}, 'LLVM module closure')
    artifacts = {p.name:digest(p.read_bytes()) for p in sorted(build.iterdir())
                 if p.suffix in ['.ll','.o','.elf'] or p.name in ['bzImage','initramfs.cpio']}
    for role, result in verification.items():
        require(result['passed'] and result['sha256']==artifacts[role+'.ll']['sha256'], 'LLVM verification binding')
    validation=json.loads((build/'validation.json').read_text())
    require(set(validation)=={'user-memory-models','process-memory-models','default-ABI-fixture','format','Rust-file-ceiling'}
            and all(r['exit_code']==0 for r in validation.values()), 'regression validation incomplete')
    report.update(schema='topal-process-mm-native-integration-v1', observation=digest(raw),
        selected_inputs={name:digest((ROOT/name).read_bytes()) for name in SELECTED},
        artifacts=artifacts, preserved=preserved, native_bounds=frame['user_extent_callers'],
        regression_validation=validation,
        last_resource_sample=resources, llvm_verified_modules=sorted(verification),
        remaining=['child exec with inactive parent','inactive live-child cancellation',
                   'native zero/one/maximum/partial heaps and resource failures',
                   'identity width exhaustion and fatal partial-effect cleanup',
                   'every mapping producer/caller and continuous loan/mode/exclusion proof',
                   'full post-INVLPG and pre-release publication observations',
                   'all 51 application/pipe regressions and cold public reconstruction',
                   'original full Linux/distribution/container/VM-host objectives'])
    output.mkdir(parents=True,exist_ok=False)
    archive = {'native-observations.json':raw}
    for name in SELECTED:
        archive['inputs/'+name] = (ROOT/name).read_bytes()
    for p in sorted(build.iterdir()):
        if p.is_file() and (p.suffix in ['.ll','.c','.py'] or p.name.endswith(('.command.json','.stdout.log','.stderr.log'))
            or p.name in ['source-component.json','frame-inspection.json','llvm-verification.json',
                          'resource-observation.json','process-byte-origins.json','extent-call-inventory.json',
                          'validation.json','readback.json']):
            archive['build/'+p.name] = p.read_bytes()
    for p in sorted((build/'process-mm-users').iterdir()):
        if p.is_file() and p.name!='observations.json':
            archive['native/'+p.name] = p.read_bytes()
    report['attempts']=[]
    for index,attempt in enumerate(args.attempt):
        attempt=attempt.resolve()
        require(attempt.is_relative_to(ROOT/'linux-kernel'), 'Linux attempt directory')
        identity={p.name:digest(p.read_bytes()) for p in sorted(attempt.iterdir())
                  if p.is_file() and p.suffix in ['.o','.ll','.elf']}
        report['attempts'].append(dict(artifacts=identity,scope='Earlier failed validation; no passing claim'))
        for p in sorted(attempt.iterdir()):
            if p.is_file() and (p.suffix in ['.ll','.c'] or p.name.endswith('.command.json')):
                archive[f'history/{index}/'+p.name]=p.read_bytes()
        for directory in sorted(attempt.glob('process-mm-users*')):
            if directory.is_dir():
                for p in sorted(directory.iterdir()):
                    if p.is_file():
                        archive[f'history/{index}/'+directory.name+'/'+p.name]=p.read_bytes()
    for name,payload in archive.items():
        target = output/(name+'.gz')
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(gzip.compress(payload,mtime=0))
    (output/'report.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    manifest = {p.relative_to(output).as_posix():digest(p.read_bytes())
                for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps({k:report[k] for k in ['passed','cases','operations','copied_heap_bytes',
                                         'erased_released_bytes','retired_extents','native_bounds']}))


if __name__ == '__main__':
    main()
