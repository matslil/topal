"""Reject altered saved image-publication records; execute no guest changes."""
import argparse
import copy
import json
from pathlib import Path

from inspect_call_inventory import digest
from readback_image_publication import readback


def controls(build,output):
    build=Path(build)
    raw=(build/'source-user-extents/observations.json').read_bytes()
    baseline=readback(build)
    observed=json.loads(raw)
    def image(value):return value['image_publication_observation']
    def relro(value):return next(row for row in image(value)['records'] if row['kind']=='relro')
    def execute(value):return next(row for row in image(value)['records'] if row['kind']=='exec')
    def leaf(value,field,index,mask):
        row=relro(value)
        row[field]['tables'][str(row['before']['context']['low'])][index]^=mask
    changes=[
        ('missing-complete-generation',lambda v:image(v)['records'].__delitem__(slice(0,2))),
        ('reused-generation-traces',lambda v:image(v)['records'].__setitem__(slice(2,4),copy.deepcopy(image(v)['records'][:2]))),
        ('missing-relro-record',lambda v:image(v)['records'].remove(relro(v))),
        ('missing-exec-record',lambda v:image(v)['records'].remove(execute(v))),
        ('missing-invalidation',lambda v:relro(v)['invalidations'].pop()),
        ('false-publication-barrier',lambda v:relro(v).update(publication_barrier_observed=False)),
        ('false-final-relro-barrier',lambda v:relro(v).update(final_barrier_observed=False)),
        ('invented-final-exec-barrier',lambda v:execute(v).update(final_barrier_observed=True)),
        ('wrong-relro-write-bit',lambda v:leaf(v,'stored',485,2)),
        ('wrong-relro-physical-leaf',lambda v:leaf(v,'stored',485,4096)),
        ('wrong-owner-pin',lambda v:relro(v)['before']['roles'][2]['record'].__setitem__(6,0)),
        ('wrong-syscall-epoch',lambda v:relro(v)['before'].update(epoch=[1,5])),
        ('changed-cr3-before-return',lambda v:relro(v)['after'].update(cr3=0)),
        ('changed-copy-cpu',lambda v:relro(v).update(thread_after=[1,2,0])),
        ('missing-raw-trace',lambda v:relro(v).update(trace_slices=[])),
        ('false-raw-log-hash',lambda v:relro(v)['trace_slices'][0].update(sha256='0'*64)),
    ]
    rows=[]
    for name,change in changes:
        value=copy.deepcopy(observed)
        change(value)
        try:readback(build,json.dumps(value).encode())
        except AssertionError as error:rows.append(dict(name=name,rejected=True,error=str(error)))
        else:raise AssertionError('accepted altered image publication record: '+name)
    report=dict(passed=True,observation_sha256=digest(raw),runner_sha256=digest(Path(__file__).read_bytes()),
        controls=rows,baseline_publications=baseline['publication_records'],
        scope='Sixteen altered saved-JSON controls only; no modified guest state, third-party program or native negative execution')
    Path(output).write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    controls(args.build,args.output)
