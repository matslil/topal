#!/usr/bin/env python3
"""Freeze the process publication matrix and identified application histories.

TOPAL-SYSTEMS-PROCESS-MM-001 / TOPAL-SYSTEMS-USER-EXTENT-001. This archive
preserves native evidence; its saved-record checks do not execute another VM.
"""
import argparse
import gzip
import json
from pathlib import Path
import re

from archive_applications import process_reader, selected_inputs as application_inputs
from archive_process import PRESERVED, ROOT
from process_native import require
from readback_applications import facts as application_facts, readback as application_readback
from readback_interfaces import MODES, readback as interface_readback
from readback_process import digest
from readback_publication import facts, readback
from replay_process import checked
from user_extent_application_inspect import members


SCHEMA = 'topal-process-mm-native-publication-v1'
COHORTS = [
    ('owners', 'process_mm_fork_handoff_and_exit_preserve_independent_heap_owners', 1),
    ('child-exec', 'process_mm_child_exec_preserves_inactive_parent_and_fresh_child_heap', 1),
    ('allocation', 'process_mm_allocation_refusal_preserves_owner_and_pid_history', 1),
    ('geometry', 'process_mm_maximum_fork_and_clone_copy_survive_empty_and_one_page_handoff', 2),
    ('reader-cancel', 'process_mm_parent_exit_cancels_blocked_child_and_all_resources', 4),
    ('writer-cancel', 'process_mm_parent_exit_cancels_blocked_writer_and_dirty_fifo', 4),
    ('relationships', 'process_mm_parent_exit_retires_ready_and_zombie_relationships', 8),
]
TOTALS = ['publications', 'pages_invalidated', 'equal_size_process_publications',
          'retirements', 'erased_bytes', 'root_switches', 'cancelled_CPU_blocks']
REMAINING = [
    'all remaining caller/private-loan/competing-writer and mapping-producer paths',
    'continuous lifetime/exclusion and AP/DMA/asynchronous invalidation qualification',
    'identity-width/resource/fatal partial-effect qualification',
    'unexplained earlier metadata observer assertion and broader observer reliability',
    'cold production reconstruction and public activation',
    'complete Linux UAPI, distribution, Docker/Podman and VM-host objectives',
]


def canonical(value):
    """Use the saved JSON representation, including numeric operation keys."""
    return json.loads(json.dumps(value))


def selected_inputs(watched):
    paths = set(application_inputs()) | set(watched)
    paths.update(p.relative_to(ROOT).as_posix()
                 for p in (ROOT/'linux-kernel/tools/process-memory').glob('*.py'))
    return sorted(paths)


def closed_archive(folder):
    """Validate every byte and reject unmanifested files and escaping paths."""
    raw = (folder/'manifest.json').read_bytes()
    manifest = json.loads(raw)
    require({p.relative_to(folder).as_posix() for p in folder.rglob('*') if p.is_file()}
            == set(manifest) | {'manifest.json'}, 'archive file closure')
    for name, fact in manifest.items():
        path = (folder/name).resolve()
        require(path.is_relative_to(folder), 'archive path escapes root')
        checked(path.read_bytes(), fact)
    return raw


