#!/usr/bin/env python3
"""Record bounded stage-1 model evidence; does not qualify native kernel code."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import time
import unittest


ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
DOCUMENTS = [
    'linux-kernel/design/process-memory-ownership.md',
    'linux-kernel/step-4-process-memory-ownership.md',
    'linux-kernel/tools/process-memory/README.md',
]
AUTHORITIES = {
    'docs/systems-profile.md': '### Process address-space ownership',
    'se/requirements.md': '## TOPAL-REQ-SYSTEMS-PROCESS-MM-001',
    'spec/systems-profile.md': '### TOPAL-SYSTEMS-PROCESS-MM-001',
    'src/topal-compiler/se-requirements.md': '## TOPAL-COMP-SYSTEMS-PROCESS-MM-001',
}
BASELINE = [
    'src/topal-compiler/src/systems_process_children.rs',
    'src/topal-compiler/src/systems_process_pair.rs',
    'src/topal-compiler/src/systems_process_entry_start.rs',
    'src/topal-compiler/src/runtime/kernel_frame_memory.ll',
]


def cgroup():
    lines = Path('/proc/self/cgroup').read_text().splitlines()
    group = next(line.split(':', 2)[2] for line in lines if line.startswith('0::'))
    directory = Path('/sys/fs/cgroup') / group.lstrip('/')
    maximum = (directory / 'memory.max').read_text().strip()
    swap = (directory / 'memory.swap.max').read_text().strip()
    if 'topal-bounded-' not in group or maximum == 'max' or swap != '0':
        raise RuntimeError('qualification requires the normal bounded, swap-free wrapper')
    return dict(group=group, memory_max_bytes=int(maximum), swap_max_bytes=int(swap),
                observed_peak_bytes=int((directory / 'memory.peak').read_text()),
                events={k: int(v) for k, v in
                        (line.split() for line in (directory / 'memory.events').read_text().splitlines())})


def links():
    checked = []
    for source in DOCUMENTS:
        for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)', (ROOT / source).read_text()):
            if target.startswith(('https://', 'http://', '#')):
                continue
            file, _, anchor = target.partition('#')
            destination = ((ROOT / source).parent / file).resolve()
            if not destination.is_relative_to(ROOT) or not destination.is_file():
                raise RuntimeError(f'missing local link: {source}: {target}')
            # Added authority anchors are checked against their exact headings.
            if anchor and destination.relative_to(ROOT).as_posix() in AUTHORITIES:
                heading = AUTHORITIES[destination.relative_to(ROOT).as_posix()]
                actual = next((line.lstrip('# ').strip() for line in destination.read_text().splitlines()
                               if line.startswith(heading)), '')
                slug = re.sub(r'[^\w\- ]', '', actual.lower()).replace(' ', '-')
                if anchor != slug:
                    raise RuntimeError(f'missing authority heading: {heading}')
            checked.append(dict(source=source, target=target))
    for source, heading in AUTHORITIES.items():
        if (ROOT / source).read_text().count(heading) != 1:
            raise RuntimeError(f'missing or duplicate stable heading: {source}')
    return checked


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / 'linux-kernel'):
        raise RuntimeError('keep Linux qualification files under linux-kernel')
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    resource_before = cgroup()
    stream = io.StringIO()
    suite = unittest.defaultTestLoader.discover(str(HERE), pattern='test_*.py')
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
    log = stream.getvalue()
    (output / 'tests.log').write_text(log)
    print(log, end='')
    # Link checking needs the report destination to exist; failure replaces the
    # placeholder with an explicit unsuccessful report, never passing evidence.
    report_path = output / 'report.json'
    report_path.write_text('{"passed": false, "state": "checking"}\n')
    # Committed report links refer to the historical run. On the first run,
    # require this output itself to be that exact linked destination.
    checked_links = links()
    sources = DOCUMENTS + list(AUTHORITIES) + BASELINE + [
        'linux-kernel/design/decision-register.md', 'linux-kernel/step-4-status.md',
        'linux-kernel/design/serialized-static-process-children.md',
        'linux-kernel/design/resumable-static-process-pair.md',
    ] + [p.relative_to(ROOT).as_posix() for p in sorted(HERE.glob('*.py'))]
    inputs = {source: dict(sha256=hashlib.sha256((ROOT / source).read_bytes()).hexdigest(),
                           bytes=(ROOT / source).stat().st_size) for source in sources}
    resource_after = cgroup()
    passed = result.wasSuccessful() and result.testsRun == 17
    passed = passed and all(v == 0 for v in resource_after['events'].values())
    report = dict(
        schema='topal-process-memory-model-stage-1-v1', passed=passed,
        scope='Portable normal/prepublication-unavailable heap transitions only; no native, physical or publication authority',
        tests=dict(run=result.testsRun, failures=len(result.failures), errors=len(result.errors),
                   skipped=len(result.skipped), application_seeds=32, application_transitions=3200,
                   saved_history_controls=19, inconsistent_state_controls=7),
        elapsed_seconds=time.monotonic() - started, inputs=inputs,
        log=dict(file='tests.log', sha256=hashlib.sha256(log.encode()).hexdigest()),
        links=checked_links, resources_before=resource_before, resources_after=resource_after,
        command=['rtk', 'proxy', 'env', 'RUST_MIN_STACK=16777216', 'PYTHONDONTWRITEBYTECODE=1',
                 'scripts/run_bounded.py', '--', 'python3',
                 'linux-kernel/tools/process-memory/qualify.py', '--output',
                 output.relative_to(ROOT).as_posix()],
        remaining=['native owner/entry/caller binding', 'every overlapping mapping producer',
                   'image/stack/register/descriptor preservation', 'identity width exhaustion',
                   'fatal partial-mutation cleanup', 'physical erasure and translation evidence',
                   'AP/DMA/asynchronous exclusion and fault stability',
                   'ordinary child/pipe/application regressions', 'public source/load/boot reconstruction'])
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    if not passed:
        raise SystemExit(1)
    print(f'Qualification passed: {report_path.relative_to(ROOT)}')


if __name__ == '__main__':
    main()
