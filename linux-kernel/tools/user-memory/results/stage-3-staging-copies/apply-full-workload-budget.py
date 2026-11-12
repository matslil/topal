"""Size the full four-image correctness budget only after an observed timeout.

Runs at a bounded-validation boundary, before the full-suite source capture.
Never changes guest code, VM resources, validation assertions or focused budgets.
"""
import ast
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

cache=Path('linux-kernel/.cache/x86_64')
trial=cache/'tk283-staging-focused-1794433773644775945'
record=json.loads((trial/'run.json').read_text())
output=cache/'full-workload-budget-preflight'
output.mkdir()
if record['exit_code']==0:
    (output/'preflight.json').write_text(json.dumps(dict(applied=False,
        reason='Original full workload completed under its existing budget'))+'\n')
    print('Full workload budget unchanged: original trial passed',flush=True)
    sys.exit(0)
assert record['exit_code']==101 and not record['source_changes_during_run']
assert 'full-root guest timed out (1800s)' in (trial/'test.log').read_text()
events=dict(line.split() for line in record['resources']['memory.events'].splitlines())
assert all(events[name]=='0' for name in ['max','oom','oom_kill'])
assert record['resources']['memory.swap.max']=='0'
guest=trial/'temporary/user-extent-callers-2341162-1794433774658651303/user-extent-callers'
assert 'Traceback' not in (guest/'gdb.stderr.log').read_text()
counts=[int(value) for value in re.findall(r'CALLER_PROGRESS (\d+)',(guest/'gdb.stdout.log').read_text())]
assert counts and max(counts)>=4096
identity=json.loads((cache/'before-full-workload-budget-inputs/identity.json').read_text())
assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha for name,sha in identity.items())
root=Path('linux-kernel/labs/qemu/x86_64/frame_root_inspect.py')
callers=Path('linux-kernel/labs/qemu/x86_64/user_extent_callers_inspect.py')
tests=root.with_name('test_frame_progress.py')
assert not tests.exists()
activity='''# TOPAL-SYSTEMS-USER-EXTENT-001: host validation activity only.
class OutputActivity:
    """Bound quiet diagnostic output separately from total workload time."""
    def __init__(self, seconds, now):
        assert isinstance(seconds, int) and seconds > 0
        self.seconds, self.changed, self.stamp = seconds, now, None

    def check(self, stamp, now):
        if stamp != self.stamp:
            self.stamp, self.changed = stamp, now
        assert now-self.changed < self.seconds, (
            f'full-root guest output inactive ({self.seconds}s)')


'''
source=root.read_text()
marker="def main(*, work_name='guest'"
assert source.count(marker)==1
source=source.replace(marker,activity+marker)
old='         serial_dialogue=()):'
assert source.count(old)==1
source=source.replace(old,'         serial_dialogue=(), output_activity_seconds=None):')
old='    assert isinstance(guest_timeout_seconds, int) and 0 < guest_timeout_seconds <= 3600'
assert source.count(old)==1
source=source.replace(old,old+'''
    if output_activity_seconds is not None:
        assert isinstance(output_activity_seconds, int) and 0 < output_activity_seconds <= guest_timeout_seconds''')
old='                deadline = time.monotonic() + guest_timeout_seconds\n                sent_read = sent_command = False'
assert source.count(old)==1
source=source.replace(old,'''                deadline = time.monotonic() + guest_timeout_seconds
                activity = (OutputActivity(output_activity_seconds, time.monotonic())
                            if output_activity_seconds is not None else None)
                sent_read = sent_command = False''')
old="                    assert time.monotonic() < deadline, f'full-root guest timed out ({guest_timeout_seconds}s)'"
assert source.count(old)==1
source=source.replace(old,'''                    if activity is not None:
                        activity.check((len(received), (work/'gdb.stdout.log').stat().st_size), time.monotonic())
'''+old)
caller_source=callers.read_text()
old="                 # The full fixture repeats startup over four process images.\n                 guest_timeout_seconds=1200 if selection['shell_io_only'] else 900 if focused else 1800)"
assert caller_source.count(old)==1
caller_source=caller_source.replace(old,"""                 # Four images retain the focused per-image correctness budget.
                 # A separate output-inactivity guard detects a quiet guest;
                 # final native/model assertions still decide correctness.
                 guest_timeout_seconds=1200 if selection['shell_io_only'] else 900 if focused else 4*900,
                 output_activity_seconds=None if focused else 300)""")
test_source='''"""TOPAL-SYSTEMS-USER-EXTENT-001 host-runner guards.

These do not qualify guest progress or kernel behavior.
"""
import unittest
from frame_root_inspect import OutputActivity


class OutputActivityTests(unittest.TestCase):
    def test_quiet_output_fails_at_the_inactivity_limit(self):
        guard = OutputActivity(5, 0)
        guard.check((0, 0), 0)
        guard.check((0, 0), 4.99)
        with self.assertRaisesRegex(AssertionError, 'output inactive'):
            guard.check((0, 0), 5)

    def test_new_debugger_output_resets_only_the_inactivity_clock(self):
        guard = OutputActivity(5, 0)
        guard.check((0, 0), 0)
        guard.check((0, 10), 4)
        guard.check((0, 10), 8.99)
        with self.assertRaises(AssertionError):
            guard.check((0, 10), 9)

    def test_repeated_polls_cannot_keep_quiet_output_alive(self):
        guard = OutputActivity(5, 0)
        guard.check((0, 0), 0)
        guard.check((10, 0), 4)
        for now in [5, 6, 7, 8]:
            guard.check((10, 0), now)
        with self.assertRaises(AssertionError):
            guard.check((10, 0), 9)

    def test_invalid_inactivity_limits_fail_closed(self):
        for seconds in [0, -1, 1.5]:
            with self.subTest(seconds=seconds), self.assertRaises(AssertionError):
                OutputActivity(seconds, 0)


if __name__ == '__main__':
    unittest.main()
'''
for content in [source,caller_source,test_source]:ast.parse(content)
root.write_text(source)
callers.write_text(caller_source)
tests.write_text(test_source)
command=[sys.executable,'-m','unittest','discover','-s',str(root.parent),'-p',tests.name,'-v']
with (output/'unit-tests.log').open('w') as log:
    result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
assert result.returncode==0,(output/'unit-tests.log').read_text()
assert 'Ran 4 tests in ' in (output/'unit-tests.log').read_text()
report=dict(applied=True,passed=True,previous_full_deadline_seconds=1800,
    full_deadline_seconds=3600,output_inactivity_seconds=300,unit_tests=4,
    observed_prior_helper_events=max(counts),
    source_sha256={str(path):hashlib.sha256(path.read_bytes()).hexdigest() for path in [root,callers,tests]},
    rationale='Full correctness test repeats four process images and retains the existing900s focused per-image budget. Earlier1800s attempt was still validating thousands of calls. Total3600s ceiling plus300s output-inactivity guard retain bounded execution and every correctness assertion. Prior timeouts remain failures; this is not a pass under the old deadline. VM memory, isolation and host limits are unchanged.')
(output/'preflight.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report),flush=True)
