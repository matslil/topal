#!/usr/bin/env python3
"""Bind saved native preparation/cancellation evidence without activating it."""
import argparse
from collections import Counter
from copy import deepcopy
import gzip
import hashlib
import json
from pathlib import Path
import struct

from preparation_native import check, require

ROOT = Path(__file__).resolve().parents[3]
SELECTED = [
    'src/topal-compiler/src/systems_process_mm.rs',
    'src/topal-compiler/src/runtime/kernel_process_mm.ll',
    'src/topal-compiler/src/systems_frame_target.rs',
    'src/topal-compiler/src/systems_frame_root_tests.rs',
    'src/topal-compiler/src/systems_frame_gateway_memory.rs',
    'src/topal-compiler/src/runtime/kernel_frame_memory.ll',
    'src/topal-compiler/src/runtime/kernel_frame_memory_lease.ll',
    'src/topal-compiler/src/runtime/kernel_frame_controller.ll',
    'src/topal-compiler/src/runtime/kernel_frame_target.ll',
    'src/topal-compiler/src/runtime/kernel_frame_gateway_lease.ll',
    'src/topal-compiler/src/systems_frame_inspection.rs',
    'src/topal-compiler/src/systems_user_extent.rs',
    'src/topal-compiler/src/systems_user_extent_clean.rs',
    'src/topal-compiler/src/runtime/kernel_user_extent_target.ll',
    'src/topal-compiler/src/runtime/kernel_user_extent_clean.ll',
    'linux-kernel/kernel/memory/frame-metadata.t',
    'linux-kernel/tools/user-memory/results/stage-3-source-metadata/readback-source-metadata.py',
    'linux-kernel/labs/qemu/x86_64/user_extent_source_inspect.py',
    'linux-kernel/labs/qemu/x86_64/process_mm_preparation_observer.py',
    'linux-kernel/tools/process-memory/replay_preparation.py',
    'linux-kernel/tools/process-memory/preparation_native.py',
    'linux-kernel/tools/process-memory/readback_preparation.py',
]
PRESERVED = [
    'source.o', 'runtime.o', 'owner.o', 'controller.o', 'boundary.o', 'provider.o',
    'root.o', 'extent.o', 'extent-bridge.o', 'extent-callers.o', 'gateway.o',
    'predecessor-provider.o', 'predecessor-gateway.o', 'static-elf-abi-fixture.elf',
    'initramfs.cpio',
]


def digest(raw):
    return dict(sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))


def source_elf_delta(actual, original):
    def read(raw):
        require(raw[:6] == b'\x7fELF\x02\x01', 'source ELF format')
        phoff, shoff = struct.unpack_from('<QQ', raw, 32)
        phsize, phnum, shsize, shnum, strings = struct.unpack_from('<HHHHH', raw, 54)
        headers = [struct.unpack_from('<IIQQQQIIQQ', raw, shoff + i * shsize) for i in range(shnum)]
        string = headers[strings]
        names = raw[string[4]:string[4] + string[5]]
        sections = {names[h[0]:names.index(0, h[0])].decode(): (h, raw[h[4]:h[4] + h[5]]) for h in headers}
        loads = []
        for i in range(phnum):
            p = struct.unpack_from('<IIQQQQQQ', raw, phoff + i * phsize)
            if p[0] == 1:
                loads.append((p, raw[p[2]:p[2] + p[5]]))
        return sections, loads
    a, loads_a = read(actual)
    b, loads_b = read(original)
    require(loads_a == loads_b and actual[16:40] == original[16:40], 'source ELF load/entry changed')
    require(set(a) == set(b), 'source ELF section set changed')
    allowed = {'.debug_info', '.debug_line', '.debug_line_str'}
    changed = {}
    for name in a:
        h, raw = a[name]
        previous, old = b[name]
        require(tuple(h[i] for i in [1,2,3,6,7,8,9]) == tuple(previous[i] for i in [1,2,3,6,7,8,9]), 'source section meaning changed: ' + name)
        if raw != old:
            require(name in allowed and h[2] & 2 == 0, 'unexpected source ELF byte change: ' + name)
            changed[name] = dict(before=digest(old), after=digest(raw))
    return dict(loadable_bytes_equal=True, entry_equal=True,
                other_sections_equal=True, changed_nonallocated_debug_sections=changed,
                scope='Exact loaded bytes, entry and other sections; debug bytes differ and no debug-equivalence proof is claimed')


