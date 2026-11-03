#!/usr/bin/env python3
"""Observe ordinary Topal actions on actual shared file-description fields."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess
import shutil
import tempfile
import threading
import time

import topal_gate as GATE
import positional_gate as POSITIONAL
from qemu_control import stopped_status

LAB = Path(__file__).resolve().parent


def selected_source(text):
    if '  file-position-library is' in text:
        return text
    declaration = ('  file-position-library is lang systems source-library (\n'
        '    source is "policy/file-position.t",\n'
        '    export is advance,\n'
        '    interface is borrowed-byte-region-instance-action-v1,\n'
        '    stack-limit-bytes is 4096,\n'
        '    runtime-limit-bytes is 65536\n  ),\n')
    return text.replace('  region-library is', declaration + '  region-library is')


def build(destination, lifetime, source_path=None):
    original = POSITIONAL.GATE.SOURCE_PATH
    source = destination.parent/'file-region-root.t'
    source.write_text(selected_source((source_path or original).read_text()))
    POSITIONAL.GATE.SOURCE_PATH = source
    try:
        return POSITIONAL.build(destination, lifetime)
    finally:
        POSITIONAL.GATE.SOURCE_PATH = original


def observe(root, values, timeout, cases=(), **mode):
    root.mkdir(parents=True)
    artifact = json.loads(values[3].read_text())
    debug, failed, result, script = (root/name for name in
        ('debug.sock', 'failed.txt', 'observation.json', 'file-region.gdb'))
    config = dict(output=str(root), artifact=artifact, cases=cases, **mode)
    controlled = bool(mode)
    qmp = root/'qmp.sock'
    disconnect = 'disconnect' if controlled else 'detach'
    script.write_text(f"set pagination off\nset confirm off\nfile {values[3].parent/'kernel.elf'}\n"
        f"target remote {debug}\npython\nimport sys\nsys.path.insert(0,{str(LAB)!r})\n"
        f"import file_region_inspect\nfile_region_inspect.install({config!r})\nend\ncontinue\n{disconnect}\n")
    outcome = []
    stopped = []

    def inspect():
        try:
            until = time.monotonic()+10
            while not debug.exists() and time.monotonic() < until:
                time.sleep(.02)
            run = subprocess.run(['gdb', '-q', '-batch', '-x', str(script)],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout+10)
            (root/'debug.log').write_bytes(run.stdout)
            outcome.append(run.returncode)
        except BaseException as error:
            failed.write_text(repr(error))

    thread = threading.Thread(target=inspect, daemon=True)
    original = GATE.qemu_command
    original_markers = GATE.MARKERS

    def command(*args):
        thread.start()
        command = original(*args)+['-S', '-gdb', f'unix:{debug},server=on,wait=off']
        if controlled:
            command += ['-qmp', f'unix:{qmp},server=on,wait=off']
        return command

    def failure():
        if failed.exists():
            return failed.read_text()
        if outcome and outcome != [0]:
            return 'file-region debugger failed: '+repr(outcome)
        if controlled and result.exists() and outcome == [0]:
            try:
                if not stopped:
                    stopped.append(stopped_status(qmp))
            except BaseException as error:
                failed.write_text(repr(error))
                return failed.read_text()
            return 'controlled persistent field observation complete'
        return None

    GATE.qemu_command = command
    if controlled:
        # Controlled terminal tests can emit every normal root marker. Keep
        # observation running until the inspector has disconnected and QMP
        # proves the actual stopped state, before observe_gate tears QEMU down.
        GATE.MARKERS = (*original_markers, b"FILE_REGION_CONTROL_REQUIRES_DEBUGGER_DISCONNECT")
    try:
        try:
            GATE.observe_gate(values[0], values[1], root/'serial.log', root/'qemu.log',
                              timeout, failure_check=failure)
        except GATE.GateError as error:
            if not controlled or failed.exists() or not result.exists():
                details = failed.read_text() if failed.exists() else ''
                raise GATE.GateError(str(error)+'\n'+details) from error
        else:
            if controlled:
                raise GATE.GateError('controlled persistent action resumed ordinary completion')
    finally:
        GATE.qemu_command = original
        GATE.MARKERS = original_markers
    thread.join(timeout=5)
    if outcome != [0] or failed.exists() or not result.exists():
        raise GATE.GateError('persistent field inspector did not complete: '+repr(outcome))
    observation = json.loads(result.read_text())
    expected = ('expected-pre-entry-refusal' if mode.get('fault') else
                'expected-false-action-fatal' if mode.get('false_action') else
                'pass-controlled-inactive-terminal' if mode.get('terminate_inactive') else
                'expected-wrong-source-byte-rejection' if mode.get('wrong_codec') else 'pass')
    if observation['result'] != expected or (controlled and not stopped):
        raise GATE.GateError('persistent field inspector missed completion')
    record = dict(observation=observation, linked_kernel_sha256=GATE.digest(values[3].parent/'kernel.elf'),
                static_user_image_sha256=GATE.digest(values[5]),
                source_library=artifact['file_position_library'], binding=artifact['file_position_runtime'])
    if controlled:
        record.update(debugger_completed_before_teardown=True, stopped_target=stopped[0])
    return record


def verify(timeout=120, write_evidence=False):
    record = dict(schema='topal-file-position-region-qemu/1',
        harness_sha256=GATE.digest(Path(__file__)), qmp_control_sha256=GATE.digest(LAB/"qemu_control.py"), inspector_sha256=GATE.digest(LAB/'file_region_inspect.py'))
    record['caller_sources'] = {name: GATE.digest(LAB/name) for name in (
        'static_positional_file_checks.py', 'static_named_file_checks.py',
        'static_volatile_file_checks.py', 'static_process_checks.py')}
    record['linux_controls'] = {}
    for name in ('volatile', 'named', 'positional'):
        path = LAB/('results/topal-'+name+'-file-linux-control.json')
        control = json.loads(path.read_text())
        if (control['result'] != 'pass' or control['kernel_release'] != '7.2.9'
                or control['harness_sha256'] != GATE.digest(LAB/(name+'_control.py'))
                or control['source_sha256'] != GATE.digest(LAB/(name+'_file_control.c'))):
            raise GATE.GateError('persistent field qualification needs current pinned Linux controls')
        record['linux_controls'][name] = GATE.digest(path)
    with tempfile.TemporaryDirectory(prefix='topal-file-region-', dir='/tmp') as temporary:
        root = Path(temporary)
        for name, lifetime in [('primary', False), ('lifetime', True)]:
            values, calls = build(root/(name+'-artifact'), lifetime)
            record[name] = observe(root/name, values, timeout, calls)
            if lifetime:
                record['inactive_terminal'] = observe(root/'inactive-terminal', values, timeout, calls,
                                                      terminate_inactive=True)
            if name == 'primary':
                for fault in ('owner-prefix', 'field-stride', 'interrupts-enabled', 'dirty-arena', 'generation-exhausted'):
                    record[fault] = observe(root/fault, values, timeout, fault=fault)
        for name, interface, schema in (
            ('legacy', 'borrowed-byte-region-action-v1', 'topal.kernel-region-action-source-object.x86_64/1'),
            ('unscoped_frame', 'borrowed-byte-region-framed-action-v1', 'topal.kernel-region-frame-source-object.x86_64/1'),
        ):
            legacy = root/(name+'-root.t')
            text = selected_source(GATE.SOURCE_PATH.read_text())
            start = text.index('  file-position-library is')
            end = text.index('  ),\n', start)+len('  ),\n')
            declaration = text[start:end].replace('borrowed-byte-region-instance-action-v1', interface)
            legacy.write_text(text[:start]+declaration+text[end:])
            values, calls = build(root/(name+'-artifact'), False, legacy)
            record[name] = observe(root/name, values, timeout, calls)
            if record[name]['source_library']['schema'] != schema:
                raise GATE.GateError('legacy control did not select its frozen transport')
        for name, mode in [('wrong-codec', dict(wrong_codec=True)), ('false-action', dict(false_action=True))]:
            libraries = root/name/'library'
            shutil.copytree(GATE.LINUX_KERNEL/'kernel', libraries)
            helper = libraries/'support/word-bytes.t'
            source = helper.read_text()
            if name == 'wrong-codec':
                changed = source.replace('offset-bytes is 0, value is byte', 'offset-bytes is 0, value is 0')
            else:
                changed = source.replace('offset-bytes is 7, value is byte)\n    true then true',
                                         'offset-bytes is 7, value is byte)\n    true then false')
            if changed == source:
                raise GATE.GateError('source control did not change its ordinary codec')
            helper.write_text(changed)
            selected = root/(name+'-root.t')
            selected.write_text(selected_source(GATE.SOURCE_PATH.read_text()))
            original = GATE.SOURCE_PATH
            GATE.SOURCE_PATH = selected
            try:
                values = GATE.build_image(root/name/'artifact', library_root=libraries)
            finally:
                GATE.SOURCE_PATH = original
            record[name] = observe(root/name/'observe', values, timeout, **mode)
        absent = root/'absent-root.t'
        source = selected_source(GATE.SOURCE_PATH.read_text())
        start = source.index('  file-position-library is')
        end = source.index('  ),\n', start)+len('  ),\n')
        absent.write_text(source[:start]+source[end:])
        original_source, original_build = GATE.SOURCE_PATH, GATE.build_image
        GATE.SOURCE_PATH = absent
        try:
            values = original_build(root/'absent-artifact')
            metadata = json.loads(values[3].read_text())
            if metadata.get('file_position_library') is not None or metadata.get('file_position_runtime') is not None:
                raise GATE.GateError('absent action retained source or invocation storage')
            GATE.build_image = lambda *_a, **_k: values
            record['absent'] = GATE.verify(timeout, False)
        finally:
            GATE.SOURCE_PATH, GATE.build_image = original_source, original_build
    if write_evidence:
        (LAB/'results/topal-file-position-region-gate.json').write_text(
            json.dumps(record, indent=2, sort_keys=True)+'\n')
    print('PASS persistent field actions, scalar/Boolean transport, caller cleanup and position oracles', flush=True)
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--write-evidence', action='store_true')
    args = parser.parse_args()
    verify(args.timeout, args.write_evidence)