def retained(read, previous, kernel, binding, save=None):
    """Replay identified earlier executions with their original boot facts."""
    require(previous['passed'] and previous['schema'] == 'topal-process-mm-native-application-corpus-v1'
            and previous['artifacts']['kernel.elf'] == kernel, 'preceding application/kernel identity')
    require(binding == previous['facts'], 'retained application facts differ')
    result = dict(application_facts=binding, applications={}, interfaces={}, processes={})

    def payload(name, fact):
        raw = checked(read(name), fact)
        if save is not None:
            save('retained/'+name, raw)
        return raw

    require(len(previous['cases']) == 51, 'retained 51-workload closure')
    for name, row in previous['cases'].items():
        raw = payload('cases/'+name+'/observations.json', row['observation'])
        mm = payload('cases/'+name+'/process-mm-observations.json', row['mm_observation'])
        serial = payload('cases/'+name+'/serial.log', row['serial'])
        actual = canonical(application_readback(json.loads(raw), json.loads(mm), serial, name, binding))
        require(all(actual[key] == row[key] for key in actual), 'retained application summary differs')
        result['applications'][name] = actual
    require(set(previous['retained_interfaces']) == set(MODES), 'retained interface closure')
    for name, row in previous['retained_interfaces'].items():
        raw = payload('retained/interfaces/'+name+'/observations.json', row['observation'])
        mm = payload('retained/interfaces/'+name+'/mm-observations.json', row['mm_observation'])
        actual = canonical(interface_readback(json.loads(raw), json.loads(mm), kernel['sha256']))
        require(actual == row['summary'], 'retained interface summary differs')
        result['interfaces'][name] = actual
    names = {'maximum-56', 'maximum-57', 'allocation-refusal', 'child-exec'} | {
        f'{wait}-cancellation-{stage}-{kind}' for wait in ['reader', 'writer']
        for stage in ['early', 'late'] for kind in ['exit', 'exit-group']} | {
        f'relationship-{z}-{e}-{s}' for z in [0, 1] for e in [0, 1] for s in [60, 231]}
    require(set(previous['retained_process']) == names, 'retained twenty-process closure')
    for name, row in previous['retained_process'].items():
        raw = payload('retained/process/'+name+'/observations.json', row['observation'])
        actual = canonical(process_reader(name)(json.loads(raw)))
        require(actual == row['summary'], 'retained process summary differs')
        result['processes'][name] = actual
    return result


def native_build(build, previous):
    """Check the complete source-bound publication reader and native artifacts."""
    require(build.is_relative_to(ROOT/'linux-kernel'), 'Linux native build path')
    elf, binding = facts(build)
    kernel = digest(elf.data)
    require(kernel == previous['artifacts']['kernel.elf'], 'changed publication kernel')
    work = build/'process-mm-users'
    result = canonical(readback(build))
    require(result == json.loads((work/'publication-readback.json').read_text()),
            'matrix publication readback differs')
    require(json.loads((build/'guest.command.json').read_text())['exit_code'] == 0,
            'native publication guest failure')
    require((work/'gdb.stdout.log').read_bytes().count(b'FRAME_ROOT_GDB_PASS') == 1,
            'native debugger completion')
    qemu = json.loads((work/'qemu.command.json').read_text())
    expected_ram = '52M' if binding['component']['mode'] == 'process-mm-allocation' else '640M'
    require(qemu[qemu.index('-m')+1] == expected_ram
            and qemu[qemu.index('-smp')+1].startswith('cpus=2,')
            and qemu[qemu.index('-accel')+1].startswith('tcg,'), 'publication VM configuration')
    artifacts = {p.name: digest(p.read_bytes()) for p in sorted(build.iterdir())
                 if p.is_file() and (p.suffix in ['.o', '.ll', '.elf'] or p.name in ['bzImage', 'initramfs.cpio'])}
    require(artifacts['static-elf-abi-fixture.elf']['sha256'] == binding['component']['image_sha256']
            and artifacts['source-user.elf']['sha256'] == binding['component']['linked_sha256'],
            'publication application image binding')
    boot = members((build/'initramfs.cpio').read_bytes())
    require(set(boot) == {'init', 'static-user.elf'}
            and boot['static-user.elf'] == (build/'static-elf-abi-fixture.elf').read_bytes(),
            'publication application boot-resource binding')
    require(binding['component']['conservative_user_stack_bytes'] <= 2048,
            'publication application stack bound')
    fields = {'source': 'source', 'startup': 'startup', 'file': 'file', 'shell': 'shell', 'command': 'command',
              'exec': 'exec', 'signal': 'signal', 'poll': 'poll', 'pipe': 'pipe', 'storage': 'storage',
              'metadata': 'metadata', 'serial': 'serial', 'stat': 'stat', 'process_mm': 'process-mm',
              **{'process_mm_'+n: 'process-mm-'+n for n in ['exec', 'cancel', 'allocation', 'geometry', 'relationship']}}
    sources = {}
    for field, stem in fields.items():
        sha = binding['component'].get('source_sha256' if field == 'source' else field+'_source_sha256')
        if sha is None:
            continue
        source = ROOT/('linux-kernel/labs/qemu/x86_64/user_extent_'+field+'_cases.c')
        raw = source.read_bytes()
        require(digest(raw)['sha256'] == sha and (build/(stem+'-cases.c')).read_bytes() == raw,
                'publication source fixture binding: '+field)
        sources[source.relative_to(ROOT).as_posix()] = digest(raw)
    frame = json.loads((build/'frame-inspection.json').read_text())
    require(all(frame['user_extent_callers'][k] <= 4096
                for k in ['complete_stack_bytes', 'complete_physical_bytes'])
            and {'p.prepare', 'p.commit', 'p.switch', 'p.child_exit', 'p.cancel_inactive'} <= set(frame['calls']),
            'publication native caller bounds')
    preserved = {name: artifacts[name] for name in PRESERVED + ['retained-owned-provider.o', 'retained-owned-gateway.o']}
    require(preserved == previous['preserved'], 'changed predecessor contributions')
    verification = json.loads((build/'llvm-verification.json').read_text())
    require(set(verification) == {'source', 'runtime', 'owner', 'controller', 'target', 'dispatch', 'extent'}
            and all(row['passed'] and row['sha256'] == artifacts[name+'.ll']['sha256']
                    for name, row in verification.items()), 'publication seven-module LLVM binding')
    return dict(summary=result, facts=binding, artifacts=artifacts, preserved=preserved,
                native_bounds=frame['user_extent_callers'], source_fixtures=sources,
                llvm_verification=verification)


