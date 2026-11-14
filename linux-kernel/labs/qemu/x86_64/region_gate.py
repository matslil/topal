#!/usr/bin/env python3
"""Qualify actual root-owned byte loans and ordinary Topal helper execution."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time
import topal_gate as GATE
from qemu_control import stopped_status

LAB = Path(__file__).resolve().parent


def without_region(source):
    start = source.index('  region-library is lang systems source-library (')
    end = source.index('  ),\n', start) + len('  ),\n')
    return (source[:start] + source[end:]).replace(
        '                                                                      region library apply\n', ''
    ).replace('observed : Nat is region byte load (offset-bytes is 1)',
              'observed : Nat is region byte load (offset-bytes is 0)')


def observe(root, values, timeout, **mode):
    root.mkdir(parents=True)
    artifact = json.loads(values[3].read_text())
    debug, failed, result, script = (root / name for name in
        ('debug.sock', 'failed.txt', 'observation.json', 'region.gdb'))
    config = dict(output=str(root), artifact=artifact, **mode)
    controlled = bool(mode.get('fault') or mode.get('wrong_helper') or mode.get('false_action'))
    qmp = root/'qmp.sock'
    ending = 'disconnect\npython\nimport time\ntime.sleep(.2)\nend' if controlled else 'detach'
    script.write_text(f"set pagination off\nset confirm off\nfile {values[3].parent/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import region_inspect\nregion_inspect.install({config!r})\nend\ncontinue\n{ending}\n")
    outcome = []
    stopped = []

    def inspect():
        try:
            until = time.monotonic() + 10
            while not debug.exists() and time.monotonic() < until:
                time.sleep(.02)
            run = subprocess.run(['gdb', '-q', '-batch', '-x', str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout + 10)
            (root / 'debug.log').write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=inspect, daemon=True)
    original_build, original_command, original_observe = GATE.build_image, GATE.qemu_command, GATE.observe_gate
    original_markers = GATE.MARKERS

    def command(*args):
        thread.start()
        command = original_command(*args) + ['-S', '-gdb', f'unix:{debug},server=on,wait=off']
        if controlled:
            command += ['-qmp', f'unix:{qmp},server=on,wait=off']
        return command

    def failure():
        if failed.exists():
            return failed.read_text()
        if outcome and outcome != [0]:
            return 'region debugger failed: ' + repr(outcome)
        if controlled and result.exists() and outcome == [0]:
            try:
                if not stopped:
                    stopped.append(stopped_status(qmp))
            except BaseException as error:
                failed.write_text(repr(error))
                return failed.read_text()
            return 'controlled root region refusal'
        return None

    GATE.build_image = lambda *_args, **_kwargs: values
    GATE.qemu_command = command
    GATE.observe_gate = lambda *args, **kwargs: original_observe(*args, **kwargs, failure_check=failure)
    if controlled:
        GATE.MARKERS = (*original_markers, b'ROOT_REGION_CONTROL_REQUIRES_DEBUGGER_DISCONNECT')
    runtime = None
    try:
        try:
            runtime = GATE.verify(timeout, False)
        except GATE.GateError:
            if not controlled or failed.exists() or not result.exists():
                raise
        else:
            if controlled:
                raise GATE.GateError('region control escaped its required refusal')
    finally:
        GATE.build_image, GATE.qemu_command, GATE.observe_gate = original_build, original_command, original_observe
        GATE.MARKERS = original_markers
    thread.join(timeout=5)
    if outcome != [0] or failed.exists() or not result.exists():
        raise GATE.GateError('region inspector failed: ' + repr(outcome))
    observation = json.loads(result.read_text())
    expected = ('expected-pre-entry-refusal' if mode.get('fault') else
                'expected-false-action-fatal' if mode.get('false_action') else
                'expected-owner-byte-guard-refusal' if mode.get('wrong_helper') else 'pass')
    if observation['result'] != expected or (controlled and not stopped):
        raise GATE.GateError('wrong root region disposition')
    record = dict(runtime=runtime, observation=observation,
        linked_kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
        source_library=artifact['region_library'], root_binding=artifact['region_runtime'])
    if controlled:
        record.update(debugger_completed_before_teardown=True, stopped_target=stopped[0],
                      post_disconnect_delay_seconds=.2)
    return record


def verify(timeout=120, write_evidence=False):
    original_source, original_build = GATE.SOURCE_PATH, GATE.build_image
    source = (GATE.LINUX_KERNEL/'kernel/policy/bootstrap-byte-copy.t').read_text()
    record = dict(schema='topal-root-region-qemu/1', harness_sha256=GATE.digest(Path(__file__)),
                  qmp_control_sha256=GATE.digest(LAB/'qemu_control.py'),
                  inspector_sha256=GATE.digest(LAB/'region_inspect.py'))
    with tempfile.TemporaryDirectory(prefix='topal-root-region-', dir='/tmp') as temporary:
        root = Path(temporary)
        try:
            primary = original_build(root/'primary-artifact')
            record['primary'] = observe(root/'primary', primary, timeout)
            for fault in ('owner-prefix', 'interrupts-enabled', 'dirty-arena'):
                record[fault] = observe(root/fault, primary, timeout, fault=fault)
            for name, text, mode in [
                ('store24', source.replace('  ()', '  changed : Boolean is region byte store (offset-bytes is 24, value is 37)\n  ()'), dict(store24=True)),
                ('wrong-helper', source.replace('(region, 0, 1)', '(region, 0, 2)'), dict(wrong_helper=True, destination=2)),
            ]:
                libraries = root/name/'library'
                shutil.copytree(GATE.LINUX_KERNEL/'kernel', libraries)
                (libraries/'policy/bootstrap-byte-copy.t').write_text(text)
                values = original_build(root/name/'artifact', library_root=libraries)
                record[name] = observe(root/name/'observe', values, timeout, **mode)
                if record[name]['linked_kernel_sha256'] == record['primary']['linked_kernel_sha256']:
                    raise GATE.GateError('changed region source did not change the linked kernel')
            libraries = root/'false-action'/'library'
            shutil.copytree(GATE.LINUX_KERNEL/'kernel', libraries)
            wrapper = libraries/'policy/bootstrap-byte-copy-frame.t'
            text = wrapper.read_text()
            changed = text.replace('  true\n', '  false\n')
            if text == changed:
                raise GATE.GateError('root false control did not change its actual source result')
            wrapper.write_text(changed)
            values = original_build(root/'false-action'/'artifact', library_root=libraries)
            record['false_action'] = observe(root/'false-action'/'observe', values, timeout, false_action=True)
            legacy = root/'legacy-root.t'
            text = original_source.read_text()
            start = text.index('  region-library is')
            end = text.index('  ),\n', start)+len('  ),\n')
            declaration = text[start:end].replace('bootstrap-byte-copy-frame.t', 'bootstrap-byte-copy.t').replace(
                'borrowed-byte-region-instance-action-v1', 'borrowed-byte-region-v1')
            legacy.write_text(text[:start]+declaration+text[end:])
            GATE.SOURCE_PATH = legacy
            values = original_build(root/'legacy-artifact')
            record['legacy'] = observe(root/'legacy', values, timeout)
            if record['legacy']['source_library']['schema'] != 'topal.kernel-region-source-object.x86_64/3':
                raise GATE.GateError('legacy root did not select the frozen Unit transport')
            absent = root/'absent.t'
            absent.write_text(without_region(original_source.read_text()))
            GATE.SOURCE_PATH = absent
            values = original_build(root/'absent-artifact')
            metadata = json.loads(values[3].read_text())
            if metadata.get('region_library') is not None or metadata.get('region_runtime') is not None:
                raise GATE.GateError('absent declaration retained region ownership')
            GATE.build_image = lambda *_args, **_kwargs: values
            record['absent'] = GATE.verify(timeout, False)
        finally:
            GATE.SOURCE_PATH, GATE.build_image = original_source, original_build
    if write_evidence:
        (LAB/'results/topal-root-region-gate.json').write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    print('PASS root/source/helper frames, real owner bytes, caller restoration, complete cleanup and controls', flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--write-evidence', action='store_true')
    args = parser.parse_args()
    verify(args.timeout, args.write_evidence)
