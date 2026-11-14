"""Validate complete native boot-owned startup storage in the bounded VM."""
import hashlib,json,os,re,subprocess,sys,time,traceback
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'linux-kernel/tools/process-memory'))
from run_policy_failure import watched,hashes


def main():
    output=Path(sys.argv[1]).resolve()
    assert output.is_relative_to(ROOT/'linux-kernel/.cache/x86_64')
    group=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    resources=Path('/sys/fs/cgroup')/group.lstrip('/')
    assert 'topal-bounded-' in group and (resources/'memory.swap.max').read_text().strip()=='0'
    output.mkdir(parents=True,exist_ok=False);temporary=output/'temporary';temporary.mkdir()
    paths=set(watched())
    for base in ['src/topal-language/src','src/topal-semantics/src']:
        paths.update(str(p.relative_to(ROOT)) for p in (ROOT/base).rglob('*.rs'))
    paths.update(['docs/systems-profile.md','se/requirements.md','spec/systems-profile.md',
        'src/topal-compiler/se-requirements.md','linux-kernel/design/decision-register.md',
        'linux-kernel/design/boot-startup-storage.md','linux-kernel/tools/user-memory/boot_participant_source.rs'])
    before=hashes(sorted(paths));started=time.monotonic()
    environment={k:v for k,v in os.environ.items() if not k.startswith('TOPAL_TEST_') and k!='TOPAL_PINNED_STATIC_USER_IMAGE'}
    environment.update(RUST_MIN_STACK='16777216',RUST_TEST_THREADS='1',PYTHONDONTWRITEBYTECODE='1',TMPDIR=str(temporary))
    record=dict(schema='topal.kernel.boot-startup-storage/1',passed=False,watched_inputs=before,commands={},
        scope='Actual loaded/claimed/probed/restored/retained setup page; no CPU startup or arrival',
        processor_startup=False,participant_arrival=False,protected_context=False,production_qualified=False)
    def save():
        record['resources']={n:(resources/n).read_text().strip() for n in ['memory.max','memory.peak','memory.swap.max','memory.events']}
        (output/'qualification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    def run(label,argv):
        log=output/(label+'.log');begin=time.monotonic()
        record['commands'][label]=dict(argv=['rtk','proxy',*argv],state='running');save()
        with log.open('w') as stream:
            result=subprocess.run(['rtk','proxy',*argv],cwd=ROOT,env=environment,stdout=stream,stderr=subprocess.STDOUT,check=False)
        data=log.read_bytes();text=data.decode()
        counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
        record['commands'][label].update(state='finished',exit_code=result.returncode,seconds=time.monotonic()-begin,
            test_counts=[list(map(int,r)) for r in counts],log=dict(path=str(log.relative_to(ROOT)),bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
        save();print(label,result.returncode,counts,flush=True)
        assert result.returncode==0,label
        return text
    save()
    try:
        test=['cargo','test','-p','topal-compiler','--lib','--locked','-j1']
        run('startup-component',test+['startup_storage_','--','--test-threads=1'])
        text=run('startup-guest',test+['systems_boot_image::startup::tests::startup_storage_is_loaded_claimed_probed_restored_and_retained_in_the_vm',
            '--','--ignored','--exact','--test-threads=1','--nocapture'])
        matches=re.findall(r'final startup storage candidate: ([^\n]+)',text);assert len(matches)==1,matches
        guest=Path(matches[0].strip()).resolve();assert guest.is_relative_to(ROOT/'linux-kernel')
        record['guest_build']=str(guest.relative_to(ROOT))
        run('retained-readback',['python3','-B','linux-kernel/tools/user-memory/readback_retained_image_tables.py',str(guest)])
        run('startup-readback',['python3','-B','linux-kernel/tools/user-memory/readback_boot_startup_storage.py',str(guest)])
        record['startup_readback']=json.loads((guest/'boot-startup-storage-readback.json').read_text())
        record['retained_readback']=json.loads((guest/'retained-image-tables-readback.json').read_text())
        record['native']=json.loads((guest/'image-tables-observations.json').read_text())
        assert record['native']['passed'] and record['native']['native_execution']
        record['component']=json.loads((guest/'image-tables-component.json').read_text())
        record['artifacts']={name:dict(bytes=(guest/name).stat().st_size,sha256=hashlib.sha256((guest/name).read_bytes()).hexdigest())
            for name in ['kernel.elf','bzImage','image-tables.o','root.o','image-tables-plan-binding.json','kernel-image-boot-binding.json']}
        run('boot-adapter-regression',test+['systems_boot_image::','--','--test-threads=1'])
        run('predecessor-matrix',['python3','-B','linux-kernel/tools/user-memory/run_boot_participant_source.py',str(output/'predecessor')])
        prior=json.loads((output/'predecessor/qualification.json').read_text())
        assert prior['passed'] and prior['unchanged_inputs']
        record['predecessor']=dict(commands=prior['commands'],legacy_build=prior['legacy_build'],seconds=prior['seconds'])
        record['passed']=True
    except BaseException:
        record['failure']=traceback.format_exc();print(record['failure'],flush=True)
    finally:
        record['unchanged_inputs']=before==hashes(sorted(paths))
        record['passed'] &= record['unchanged_inputs']
        record['seconds']=time.monotonic()-started;save()
    return 0 if record['passed'] else 1


if __name__=='__main__':raise SystemExit(main())
