"""Collect only terminal, matching-kernel results; preserve historical failures."""
import hashlib
import json
from pathlib import Path
import re

cache=Path('linux-kernel/.cache/x86_64')
catalog_path=cache/'staging-copies-catalog.json'
catalog=json.loads(catalog_path.read_text())
expected=catalog['private_clean']['kernel_sha256']
required=json.loads((Path(catalog['native']['poll']['run'])/'run.json').read_text())['modified_source_hashes_before']
budget_path=cache/'full-workload-budget-preflight/preflight.json'
budget=json.loads(budget_path.read_text()) if budget_path.exists() else dict(applied=False)
frame_tests=8 if budget['applied'] else 4
cohorts={
    'owned_source_fixture_checks_native_user_buffer_boundaries':'source',
    'owned_source_fixture_checks_storage_transfer_boundaries':'storage',
    'owned_source_fixture_checks_serial_transfer_boundaries':'serial',
    'owned_source_fixture_checks_pipe_transfer_boundaries':'pipes',
    'owned_source_fixture_checks_poll_transfer_boundaries':'poll',
    'owned_first_image_exit_retires_transferred_signal_state':'pinned',
}
additional_selectors={
    'owned_source_fixture_checks_exec_vector_boundaries':'exec',
    'owned_source_fixture_checks_signal_transfer_boundaries':'signals',
}
accepted={}
additional={}
for directory in sorted(cache.glob('tk283-staging-focused-*')):
    if not (directory/'run.json').exists():continue
    record=json.loads((directory/'run.json').read_text())
    matches=[name for selector,name in cohorts.items()
             if 'systems_frame_target::root_tests::'+selector in record['command']]
    extra=[name for selector,name in additional_selectors.items()
           if 'systems_frame_target::root_tests::'+selector in record['command']]
    is_additional=bool(extra)
    matches+=extra
    if not matches or record['exit_code']!=0 or record['source_changes_during_run']:continue
    assert len(matches)==1
    output=(directory/'test.log').read_text()
    assert '1 passed; 0 failed;' in output,'zero or multiple tests cannot qualify a cohort'
    roots=re.findall(r'actual frame source root: ([^\n]+)',output)
    assert len(roots)==1
    build=Path(roots[0]).resolve()
    if hashlib.sha256((build/'kernel.elf').read_bytes()).hexdigest()!=expected:continue
    before=record['modified_source_hashes_before']
    assert before==record['modified_source_hashes_after']
    assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in before.items())
    name=matches[0]
    guest=build/('first-image-exit' if name=='pinned' else 'source-user-extents')
    observation=json.loads((guest/'observations.json').read_text())
    assert observation['passed']
    row=dict(run=str(directory),build=str(build.relative_to(Path.cwd())))
    if is_additional:
        assert json.loads((build/'source-component.json').read_text())['mode']==name
        additional[name]=row
    else:
        assert observation['extent_call_coverage']['kernel_sha256']==expected
        accepted[name]=row
catalog['native']=accepted
catalog['native_additional']=additional
for directory in sorted(cache.glob('tk283-staging-library-*')):
    if not (directory/'run.json').exists():continue
    record=json.loads((directory/'run.json').read_text())
    if record['exit_code'] or record['source_changes_during_run']:continue
    before=record['modified_source_hashes_before']
    if not set(required)<=set(before):continue
    assert all(before[path]==sha for path,sha in required.items())
    if budget['applied']:
        assert budget['passed'] and all(before.get(path)==sha for path,sha in budget['source_sha256'].items())
    assert before==record['modified_source_hashes_after']
    if not all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in before.items()):continue
    output=(directory/'test.log').read_text()
    assert re.search(r'test result: ok\. 383 passed; 0 failed; 8 ignored;',output)
    catalog['library']=str(directory)
for directory in sorted(cache.glob('tk283-staging-copies-post-*')):
    if not (directory/'validation.json').exists():continue
    record=json.loads((directory/'validation.json').read_text())
    before=record.get('modified_source_hashes_before',{})
    if not record['passed'] or not set(required)<=set(before):continue
    assert all(before[path]==sha for path,sha in required.items())
    if budget['applied']:
        assert all(before.get(path)==sha for path,sha in budget['source_sha256'].items())
    assert before==record['modified_source_hashes_after'] and not record['source_changes_during_run']
    assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in before.items())
    assert len(record['results'])==8 and all(row['exit_code']==0 for row in record['results'])
    assert 'Ran 52 tests in ' in (directory/'0.log').read_text()
    assert f'Ran {frame_tests} tests in ' in (directory/'1.log').read_text()
    catalog['post']=str(directory)
checks=[('graph','staging-wide-graph-readback/gnu-verification.json'),
        ('gnu','staging-wide-gnu-readback/readback.json'),
        ('difference','staging-wide-kernel-diff.json'),
        ('controls','staging-wide-controls/controls.json')]
for name,relative in checks:
    path=cache/relative
    if not path.exists():continue
    report=json.loads(path.read_text())
    assert report['passed']
    if name in ['graph','gnu','difference']:assert report['kernel_sha256']==expected
    if name=='controls':
        assert report['unchanged_positive_report_equal'] and len(report['rejections'])==11
        assert all(row['rejected'] for row in report['rejections'])
        assert json.loads((path.parent/'positive.log').read_text())['kernel_sha256']==expected
    catalog[name]=str(path if name=='difference' else path.parent)
catalog_path.write_text(json.dumps(catalog,indent=2)+'\n')
print(json.dumps(dict(kernel_sha256=expected,accepted_native_cohorts=sorted(accepted),
                     accepted_additional_semantic_cohorts=sorted(additional),
                     full_library_accepted='library' in catalog,post_accepted='post' in catalog)))
