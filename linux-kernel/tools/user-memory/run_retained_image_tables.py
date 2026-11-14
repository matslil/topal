"""Qualify scoped inactive ownership plus the complete predecessor matrix."""
import argparse,hashlib,json,os,re,subprocess,time,traceback
from pathlib import Path
from run_image_backing import ROOT
from run_policy_failure import hashes,watched


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    output=parser.parse_args().output.resolve()
    assert output.is_relative_to(ROOT/'linux-kernel')
    group=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    resources=Path('/sys/fs/cgroup')/group.lstrip('/')
    assert 'topal-bounded-' in group and (resources/'memory.swap.max').read_text().strip()=='0'
    output.mkdir(parents=True,exist_ok=False)
    temporary=output/'temporary';temporary.mkdir()
    names=set(watched())
    for base in ['src/topal-language/src','src/topal-semantics/src']:
        names.update(p.relative_to(ROOT).as_posix() for p in (ROOT/base).rglob('*.rs'))
    names.update(['docs/systems-profile.md','se/requirements.md','spec/systems-profile.md',
                  'src/topal-compiler/se-requirements.md','linux-kernel/design/retained-image-tables.md',
                  'linux-kernel/design/inactive-image-table-construction.md',
                  'linux-kernel/design/kernel-image-protection.md','linux-kernel/design/kernel-image-backing-preparation.md',
                  'linux-kernel/design/kernel-image-boot-binding.md','linux-kernel/design/boot-processor-topology.md',
                  'linux-kernel/design/inactive-boot-coverage.md','linux-kernel/design/decision-register.md',
                  'linux-kernel/tools/user-memory/image_backing_source.rs'])
    names=sorted(names);before=hashes(names)
    env={k:v for k,v in os.environ.items() if not k.startswith('TOPAL_TEST_') and k!='TOPAL_PINNED_STATIC_USER_IMAGE'}
    env.update(RUST_MIN_STACK='16777216',RUST_TEST_THREADS='1',PYTHONDONTWRITEBYTECODE='1',TMPDIR=str(temporary))
    began=time.monotonic()
    report=dict(passed=False,commands={},source_hashes_before=before,
                scope='Actual whole pinned inactive banks through ordinary user exec/exit and restored-root retirement, plus complete immediate-discard/backing predecessor matrix; no protected activation or continuous exclusion')
    def save():
        report['resources']={n:(resources/n).read_text().strip() for n in ['memory.max','memory.swap.max','memory.peak','memory.events']}
        (output/'run.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    def command(name,argv):
        start=time.monotonic();report['commands'][name]=dict(argv=['rtk','proxy',*argv],state='running');save()
        print(name,'running',flush=True)
        with (output/(name+'.stdout.log')).open('w') as out,(output/(name+'.stderr.log')).open('w') as err:
            result=subprocess.run(['rtk','proxy',*argv],cwd=ROOT,env=env,stdout=out,stderr=err)
        report['commands'][name].update(state='finished',exit_code=result.returncode,seconds=time.monotonic()-start);save()
        print(name,result.returncode,flush=True);assert result.returncode==0,name
    save()
    try:
        command('retained-guest',['cargo','test','-p','topal-compiler','--lib','--locked','-j1',
                'retained_tables_cross_ordinary_exec_and_restore_before_whole_retirement','--','--ignored','--test-threads=1','--nocapture'])
        log=(output/'retained-guest.stdout.log').read_text()+(output/'retained-guest.stderr.log').read_text()
        matches=re.findall(r'final retained image tables candidate: ([^\n]+)',log);assert len(matches)==1
        build=Path(matches[0]).resolve();assert build.is_relative_to(ROOT/'linux-kernel')
        command('retained-readback',['python3','-B','linux-kernel/tools/user-memory/readback_retained_image_tables.py',str(build)])
        result=json.loads((build/'image-tables-observations.json').read_text())
        readback=json.loads((build/'retained-image-tables-readback.json').read_text())
        assert result['passed'] and result['native_execution'] and result['retained'] and result['retention_events']>=4
        assert result['successful_execs']==3 and result['checked_console_copies']==8
        assert readback['passed'] and readback['completed_coverage_checks']==3 and len(readback['rejected_offline_controls'])==21
        report.update(native_build=build.relative_to(ROOT).as_posix(),native=result,readback=readback,
                      component=json.loads((build/'image-tables-component.json').read_text()),
                      binding=json.loads((build/'private-image-binding.json').read_text()),
                      ordinary_owners=json.loads((build/'root-backing-owners.json').read_text()))
        command('predecessor-matrix',['python3','-B','linux-kernel/tools/user-memory/run_image_backing.py',str(output/'predecessors'),'--tables'])
        previous=json.loads((output/'predecessors/run.json').read_text())
        assert previous['passed'] and len(previous['commands'])==15
        assert previous['native']['kernel_sha256']=='d86386762f9842244870d593ebf3c073d2244e8110e8f93e7806085151b12a6e'
        report['predecessor_matrix']=previous
        report['passed']=True
    except BaseException:
        report['failure']=traceback.format_exc();print(report['failure'],flush=True)
    finally:
        report['source_hashes_after']=hashes(names)
        report['passed'] &= before==report['source_hashes_after']
        report['seconds']=time.monotonic()-began;save()
    raise SystemExit(0 if report['passed'] else 1)


if __name__=='__main__': main()