def history_record(history, number, save):
    """Retain failed/pilot evidence without applying today's observer to it."""
    run = json.loads((history/'run.json').read_text())
    require(not run['source_changes_during_run']
            and run['source_hashes_before'] == run['source_hashes_after'], 'historical source instability')
    inputs = history/'history-inputs'
    recorded = json.loads((inputs/'manifest.json').read_text())
    hashes = recorded.get('source_hashes', recorded)
    reconstructed = recorded.get('reconstructed', [])
    for name, sha in hashes.items():
        source = (inputs/name).resolve()
        require(source.is_relative_to(inputs), 'historical source path')
        require(digest(source.read_bytes())['sha256'] == sha, 'historical source identity')
        if name in run['source_hashes_before']:
            require(sha == run['source_hashes_before'][name], 'historical native input identity')
    row = dict(path=history.relative_to(ROOT).as_posix(), exit_code=run['exit_code'],
               elapsed_seconds=run['elapsed_seconds'], resources=run['resources'],
               retained_input_count=len(hashes), reconstructed=reconstructed,
               inputs_outside_native_watch=sorted(set(hashes)-set(run['source_hashes_before'])), builds=[],
               scope='Historical failure/method pilot with retained source identities; not current observer qualification, a new execution or a full historical rebuild. Selected reconstructed source bytes are labeled.')
    for p in sorted(history.rglob('*')):
        if p.is_file():
            save(f'history/{number}/run/'+p.relative_to(history).as_posix(), p.read_bytes())
    if 'cohorts' in run:
        paths = [str(ROOT/name) for cohort in run['cohorts'] for name in cohort['builds']]
    else:
        paths = re.findall(r'actual frame source root: ([^\n]+)', (history/'test.log').read_text())
        require(len(paths) == 1, 'historical native build identity')
    require(paths and len(set(paths)) == len(paths), 'distinct historical native builds')
    for index, name in enumerate(paths):
        build = Path(name.strip()).resolve()
        require(build.is_relative_to(ROOT/'linux-kernel/.cache'), 'historical build path')
        row['builds'].append(dict(path=build.relative_to(ROOT).as_posix(), kernel=digest((build/'kernel.elf').read_bytes())))
        for p in sorted(build.iterdir()):
            if p.is_file() and (p.suffix in ['.ll', '.o', '.elf', '.c', '.py', '.t', '.json', '.log', '.su', '.ld']
                                or p.name in ['bzImage', 'initramfs.cpio']):
                save(f'history/{number}/builds/{index}/'+p.name, p.read_bytes())
        for p in sorted((build/'process-mm-users').iterdir()):
            if p.is_file():
                save(f'history/{number}/builds/{index}/process-mm-users/'+p.name, p.read_bytes())
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['matrix', 'driver', 'prior', 'validation', 'output']:
        parser.add_argument('--'+name, required=True, type=Path)
    parser.add_argument('--history', action='append', default=[], type=Path)
    args = parser.parse_args()
    matrix, driver, prior, validation, output = [getattr(args, k).resolve()
                                               for k in ['matrix', 'driver', 'prior', 'validation', 'output']]
    require(all(p.is_relative_to(ROOT/'linux-kernel') for p in
                [matrix, driver, prior, validation, output]+[p.resolve() for p in args.history]), 'Linux archive paths')
    prior_manifest = closed_archive(prior)
    previous_raw = (prior/'report.json').read_bytes()
    previous = json.loads(previous_raw)
    execution = json.loads((matrix/'run.json').read_text())
    require(execution['passed'] and execution['exit_code'] == 0
            and not execution['source_changes_during_run']
            and execution['source_hashes_before'] == execution['source_hashes_after'], 'native matrix source/result closure')
    require(execution['environment'] == {'TOPAL_TEST_PROCESS_MM_PUBLICATION': '1'}, 'native publication selection')
    require(len(execution['cohorts']) == len(COHORTS), 'seven native tests')
    for name, sha in execution['source_hashes_before'].items():
        target = (ROOT/name).resolve()
        require(target.is_relative_to(ROOT/'linux-kernel') or target.is_relative_to(ROOT/'src'), 'native input path')
        require(digest(target.read_bytes())['sha256'] == sha, 'native input changed: '+name)
    resources = execution['resources']
    require(int(resources['memory.swap.max']) == 0 and 0 < int(resources['memory.peak']) <= int(resources['memory.max'])
            and all(int(line.split()[1]) == 0 for line in resources['memory.events'].splitlines()), 'native resource failure')
    output.mkdir(parents=True, exist_ok=False)

    def save(name, raw):
        target = (output/(name+'.gz')).resolve()
        require(target.is_relative_to(output), 'archive destination path')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(gzip.compress(raw, mtime=0))

    report = dict(passed=True, schema=SCHEMA, cohorts=[], execution=execution,
                  prior_archive=dict(path=prior.relative_to(ROOT).as_posix(), manifest=digest(prior_manifest),
                                     report=digest(previous_raw)), remaining=REMAINING)
    save('prior/manifest.json', prior_manifest)
    save('prior/report.json', previous_raw)
    save('execution/driver.py', driver.read_bytes())
    report['driver'] = digest(driver.read_bytes())
    seen = set()
    for row, (label, test, count) in zip(execution['cohorts'], COHORTS):
        require(row['label'] == label and row['exit_code'] == 0 and len(row['builds']) == count
                and row['argv'] == ['rtk', 'proxy', 'cargo', 'test', '-p', 'topal-compiler', '--lib', '--locked', '-j1',
                    'systems_frame_target::root_tests::'+test, '--', '--ignored', '--exact', '--test-threads=1', '--nocapture'],
                'native test selection/count')
        for index, name in enumerate(row['builds']):
            build = (ROOT/name).resolve()
            require(build not in seen, 'reused native build')
            seen.add(build)
            command = json.loads((matrix/label/f'readback-{index}.command.json').read_text())
            require(command == dict(argv=['rtk', 'proxy', 'python3', 'linux-kernel/tools/process-memory/readback_publication.py', str(build)],
                                    exit_code=0), 'matrix acceptance reader command')
            result = native_build(build, previous)
            number = len(report['cohorts'])
            result.update(label=label, native_build=name)
            report['cohorts'].append(result)
            for p in sorted(build.iterdir()):
                if p.is_file() and (p.suffix in ['.ll', '.o', '.elf', '.c', '.py', '.t', '.json', '.log', '.su', '.ld']
                                    or p.name in ['bzImage', 'initramfs.cpio']):
                    save(f'cohorts/{number}/build/'+p.name, p.read_bytes())
            for p in sorted((build/'process-mm-users').iterdir()):
                if p.is_file():
                    save(f'cohorts/{number}/build/process-mm-users/'+p.name, p.read_bytes())
    require(len(seen) == 21, 'twenty-one distinct VM builds')
    report['native_summary'] = {key: sum(row['summary'][key] for row in report['cohorts']) for key in TOTALS}
    report['native_summary'].update(VMs=21, tests=7, controls=sum(len(row['summary']['controls']) for row in report['cohorts']))
    for p in sorted(matrix.rglob('*')):
        if p.is_file():
            save('execution/'+p.relative_to(matrix).as_posix(), p.read_bytes())
    validation_result = json.loads((validation/'validation.json').read_text())
    require(set(validation_result) == {'workspace-check', 'user-memory-models', 'process-memory-models',
                                      'default-ABI-fixture', 'format', 'Rust-file-ceiling'}
            and all(row['exit_code'] == 0 for row in validation_result.values()), 'fresh validation closure')
    require('Ran 203 tests' in (validation/'user-memory-models.stderr.log').read_text()
            and 'Ran 59 tests' in (validation/'process-memory-models.stderr.log').read_text()
            and 'test result: ok. 1 passed' in (validation/'default-ABI-fixture.stdout.log').read_text(), 'fresh model/default test coverage')
    report['validation'] = validation_result
    stable = json.loads((validation/'source-stability.json').read_text())
    require(stable['passed'] and stable['source_hashes_before'] == stable['source_hashes_after'],
            'validation source stability')
    require(set(stable['source_hashes_before']) == set(selected_inputs(execution['source_hashes_before'])),
            'validation selected-input closure')
    require(all(digest((ROOT/name).read_bytes())['sha256'] == sha
                for name, sha in stable['source_hashes_before'].items()), 'input changed after validation')
    report['validation_stability'] = stable
    for p in sorted(validation.iterdir()):
        if p.is_file():
            save('validation/'+p.name, p.read_bytes())
    def old_read(name):
        return gzip.decompress((prior/(name+'.gz')).read_bytes())
    # Keep original boot-resource length and inventory; new C fixtures differ.
    old_build = validation/'retained-application-build'
    old_build.mkdir(exist_ok=False)
    for name in ['kernel.elf', 'initramfs.cpio', 'extent-call-inventory.json']:
        raw = old_read('build/'+name)
        if name in previous['artifacts']:
            checked(raw, previous['artifacts'][name])
        (old_build/name).write_bytes(raw)
        save('retained/build/'+name, raw)
    binding = application_facts(old_build)
    report['retained'] = retained(old_read, previous, previous['artifacts']['kernel.elf'], binding, save)
    report['retained_scope'] = 'Identified previous 51 application, nine interface and twenty process native histories on the byte-identical kernel; current saved-record replay, no new execution or full historical rebuild'
    require(len(args.history) == 4, 'three historical pilots and first matrix attempt')
    report['history'] = [history_record(path.resolve(), i, save) for i, path in enumerate(args.history)]
    require([row['exit_code'] for row in report['history']] == [101, 0, 0, 1], 'historical failure/pilot sequence')
    inputs = selected_inputs(execution['source_hashes_before'])
    report['selected_inputs'] = {name: digest((ROOT/name).read_bytes()) for name in inputs}
    for name in inputs:
        save('inputs/'+name, (ROOT/name).read_bytes())
    (output/'report.json').write_text(json.dumps(report, indent=2, sort_keys=True)+'\n')
    manifest = {p.relative_to(output).as_posix(): digest(p.read_bytes()) for p in sorted(output.rglob('*')) if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True)+'\n')
    print(json.dumps(dict(passed=True, entries=len(manifest), inputs=len(inputs), summary=report['native_summary'])))


if __name__ == '__main__':
    main()
