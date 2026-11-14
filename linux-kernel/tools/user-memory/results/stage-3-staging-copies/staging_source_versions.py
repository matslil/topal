"""Validate the actual input scope of each retained successful run."""
import gzip
import hashlib
import json
import re


def verify_records(root, selected, read, base_read):
    digest = lambda data: hashlib.sha256(data).hexdigest()
    load = lambda path: json.loads(read(path))

    def record(path, count, expected=None):
        row = load(path)
        assert row.get('passed', row.get('exit_code') == 0), str(path)
        assert not row['source_changes_during_run'], str(path)
        before = row['modified_source_hashes_before']
        assert before == row['modified_source_hashes_after'] and len(before) == count
        if expected is not None:
            assert before == expected
        for name, sha in before.items():
            entry = selected[name]
            assert sha == entry['sha256']
            assert digest(gzip.decompress(read(root/entry['snapshot']))) == sha
        resources = row['resources']
        assert int(resources['memory.max']) > 0 and resources['memory.swap.max'] == '0'
        assert 0 < int(resources['memory.peak']) <= int(resources['memory.max'])
        events = dict(line.split() for line in resources['memory.events'].splitlines())
        assert all(events[name] == '0' for name in ['max', 'oom', 'oom_kill', 'oom_group_kill'])
        return before

    native_names = ['source', 'storage', 'serial', 'pinned', 'pipes', 'poll']
    context_names = {'linux-kernel/labs/qemu/x86_64/frame_root_inspect.py',
                     'linux-kernel/labs/qemu/x86_64/user_extent_callers_inspect.py'}
    for name in native_names:
        directory = root/'native'/name
        record(directory/'run.json', 19)
        assert '1 passed; 0 failed;' in read(directory/'test.log').decode()
        versions = load(directory/'source-overrides.json')
        assert versions['inputs'] == {} and versions['recorded_modified_inputs'] == 19
        assert set(versions['baseline_context']) == context_names
        for path, entry in versions['baseline_context'].items():
            data = gzip.decompress(read(directory/entry['snapshot']))
            assert digest(data) == entry['sha256'] and data == base_read(path)
            assert entry['sha256'] != selected[path]['sha256']
        observed = json.loads(gzip.decompress(read(directory/'observations.json.gz')))
        assert observed['passed'] and observed['copy_transfer_observation']['passed']

    freeze = record(root/'library-validation.json', 22)
    assert re.search(r'test result: ok\. 383 passed; 0 failed; 8 ignored;',
                     gzip.decompress(read(root/'library-test.log.gz')).decode())
    record(root/'post-validation.json', 22, freeze)
    post = load(root/'post-validation.json')
    assert len(post['results']) == 8 and all(row['exit_code'] == 0 for row in post['results'])
    assert 'Ran 52 tests in ' in read(root/'post/0.log').decode()
    assert 'Ran 8 tests in ' in read(root/'post/1.log').decode()
    for name in ['exec', 'signals']:
        directory = root/'native-additional'/name
        record(directory/'run.json', 22, freeze)
        assert '1 passed; 0 failed;' in read(directory/'test.log').decode()
        observed = json.loads(gzip.decompress(read(directory/'observations.json.gz')))
        assert observed['passed'] and 'extent_call_coverage' not in observed
        component = json.loads(gzip.decompress(read(directory/'source-component.json.gz')))
        assert component['mode'] == name
    record(root/'private-clean/run.json', 19)
    assert '3 passed; 0 failed;' in read(root/'private-clean/test.log').decode()
    assert gzip.decompress(read(root/'private-clean/component/probe.stdout.log.gz')).strip() == b'PRIVATE_CLEAN_COMPONENT_PASS probes=210947'
    budget = load(root/'method/full-workload-budget/preflight.json')
    assert budget['passed'] and budget['applied'] and budget['unit_tests'] == 4
    assert budget['previous_full_deadline_seconds'] == 1800
    assert budget['full_deadline_seconds'] == 3600 and budget['output_inactivity_seconds'] == 300
    assert all(freeze[name] == sha for name, sha in budget['source_sha256'].items())
    return dict(native_source_freezes=6,additional_native_source_freezes=2,
                component_source_freezes=1,full_library_and_post_source_freezes=2)
