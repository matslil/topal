#!/usr/bin/env python3
"""Replay frozen native publication records and model suites without a VM."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

from archive_publication import SCHEMA, closed_archive
from process_native import require
from replay_process import checked


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    archive, output = args.archive.resolve(), args.output.resolve()
    root = Path(__file__).resolve().parents[3]
    require(all(p.is_relative_to(root/'linux-kernel') for p in [archive, output]), 'Linux replay paths')
    manifest_raw = closed_archive(archive)
    manifest = json.loads(manifest_raw)
    report = json.loads((archive/'report.json').read_text())
    require(report['passed'] and report['schema'] == SCHEMA, 'publication archive schema')
    require(len(report['cohorts']) == 21, 'publication archive VM count')
    output.mkdir(parents=True, exist_ok=False)

    def write(name, raw):
        target = (output/name).resolve()
        require(target.is_relative_to(output), 'replay destination path')
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)

    def unpack(name, fact=None):
        raw = gzip.decompress((archive/(name+'.gz')).read_bytes())
        return checked(raw, fact) if fact is not None else raw

    unpack('execution/driver.py', report['driver'])
    previous = unpack('prior/report.json', report['prior_archive']['report'])
    unpack('prior/manifest.json', report['prior_archive']['manifest'])
    write('previous.json', previous)
    for name, fact in report['selected_inputs'].items():
        write(name, unpack('inputs/'+name, fact))
    # Restore complete build/native records, including unchanged raw CPU logs
    # and exact GDB programs. Full acceptance uses the frozen source reader.
    for name in manifest:
        if name.startswith(('cohorts/', 'retained/')):
            require(name.endswith('.gz'), 'compressed publication evidence')
            write('linux-kernel/native/'+name[:-3], unpack(name[:-3]))
    write('expected.json', json.dumps(report).encode())
    script = '''import json,sys
from pathlib import Path
sys.path.insert(0,'linux-kernel/tools/process-memory')
from archive_publication import COHORTS,TOTALS,native_build,retained
from readback_applications import facts as application_facts
expected=json.loads(Path('expected.json').read_text())
previous=json.loads(Path('previous.json').read_text())
native=Path('linux-kernel/native').resolve()
assert [(row['label'],sum(r['label']==row['label'] for r in expected['cohorts']))
        for row in expected['cohorts']]==[(label,count) for label,test,count in COHORTS for _ in range(count)]
rows=[]
for i,wanted in enumerate(expected['cohorts']):
    build=native/'cohorts'/str(i)/'build'
    actual=native_build(build,previous)
    assert actual=={k:v for k,v in wanted.items() if k not in ['label','native_build']}
    rows.append(actual)
summary={key:sum(row['summary'][key] for row in rows) for key in TOTALS}
summary.update(VMs=21,tests=7,controls=sum(len(row['summary']['controls']) for row in rows))
assert summary==expected['native_summary']
binding=application_facts(native/'retained/build')
def old_read(name):return (native/'retained'/name).read_bytes()
earlier=retained(old_read,previous,previous['artifacts']['kernel.elf'],binding)
assert earlier==expected['retained']
result=dict(passed=True,native_summary=summary,
    publication_controls=[len(row['summary']['controls']) for row in rows],
    retained_applications=len(earlier['applications']),retained_interfaces=len(earlier['interfaces']),
    retained_processes=len(earlier['processes']))
Path('records.json').write_text(json.dumps(result)+'\\n')
print('Frozen 21 publication, 51 application, nine interface and twenty process records passed')
'''
    command = ['rtk', 'proxy', 'python3', '-c', script]
    result = subprocess.run(command, cwd=output, capture_output=True, text=True)
    write('records.stdout.log', result.stdout.encode())
    write('records.stderr.log', result.stderr.encode())
    write('records.command.json', json.dumps(dict(argv=command, exit_code=result.returncode)).encode())
    require(result.returncode == 0, 'frozen publication records failed; inspect records.stderr.log')
    suites = {}
    for name, count in [('user-memory', 203), ('process-memory', 59)]:
        command = ['rtk', 'proxy', 'python3', '-m', 'unittest', 'discover',
                   '-s', 'linux-kernel/tools/'+name, '-p', 'test_*.py']
        result = subprocess.run(command, cwd=output, capture_output=True, text=True)
        write(name+'-models.stdout.log', result.stdout.encode())
        write(name+'-models.stderr.log', result.stderr.encode())
        write(name+'-models.command.json', json.dumps(dict(argv=command, exit_code=result.returncode)).encode())
        require(result.returncode == 0 and f'Ran {count} tests' in result.stderr, 'frozen '+name+' coverage failed')
        suites[name] = dict(passed=True, tests=count, exit_code=result.returncode)
    completed = json.loads((output/'records.json').read_text())
    completed.update(model_suites=suites, archive_manifest_sha256=hashlib.sha256(manifest_raw).hexdigest(),
                     scope='Frozen source-bound saved-record comparisons and Python model suites; no VM execution or kernel rebuild')
    # This is the overall completion marker. records.json alone is insufficient.
    write('replayed.json', (json.dumps(completed, indent=2, sort_keys=True)+'\n').encode())
    print(json.dumps(completed))


if __name__ == '__main__':
    main()
