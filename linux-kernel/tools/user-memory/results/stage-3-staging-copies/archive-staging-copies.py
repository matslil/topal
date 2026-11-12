"""Freeze selected sources, new kernel artifacts and separate native cohorts."""
import gzip
import hashlib
import json
from pathlib import Path
import posixpath
import re
import subprocess
import shutil
import sys

catalog=json.loads(Path(sys.argv[1]).read_text())
root=Path('linux-kernel/tools/user-memory/results/stage-3-staging-copies')
prior=root.parent/'stage-3-additional-scalars'
# Publication is gated on terminal results, never an in-progress catalog entry.
assert set(catalog['native']) == {'source','storage','serial','pinned','pipes','poll'}
assert set(catalog['native_additional']) == {'exec','signals'}, 'remaining semantic VM cohorts are not accepted'
library_directory = Path(catalog['library'])
library_record = json.loads((library_directory/'run.json').read_text())
assert library_record['exit_code'] == 0 and not library_record['source_changes_during_run']
assert library_record['modified_source_hashes_before'] == library_record['modified_source_hashes_after']
assert re.search(r'test result: ok\. 383 passed; 0 failed; 8 ignored;',
                 (library_directory/'test.log').read_text())
freeze = library_record['modified_source_hashes_before']
assert freeze == catalog['current_modified_source_freeze'] and len(freeze) == 22
post_directory = Path(catalog['post'])
post_record = json.loads((post_directory/'validation.json').read_text())
assert post_record['passed'] and not post_record['source_changes_during_run']
assert post_record['modified_source_hashes_before'] == post_record['modified_source_hashes_after'] == freeze
assert len(post_record['results']) == 8 and all(r['exit_code'] == 0 for r in post_record['results'])
assert 'Ran 52 tests in ' in (post_directory/'0.log').read_text()
assert 'Ran 8 tests in ' in (post_directory/'1.log').read_text()
for key, filename in [('gnu','readback.json'),('graph','gnu-verification.json'),
                      ('source_readback','independent-readback.json'),('controls','controls.json')]:
    assert json.loads((Path(catalog[key])/filename).read_text())['passed']
