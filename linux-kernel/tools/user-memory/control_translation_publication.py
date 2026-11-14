"""Reject altered saved local publication records without executing guest code."""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from readback_translation_publication import readback
from translation_publication import slot


def controls(build,output):
    build=Path(build)
    output=Path(output)
    output.mkdir(parents=True,exist_ok=False)
    raw=(build/'source-user-extents/observations.json').read_bytes()
    observation=json.loads(raw)
    baseline=readback(build)
    assert baseline['passed']
    original=observation['translation_publication_observation']
    pub=next(i for i,row in enumerate(original['records']) if max(row['old_count'],row['new_count'])>9)
    retired=next(i for i,row in enumerate(original['requests']) if row['erase'] is not None)
    def leaf(native):
        row=native['records'][pub]
        table,index=slot(row['before'],max(row['old_count'],row['new_count'])-1)
        row['stored']['tables'][table][index]^=4096
    def unrelated(native):
        row=native['records'][pub]
        table=str(row['before']['context']['root'])
        row['stored']['tables'][table][32]=7
    changes=[
        ('wrong-published-tail',leaf),
        ('wrong-invalidation-address',lambda n:n['records'][pub]['invalidations'][0].update(address=0)),
        ('missing-final-invalidation',lambda n:n['records'][pub]['invalidations'].pop()),
        ('duplicated-invalidation-index',lambda n:n['records'][pub]['invalidations'][1].update(index=0)),
        ('different-invalidation-cpu',lambda n:n['records'][pub]['invalidations'][0].update(thread=[9,9,9])),
        ('unsupported-pcid-mode',lambda n:n['records'][pub]['stored'].update(cr4=0x20620)),
        ('missing-final-barrier',lambda n:n['records'][pub].update(final_barrier_observed=False)),
        ('missing-publish-barrier',lambda n:n['records'][pub].update(published_before_invalidations=False)),
        ('wrong-publisher-return-stack',lambda n:n['records'][pub].update(return_stack=0)),
        ('missing-request-publication',lambda n:n['requests'][retired].update(publication_index=None)),
        ('missing-old-erase',lambda n:n['requests'][retired].update(erase=None)),
        ('old-backing-still-pinned',lambda n:n['requests'][retired]['erase']['old_registry'].__setitem__(6,1)),
        ('old-backing-not-erased',lambda n:n['requests'][retired]['release'].update(erased_sha256='0'*64)),
        ('missing-old-release',lambda n:n['requests'][retired].update(release=None)),
        ('wrong-invalidation-site',lambda n:n['profile']['sites'].update(invalidated=n['profile']['sites']['invalidated']+1)),
        ('unrelated-mapping-change',unrelated),
        ('missing-terminal-root-switch',lambda n:n['root_switches'].clear()),
        ('pcid-enabled-at-root-handoff',lambda n:n['root_switches'][0]['after'].update(cr4=0x20620)),
    ]
    rows=[]
    for name,change in changes:
        value=copy.deepcopy(observation)
        change(value['translation_publication_observation'])
        changed=json.dumps(value,sort_keys=True,separators=(',',':')).encode()
        assert changed!=raw
        try:readback(build,changed)
        except AssertionError as error:rows.append(dict(name=name,rejected=True,assertion=str(error)))
        else:raise AssertionError('altered saved record accepted: '+name)
    assert len(rows)==18
    digest=lambda data:hashlib.sha256(data).hexdigest()
    report=dict(passed=True,observation_sha256=digest(raw),runner_sha256=digest(Path(__file__).read_bytes()),
        controls=rows,scope='Eighteen offline JSON-only publication/order/retirement/root-handoff/layout controls; no changed ELF, guest register, memory or program execution and no native refusal coverage')
    (output/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=controls(args.build,args.output)
    print(json.dumps(dict(passed=report['passed'],controls=len(report['controls']))))