def controls(record):
    require(record['comparison']['pages'] > 34, 'control needs actual large heap')
    paths = [
        ('unpinned-preparation', ['ready', 'prepared', 'record', 6], 0),
        ('wrong-epoch', ['ready', 'prepared_state', 6], 0),
        ('wrong-context', ['ready', 'prepared_state', 5], 0),
        ('wrong-root', ['ready', 'prepared_state', 4], 0),
        ('restored-issuance', ['after', 'allocator', 2], record['before']['allocator'][2]),
        ('nonzero-erasure', ['erased', 'bytes'], '01' + record['erased']['bytes'][2:]),
        ('erase-still-pinned', ['erased', 'record', 6], 1),
        ('unconsumed-registry', ['after', 'released_record', 1], 1),
        ('unreleased-cell', ['after', 'released_cells', 0], record['ready']['prepared']['token'][1]),
        ('retained-receipt', ['after', 'prepared_state', 6], 1),
        ('changed-parent', ['after', 'active', 'bytes'],
         bytes([int(record['after']['active']['bytes'][:2], 16) ^ 1]).hex() + record['after']['active']['bytes'][2:]),
        ('truncated-to-34-pages', ['ready', 'prepared', 'bytes'], record['ready']['prepared']['bytes'][:34*4096*2]),
    ]
    outcomes = []
    for name, path, value in paths:
        changed = deepcopy(record)
        target = changed
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        try:
            check(changed)
        except ValueError as error:
            outcomes.append(dict(name=name, rejected=True, reason=str(error)))
        else:
            raise AssertionError('accepted saved-data control: ' + name)
    changed = deepcopy(record)
    raw = bytearray.fromhex(changed['ready']['prepared']['bytes'])
    raw[34 * 4096] ^= 1
    changed['ready']['prepared']['bytes'] = raw.hex()
    try:
        check(changed)
    except ValueError as error:
        outcomes.append(dict(name='wrong-copy-beyond-page-34', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted incomplete heap copy')
    changed = deepcopy(record)
    a = changed['ready']['prepared']
    first = changed['before']['active']['cells'][2]
    a['cells'][2::3] = range(first, first + len(a['cells']) // 3)
    changed['ready']['prepared_state'][3] = first * 4096
    try:
        check(changed)
    except ValueError as error:
        outcomes.append(dict(name='prepared-aliases-parent', rejected=True, reason=str(error)))
    else:
        raise AssertionError('accepted parent/prepared physical alias')
    return outcomes


def readback(observations):
    require(observations['passed'], 'whole VM did not complete')
    native = observations['process_preparation_observation']
    require(native['passed'] and native['records'], 'missing preparation cohort')
    model = (ROOT / SELECTED[-2]).read_bytes()
    require(native['model_sha256'] == digest(model)['sha256'], 'model snapshot differs')
    results = [check(row) for row in native['records']]
    for actual, row in zip(results, native['records']):
        require(actual == row['comparison'], 'recorded comparison differs')
    large = next(row for row in native['records'] if row['comparison']['pages'] > 34)
    nonzero_large_copies = sum(any(bytes.fromhex(row['before']['active']['bytes'])[34*4096:])
                              for row in native['records'] if row['comparison']['pages'] > 34)
    require(nonzero_large_copies > 0, 'native copy needs nonzero bytes beyond predecessor heap')
    require(any(row['pages'] == 0 for row in results), 'missing empty heap')
    require(len(observations['executions']) == 4, 'ordinary metadata iterations')
    return dict(passed=True, preparations=len(results),
                pages=dict(sorted(Counter(row['pages'] for row in results).items())),
                copied_bytes=sum(row['copied_bytes'] for row in results),
                erased_before_release_bytes=sum(row['released_bytes'] for row in results),
                nonzero_large_heap_copies=nonzero_large_copies,
                metadata_syscalls=sum(len(row['calls']) for row in observations['executions']),
                controls=controls(large), comparisons=results)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build', type=Path)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    build, reference, output = args.build.resolve(), args.reference.resolve(), args.output.resolve()
    require(output.is_relative_to(ROOT / 'linux-kernel'), 'Linux output directory')
    output.mkdir(parents=True, exist_ok=False)
    raw = (build / 'source-user-extents/observations.json').read_bytes()
    report = readback(json.loads(raw))
    domain = '  %mm = and i1 %owned,%either\n'
    target = (reference / 'target.ll').read_text()
    require(target.count(domain) == 1, 'reference token domain')
    target = target.replace(domain, '  %prepared = icmp eq ptr %token,@topal_x86_process_mm_prepared_token_v1\n  %mmEither = or i1 %either,%prepared\n  %mm = and i1 %owned,%mmEither\n')
    target += '\n@topal_x86_process_mm_prepared_token_v1 = external global [3 x i64]\n'
    require(target == (build / 'target.ll').read_text(), 'unexpected target change')
    dispatch = (reference / 'dispatch.ll').read_text()
    site = '  %brk = icmp eq i64 %operation,5\n'
    require(dispatch.count(site) == 1, 'reference preparation site')
    hook = '  %pmBrk = icmp eq i64 %operation,5\n  br i1 %pmBrk,label %pmPreparation,label %pmContinue\npmPreparation:\n  %preparedHeap = call i1 @pm.prepare()\n  call void @pm.cancel(i1 %preparedHeap)\n  br label %pmContinue\npmContinue:\n'
    dispatch = dispatch.replace(site, hook + site)
    dispatch += (ROOT / SELECTED[1]).read_text()
    require(dispatch == (build / 'dispatch.ll').read_text(), 'unexpected native dispatch change')
    frame = json.loads((build / 'frame-inspection.json').read_text())
    require(frame['user_extent_callers']['complete_stack_bytes'] <= 4096, 'combined stack')
    require(frame['user_extent_callers']['complete_physical_bytes'] <= 4096, 'retirement window')
    require({'pm.prepare', 'pm.cancel', 'pm.at_erased'} <= set(frame['calls']), 'missing native closure')
    preserved = {}
    for name in PRESERVED:
        actual = (build / name).read_bytes()
        require(actual == (reference / name).read_bytes(), 'changed predecessor contribution: ' + name)
        preserved[name] = digest(actual)
    source_delta = source_elf_delta((build / 'source-user.elf').read_bytes(), (reference / 'source-user.elf').read_bytes())
    console = (build / 'source-user-extents/serial.log').read_bytes()
    require(console == (reference / 'source-user-extents/serial.log').read_bytes(), 'ordinary console changed')
    report.update(schema='topal-process-mm-preparation-native-v1',
                  scope='Experimental native preparation/cancellation only; active/inactive MM switching, fatal/refusal paths, general lifetime and production activation remain unqualified',
                  observation=digest(raw), console=digest(console), preserved=preserved,
                  source_elf_delta=source_delta,
                  native_bounds=frame['user_extent_callers'],
                  exact_target_and_dispatch_delta=True,
                  selected_inputs={name: digest((ROOT / name).read_bytes()) for name in SELECTED},
                  artifacts={p.name: digest(p.read_bytes()) for p in sorted(build.iterdir())
                             if p.suffix in ['.o', '.ll', '.elf'] or p.name in ['bzImage', 'initramfs.cpio']})
    metadata_raw = (build / 'metadata-readback/readback.json').read_bytes()
    metadata = json.loads(metadata_raw)
    require(metadata['passed'] and metadata['observation_sha256'] == digest(raw)['sha256'], 'metadata readback binding')
    require(metadata['kernel_sha256'] == report['artifacts']['kernel.elf']['sha256'], 'metadata kernel binding')
    report['metadata_readback'] = {k: metadata[k] for k in ['syscalls', 'modeled_calls', 'stat_outputs', 'clock_returns']}
    verification_raw = (build / 'llvm-verification.json').read_bytes()
    verification = json.loads(verification_raw)
    require(set(verification) == {'source', 'runtime', 'owner', 'controller', 'target', 'dispatch', 'extent'}, 'LLVM verification closure')
    for role, result in verification.items():
        require(result['passed'] and result['sha256'] == report['artifacts'][role + '.ll']['sha256'], 'LLVM verification binding')
    report['llvm_verified_modules'] = sorted(verification)
    resource = build / 'resource-observation.json'
    require(resource.is_file(), 'missing cgroup sample')
    report['last_resource_sample'] = json.loads(resource.read_text())
    require(report['last_resource_sample']['swap_max_bytes'] == 0, 'swap enabled')
    require(all(v == 0 for v in report['last_resource_sample']['events'].values()), 'observed memory event')
    archive = {'native-observations.json': raw, 'frame-inspection.json': (build / 'frame-inspection.json').read_bytes(),
               'source-component.json': (build / 'source-component.json').read_bytes(),
               'resource-observation.json': resource.read_bytes(),
               'metadata-readback.json': metadata_raw,
               'llvm-verification.json': verification_raw,
               'generated-target.ll': (build / 'target.ll').read_bytes(),
               'generated-dispatch.ll': (build / 'dispatch.ll').read_bytes(),
               'reference-target.ll': (reference / 'target.ll').read_bytes(),
               'reference-dispatch.ll': (reference / 'dispatch.ll').read_bytes()}
    for name in SELECTED:
        archive['inputs/' + name] = (ROOT / name).read_bytes()
    for p in sorted(build.glob('*.command.json')):
        archive['commands/' + p.name] = p.read_bytes()
    for p in sorted(build.glob('*.c')):
        archive['source-fixture/' + p.name] = p.read_bytes()
    for p in sorted((build / 'readback-history').iterdir()):
        if p.is_file() and p.suffix in ['.py', '.json']:
            archive['history/' + p.name] = p.read_bytes()
    for p in sorted((build / 'source-user-extents').iterdir()):
        if p.name in ['inspect.gdb', 'gdb.stdout.log', 'gdb.stderr.log', 'qemu.command.json',
                      'qemu.stderr.log', 'qemu.stdout.log', 'serial.log']:
            archive['native/' + p.name] = p.read_bytes()
    for name, payload in archive.items():
        destination = output / (name + '.gz')
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(gzip.compress(payload, mtime=0))
    write_json(output / 'report.json', report)
    manifest = {p.relative_to(output).as_posix(): digest(p.read_bytes())
                for p in sorted(output.rglob('*')) if p.is_file()}
    write_json(output / 'manifest.json', manifest)
    print(json.dumps({key: report[key] for key in ['passed', 'preparations', 'pages', 'copied_bytes',
                                                 'erased_before_release_bytes', 'metadata_syscalls', 'native_bounds']}))


if __name__ == '__main__':
    main()