assert json.loads(Path(catalog['difference']).read_text())['passed']
budget = json.loads((Path(catalog['full_workload_budget_preflight'])/'preflight.json').read_text())
assert budget['applied'] and budget['passed'] and budget['unit_tests'] == 4
assert all(freeze[path] == sha for path,sha in budget['source_sha256'].items())
assert not root.exists(), 'do not mix a new archive with a partial or old one'
root.mkdir(parents=True)
digest=lambda data:hashlib.sha256(data).hexdigest()
def write(name,value):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_text(json.dumps(value,indent=2)+'\n')
def copy(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(path,target)
def compress(path,name):
    target=root/name
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(gzip.compress(Path(path).read_bytes(),mtime=0))
def bundle(directory,name):
    rows=[]
    for path in sorted(directory.rglob('*')):
        if path.is_file():
            data=path.read_bytes()
            rows.append(dict(name=str(path.relative_to(directory)),bytes=len(data),sha256=digest(data),text=data.decode()))
    (root/name).write_bytes(gzip.compress((json.dumps(rows,indent=2)+'\n').encode(),mtime=0))

build=Path(catalog['native']['pinned']['build'])
native={name:json.loads((Path(row['run'])/'run.json').read_text()) for name,row in catalog['native'].items()}
assert set(native)=={'source','storage','serial','pinned','pipes','poll'}
for name,record in native.items():
    assert record['exit_code']==0 and not record['source_changes_during_run'],name
# These six runs preceded the host-only workload budget change. Preserve their
# actual 19 recorded inputs, and the two saved baseline harness inputs separately.
for name,record in native.items():
    before = record['modified_source_hashes_before']
    assert before == record['modified_source_hashes_after'] and len(before) == 19
    assert all(freeze[path] == sha for path,sha in before.items())
    overrides = {}
    for path in ['linux-kernel/labs/qemu/x86_64/frame_root_inspect.py',
                 'linux-kernel/labs/qemu/x86_64/user_extent_callers_inspect.py']:
        saved = Path('linux-kernel/.cache/x86_64/before-full-workload-budget-inputs')/path
        data = saved.read_bytes()
        assert data == subprocess.check_output(['git','show','609750865fa2aaf894125ec4da924d0a0a2cfd02:'+path])
        snapshot = 'earlier-inputs/before-full-workload-budget/'+path+'.gz'
        compress(saved,snapshot)
        overrides[path] = dict(sha256=digest(data),snapshot='../../'+snapshot)
    write('native/'+name+'/source-overrides.json',dict(inputs={},
        baseline_context=overrides,
        recorded_modified_inputs=len(before),
        scope='Actual recorded 19-input freeze. Two additional baseline harness files were saved before the later full-workload budget change and equal the base commit; they were not part of the per-run modified-input hash capture. New host-only test_frame_progress.py did not yet exist. Not a full checkout reconstruction.'))
previous=json.loads((prior/'source-identity.json').read_text())['selected_inputs']
selected={}
for path in sorted(set(previous)|set(freeze)):
    data=Path(path).read_bytes()
    if path in freeze:
        assert digest(data)==freeze[path]
        snapshot='inputs/'+path+'.gz'
        compress(path,snapshot)
    else:
        row=previous[path]
        snapshot=posixpath.normpath('../stage-3-additional-scalars/'+row['snapshot'])
        assert digest(data)==row['sha256'] and gzip.decompress((root/snapshot).read_bytes())==data
    selected[path]=dict(sha256=digest(data),snapshot=snapshot)
assert len(previous) == 125 and len(selected) == 135
write('source-identity.json',dict(base_commit='609750865fa2aaf894125ec4da924d0a0a2cfd02',
    inherited_record='../stage-3-additional-scalars/source-identity.json',
    inherited_record_sha256=digest((prior/'source-identity.json').read_bytes()),selected_inputs=selected,
    scope='Selected authority/producer/reader/observer inputs, not a full producer reconstruction'))
artifact_names=['root.o','source.o','runtime.o','owner.o','controller.o','target.o','dispatch.o',
    'gateway.o','boundary.o','extent.o','extent-bridge.o','extent-callers.o','provider.o',
    'predecessor-provider.o','predecessor-gateway.o','kernel.elf','bzImage',
    'process-access-inventory.json','process-byte-origins.json','process-access-origins.json',
    'extent-call-inventory.json','fixed-transfer-audit.json','copy-transfer-audit.json',
    'staged-transfer-audit.json','additional-scalar-audit.json','staging-copy-audit.json',
    'process-access-disassembly.stdout.log']
artifacts={}
for name in artifact_names:
    data=(build/name).read_bytes()
    # The disassembly transcript contains its build directory in the header.
    # Keep the pinned transcript for complete-body difference readback; compare
    # every executable, object and structured report across all six cohorts.
    if name!='process-access-disassembly.stdout.log':
        assert all((Path(r['build'])/name).read_bytes()==data for r in catalog['native'].values()),name
    snapshot='artifacts/'+name+'.gz'
    compress(build/name,snapshot)
    artifacts[name]=dict(bytes=len(data),sha256=digest(data),snapshot=snapshot,
        snapshot_sha256=digest((root/snapshot).read_bytes()))
write('artifact-identity.json',artifacts)
# Preserve the two historical first-party artifact sets needed to replay both
# transformations. Reuse current snapshots only after actual byte equality.
baseline = Path('linux-kernel/.cache/x86_64/user-extent-application-2304790-1794418256907813314')
intermediate = Path(catalog['native_before_private_clean']['pinned']['build'])
assert digest((baseline/'kernel.elf').read_bytes()) == '04f4f0b01552e5d724187ae841bec2844d00d113eee2ee940adb8304997433c6'
assert digest((intermediate/'kernel.elf').read_bytes()) == '77ba1dbf002743b6de87320f695c44f7324da367be67bf45f40e443deeffd54c'
baseline_identity = json.loads((prior/'artifact-identity.json').read_text())
for name in ['kernel.elf','provider.o','process-access-inventory.json','extent-call-inventory.json']:
    row = baseline_identity[name]
    frozen = (prior/row['snapshot']).read_bytes()
    if row.get('compressed',True):
        assert digest(frozen) == row['snapshot_sha256']
        frozen = gzip.decompress(frozen)
    assert digest(frozen) == row['sha256'] and frozen == (baseline/name).read_bytes(), name
history_names = [name for name in artifact_names if name.endswith('.o')] + [
    'kernel.elf','process-access-inventory.json','extent-call-inventory.json',
    'process-access-disassembly.stdout.log']
for label,directory in [('baseline',baseline),('intermediate',intermediate)]:
    rows = {}
    for name in history_names:
        data = (directory/name).read_bytes()
        if data == (build/name).read_bytes():
            row = dict(artifacts[name])
        else:
            snapshot = 'history/'+label+'/'+name+'.gz'
            compress(directory/name,snapshot)
            row = dict(bytes=len(data),sha256=digest(data),snapshot=snapshot,
                       snapshot_sha256=digest((root/snapshot).read_bytes()))
        rows[name] = row
    write(label+'-artifact-identity.json',rows)
selected_calls={}
accesses=set()
for name,field in [('fixed-transfer-audit.json','access_instructions'),('copy-transfer-audit.json','copy_offset'),
    ('staged-transfer-audit.json','user_accesses'),('additional-scalar-audit.json','user_accesses'),
    ('staging-copy-audit.json','copy_offset')]:
    for row in json.loads((build/name).read_text())['cases']:
        at=row['call_offset']
        assert at not in selected_calls
        selected_calls[at]=dict(name=row['name'],call_offset=at,audit='artifacts/'+name+'.gz')
        accesses.update([row[field]] if field=='copy_offset' else [r['offset'] for r in row[field]])
census=json.loads((build/'extent-call-inventory.json').read_text())
assert len(selected_calls)==79 and len(accesses)==306
write('caller-access-index.json',dict(complete=False,kernel_sha256=json.loads((build/'staging-copy-audit.json').read_text())['kernel_sha256'],
    emitted_helper_calls=91,calls_with_selected_local_intervals=79,distinct_user_access_instructions=306,
    selected_intervals=[selected_calls[at] for at in sorted(selected_calls)],
    calls_without_selected_local_intervals=[r for r in census['calls'] if r['offset'] not in selected_calls],
    scope='Fresh reports for the changed kernel; conditional selected local intervals, not whole callers, complete owners/lifetime or production qualification'))
native_summary={}
reached=set()
for name,row in catalog['native'].items():
    current=Path(row['build'])
    guest=current/('first-image-exit' if name=='pinned' else 'source-user-extents')
    observation=json.loads((guest/'observations.json').read_text())
    assert observation['passed']
    coverage=observation['extent_call_coverage']
    assert coverage['kernel_sha256']==json.loads((build/'staging-copy-audit.json').read_text())['kernel_sha256']
    reached.update(r['offset'] for r in coverage['reached'])
    copy(Path(row['run'])/'run.json','native/'+name+'/run.json')
    copy(Path(row['run'])/'test.log','native/'+name+'/test.log')
    for filename in ['observations.json','inspect.gdb']: compress(guest/filename,'native/'+name+'/'+filename+'.gz')
    for filename in ['qemu.command.json','qemu.stdout.log','qemu.stderr.log','gdb.stdout.log','gdb.stderr.log','serial.log']:
        copy(guest/filename,'native/'+name+'/'+filename)
    identities={}
    for filename in ['static-elf-abi-fixture.elf','initramfs.cpio']+([] if name=='pinned' else ['source-component.json']):
        data=(current/filename).read_bytes()
        if name!='pinned':
            compress(current/filename,'native/'+name+'/'+filename+'.gz')
        identities[filename]=dict(bytes=len(data),sha256=digest(data))
    if name=='source':
        for path in [current/'source-user.elf',*current.glob('*.su')]:
            compress(path,'native/source/'+path.name+'.gz')
    calls=[c for key in ['executions','startup_executions','file_executions','shell_executions','command_executions','poll_executions']
        for execution in observation.get(key,[]) for c in execution['calls']]
    native_summary[name]=dict(identity=identities,reached_sites=coverage['reached_sites'],
        calls=len(calls),efaults=sum(c['result']==-14 for c in calls),
        copy_observation={k:v for k,v in observation.get('copy_transfer_observation',{}).items() if k not in ['samples','source_syscall_refusals']})
write('native-summary.json',dict(cohorts=native_summary,current_kernel_reached_sites=len(reached),
    reached=sorted(reached),unreached=[r for r in census['calls'] if r['offset'] not in reached],
    scope='Only actual matching changed-kernel observations; historical old-kernel 88/91 union is not reused'))
# The remaining two ignored VM cases provide semantic regressions without a
# helper-site census. Keep them separate from the six-cohort coverage union.
for name,row in catalog['native_additional'].items():
    current = Path(row['build'])
    directory = Path(row['run'])
    record = json.loads((directory/'run.json').read_text())
    assert record['exit_code'] == 0 and not record['source_changes_during_run']
    assert record['modified_source_hashes_before'] == record['modified_source_hashes_after'] == freeze
    assert '1 passed; 0 failed;' in (directory/'test.log').read_text()
    assert digest((current/'kernel.elf').read_bytes()) == digest((build/'kernel.elf').read_bytes())
    guest = current/'source-user-extents'
    observation = json.loads((guest/'observations.json').read_text())
    assert observation['passed'] and 'extent_call_coverage' not in observation
    assert json.loads((current/'source-component.json').read_text())['mode'] == name
    destination = 'native-additional/'+name+'/'
    for filename in ['run.json','test.log']: copy(directory/filename,destination+filename)
    for filename in ['observations.json','inspect.gdb']: compress(guest/filename,destination+filename+'.gz')
    for filename in ['qemu.command.json','qemu.stdout.log','qemu.stderr.log','gdb.stdout.log','gdb.stderr.log','serial.log']:
        copy(guest/filename,destination+filename)
    for filename in ['source-component.json','static-elf-abi-fixture.elf','initramfs.cpio']:
        compress(current/filename,destination+filename+'.gz')

private = catalog['private_clean']
component_run,component_build = Path(private['component_run']),Path(private['component_build'])
component_record = json.loads((component_run/'run.json').read_text())
assert component_record['exit_code'] == 0 and not component_record['source_changes_during_run']
assert component_record['modified_source_hashes_before'] == component_record['modified_source_hashes_after']
assert all(freeze[path] == sha for path,sha in component_record['modified_source_hashes_before'].items())
assert '3 passed; 0 failed;' in (component_run/'test.log').read_text()
assert (component_build/'probe.stdout.log').read_text().strip() == 'PRIVATE_CLEAN_COMPONENT_PASS probes=210947'
for filename in ['run.json','test.log']: copy(component_run/filename,'private-clean/'+filename)
for path in sorted(component_build.iterdir()):
    if not path.is_file():
        continue
    if path.name == 'cases':
        # The host executable links system startup/runtime material. The exact
        # first-party predicate object is preserved; retain this identity only.
        data = path.read_bytes()
        write('private-clean/component-executable-identity.json',dict(
            bytes=len(data),sha256=digest(data),archived=False,
            scope='Host test executable identity only; source, predicate object, commands and actual output are retained. System startup/runtime binary redistribution is not introduced.'))
    else:
        compress(path,'private-clean/component/'+path.name+'.gz')
for field in ['review','target_binding','shared_target_binding']:
    copy(Path(private[field]),'private-clean/'+Path(private[field]).name)
for filename in ['observer-word-decoding-review.json','bounded-count-root-guard-review.json',
                 'bounded-count-root-guard-candidate.py','full-shared-input-comparison.json']:
    copy(Path('linux-kernel/.cache/x86_64')/filename,'method/'+filename)
for path in Path(catalog['root_guard_preflight']).iterdir():
    if path.is_file(): copy(path,'method/root-guard/'+path.name)
for path in Path(catalog['full_workload_budget_preflight']).iterdir():
    if path.is_file(): copy(path,'method/full-workload-budget/'+path.name)
copy(Path(catalog['non_acceptance_zero_filter'])/'run.json','failures/zero-test-filter/run.json')
copy(Path(catalog['non_acceptance_zero_filter'])/'test.log','failures/zero-test-filter/test.log')
write('failures/zero-test-filter/scope.json',dict(native_acceptance=False,
    reason='Exact module-prefix filter selected zero tests; compilation only. The subsequent actual three-test component run is separate.'))

for key,filename in [('post','post-validation.json'),('library','library-validation.json')]:
    directory=Path(catalog[key])
    source='validation.json' if key=='post' else 'run.json'
    report=json.loads((directory/source).read_text())
    assert report.get('passed',report.get('exit_code')==0)
    copy(directory/source,filename)
    if key=='post':
        for log in directory.glob('*.log'): copy(log,'post/'+log.name)
    else: compress(directory/'test.log','library-test.log.gz')
for key in ['gnu','graph']:
    directory=Path(catalog[key])
    report=directory/('readback.json' if key=='gnu' else 'gnu-verification.json')
    assert json.loads(report.read_text())['passed']
    copy(report,key+'-readback.json')
    bundle(directory,key+'-records.json.gz')
copy(Path(catalog['difference']),'kernel-difference.json')
# Retain the successful full shared workload separately from the six census
# cohorts. Its application is the first-party generated caller fixture.
full_candidates = list(Path(library_record['temporary_directory']).glob('user-extent-callers-*'))
assert len(full_candidates) == 1
full_build = full_candidates[0]
assert full_build.resolve() == Path(catalog['current_full_shared_guest']['build']).resolve()
full_guest = full_build/'user-extent-callers'
full_observation = json.loads((full_guest/'observations.json').read_text())
assert full_observation['passed'] and full_observation['process_cleanup_checked']
assert digest((full_build/'kernel.elf').read_bytes()) == artifacts['kernel.elf']['sha256']
comparison = json.loads(Path('linux-kernel/.cache/x86_64/full-shared-input-comparison.json').read_text())
assert comparison['passed'] and comparison['debugger_program_equal_after_build_path_and_loopback_port_normalization']
assert set(comparison['artifacts']) == {'kernel.elf','static-elf-abi-fixture.elf','initramfs.cpio',
    'caller-observation.json','caller-fixture-sites.json','user-owners.json'}
for filename,row in comparison['artifacts'].items():
    assert row['equal'] and row['after_sha256'] == digest((full_build/filename).read_bytes())
    compress(full_build/filename,'full-shared/inputs/'+filename+'.gz')
for filename in ['observations.json','inspect.gdb']:
    compress(full_guest/filename,'full-shared/'+filename+'.gz')
for filename in ['qemu.command.json','qemu.stdout.log','qemu.stderr.log','gdb.stdout.log','gdb.stderr.log','serial.log',
                 'observations.json.extent-records.jsonl.gz']:
    copy(full_guest/filename,'full-shared/'+filename)
old_shared = Path(catalog['failures']['wide-shared-original-deadline']['run'])/'run.json'
old_shared_record = json.loads(old_shared.read_text())
assert old_shared_record['exit_code'] != 0
old_guest = Path('linux-kernel/.cache/x86_64/tk283-staging-focused-1794433773644775945/temporary/user-extent-callers-2341162-1794433774658651303/user-extent-callers')
compress(old_guest/'inspect.gdb','method/before-full-workload-budget-inspect.gdb.gz')
source_readback=Path(catalog['source_readback'])
assert json.loads((source_readback/'independent-readback.json').read_text())['passed']
copy(source_readback/'independent-readback.json','source-readback.json')
controls=Path(catalog['controls'])
assert json.loads((controls/'controls.json').read_text())['passed']
copy(controls/'controls.json','controls.json')
copy(controls/'positive.log','controls-positive.log')
for path in controls.glob('*/rejection.json'):
    copy(path,'controls/'+path.parent.name+'.json')
    copy(path.parent/'rejection.log','controls/'+path.parent.name+'.log')
for name,row in catalog.get('failures',{}).items():
    if 'run' not in row:
        write('failures/'+name+'/scope.json',row)
        directory = Path(row['directory'])
        for path in directory.iterdir():
            if path.is_file() and (path.suffix in ['.json','.log'] or path.name == 'inspect.gdb'):
                compress(path,'failures/'+name+'/'+path.name+'.gz')
        continue
    directory=Path(row['run'])
    failure=json.loads((directory/'run.json').read_text())
    assert failure['exit_code']!=0 or failure['source_changes_during_run']
    for filename in ['run.json','test.log']: copy(directory/filename,'failures/'+name+'/'+filename)
    if 'inputs' in row:
        for path,sha in failure['modified_source_hashes_before'].items():
            old=Path(row['inputs'])/path
            assert digest(old.read_bytes())==sha
            compress(old,'failures/'+name+'/inputs/'+path+'.gz')
    if name=='pinned-reader-drift':
        before,after=failure['modified_source_hashes_before'],failure['modified_source_hashes_after']
        changed={path for path in before if before[path]!=after[path]}
        assert failure['exit_code']==0 and changed=={'linux-kernel/tools/user-memory/audit_staging_copies.py'}
        write('failures/'+name+'/scope.json',dict(native_test_passed=True,source_freeze_acceptance=False,
            changed_sources=sorted(changed),
            scope='The test passed with the same executable kernel, but the validation runner rejected changing reader input. It is not final source-freeze acceptance.'))
for name in ['archive-staging-copies','readback-staging-copies','check-staging-copy-controls','check-staging-copies',
    'verify-staging-graph','staging-graph-program','diff-staging-kernel','run-staging-library-tests','replay-staging-copies',
    'audit-committed-staging-copies']:
    copy(Path('linux-kernel/.cache/x86_64')/(name+'.py'),name+'.py')
for name in ['compose-staging-kernel-difference','check-private-clean-target','run-staging-focused-tests',
             'apply-full-workload-budget','apply-staging-root-guard','collect-staging-wide-results',
             'readback-full-shared-records']:
    copy(Path('linux-kernel/.cache/x86_64')/(name+'.py'),name+'.py')
copy(Path('linux-kernel/.cache/x86_64/staging_source_versions.py'),'staging_source_versions.py')
copy(Path(catalog['before_private_clean_validation']['difference']),'prior-provider-kernel-difference.json')
copy(Path('linux-kernel/.cache/x86_64/readback-staging-source.py'),'readback-staging-source.py')
copy(Path('linux-kernel/.cache/x86_64/finalize-staging-copies.py'),'finalize-staging-copies.py')
copy(Path('linux-kernel/.cache/x86_64/staging-library-initial-failure.json'),'failures/library-initial-summary.json')
copy(Path('linux-kernel/.cache/x86_64/staging-copies-review.json'),'review.json')
copy(Path('linux-kernel/.cache/x86_64/staging-copies-review.before-wide.json'),'method/initial-change-review.json')
copy(Path('linux-kernel/.cache/tk283-source-retry.py'),'native-source-runner.py')
copy(Path('linux-kernel/.cache/tk283-application-retry.py'),'native-pinned-runner.py')
write('catalog.json',{k:v for k,v in catalog.items() if k not in ['pending_exec_sessions','notes']})
(root/'.gitattributes').write_text('*.log -text -whitespace\n')
print(json.dumps(dict(archive=str(root),selected_inputs=len(selected),local_intervals=len(selected_calls),distinct_accesses=len(accesses),native_reached_sites=len(reached))))
