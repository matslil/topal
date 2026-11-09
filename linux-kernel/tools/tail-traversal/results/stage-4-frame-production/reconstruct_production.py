"""Restore the production image from retained first-party bytes and a pinned dependency.

Run from the repository root through scripts/run_bounded.py. Boot packaging
reconstructs the complete source/native binding from the retained snapshots,
including their original diagnostic source names; no original absolute source
directory or private qualification cache is needed.
"""
from pathlib import Path
import argparse,gzip,hashlib,json,os,runpy,subprocess


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--kind',choices=('busybox','abi'),default='busybox')
    parser.add_argument('--destination',type=Path,required=True)
    parser.add_argument('--busybox',type=Path)
    args=parser.parse_args()
    root=Path.cwd();out=Path(__file__).resolve().parent
    relative=next(line.split(':',2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    group=Path('/sys/fs/cgroup')/relative.lstrip('/')
    assert 'topal-bounded-' in relative and (group/'memory.swap.max').read_text().strip()=='0', 'run through scripts/run_bounded.py'
    record=json.loads((out/'qualification.json').read_text());assert record['complete'] and record['passed']
    sha=lambda b:hashlib.sha256(b).hexdigest()
    for name,digest in record['sources'].items():
        assert name!='topal-as-dsl.md' and sha((root/name).read_bytes())==digest,('qualified source differs',name)
    destination=args.destination.resolve()
    assert destination.is_relative_to((root/'linux-kernel/.cache').resolve()), 'use an ignored linux-kernel/.cache destination'
    assert not destination.exists(),'destination already exists'
    label='production-'+args.kind
    component=next(c for c in record['components'] if c['label']==label)
    projection=json.loads((out/'production-binaries.json').read_text())['entries'][label]
    def read(info):
        compressed=(out/info['file']).read_bytes();assert sha(compressed)==info['sha256']
        data=gzip.decompress(compressed);assert sha(data)==info['uncompressed_sha256']
        if 'bytes' in info:assert len(data)==info['bytes']
        return data
    if args.kind=='busybox':
        assert args.busybox is not None,'supply the exact pinned BusyBox executable'
        static=args.busybox.read_bytes()
        assert len(static)==record['dependency']['bytes'] and sha(static)==record['dependency']['sha256'],'wrong pinned dependency'
    else:
        assert args.busybox is None
        gate=runpy.run_path(str(root/'linux-kernel/labs/qemu/x86_64/topal_gate.py'))
        static=gate['static_user_fixture']()
    assert sha(static)==component['binaries']['artifact/linked/static-user.elf']['sha256']
    linked=destination/'linked';linked.mkdir(parents=True)
    for name,info in component['snapshots'].items():
        prefix='artifact/linked/'
        if not name.startswith(prefix):continue
        target=linked/name.removeprefix(prefix);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(read(info))
    for name in ['artifact/linked/kernel.elf','artifact/linked/kernel.debug','artifact/linked/initial-user.elf']:
        data=read(projection[name]);assert sha(data)==component['binaries'][name]['sha256']
        (linked/Path(name).name).write_bytes(data)
    (linked/'static-user.elf').write_bytes(static)
    provenance=json.loads((linked/'provenance.json').read_text())
    for output in provenance['outputs']:assert sha((linked/output['identity']).read_bytes())==output['sha256'],output['identity']
    command=['cargo','run','--quiet','--locked','-j1','-p','topal-kernel-toolchain-gate-builder','--',str(linked),str(destination/'boot')]
    print('+',' '.join(command),flush=True)
    subprocess.run(command,cwd=root,env=dict(os.environ,CARGO_BUILD_JOBS='1'),check=True)
    for name in ['bzImage','initramfs.cpio']:
        raw=(destination/'boot'/name).read_bytes();original=component['binaries']['artifact/boot/'+name]
        assert sha(raw)==original['sha256'] and len(raw)==original['bytes'],name
    assert (destination/'boot/boot-provenance.json').read_bytes()==read(component['snapshots']['artifact/boot/boot-provenance.json'])
    print('PRODUCTION_ARTIFACT_RECONSTRUCTION_PASS',args.kind,sha((destination/'boot/bzImage').read_bytes()),flush=True)


if __name__=='__main__':main()
