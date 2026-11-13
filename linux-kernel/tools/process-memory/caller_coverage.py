"""Combine first-reach samples without turning them into caller qualification.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001.
Argument admission, return, loan lifetime and path feasibility remain separate.
"""
from copy import deepcopy


def require(condition, message):
    if not condition:
        raise ValueError(message)


def combine(inventory, inventory_sha256, kernel_sha256, observations):
    require(inventory['schema']=='topal.owned-extent-call-inventory/1','caller inventory schema')
    calls=inventory['calls'];sites={row['offset']:row for row in calls}
    require(len(calls)==len(sites)>0,'caller duplicate/empty inventory')
    for row in calls:
        require(type(row['offset']) is int and 0<=row['offset']<=inventory['function_bytes']-5,
                'caller inventory offset')
        require(row['target'] in ['runtime-readable-extent','runtime-writable-extent'],
                'caller inventory target')
    seen={};excluded=[];cohorts={}
    require(bool(observations),'caller observations absent')
    for name,observed in observations.items():
        require(observed['passed'] is True and 'failure' not in observed,'caller observation failed')
        coverage=observed.get('extent_call_coverage')
        if coverage is None:
            # Old exec/signal fixtures have no such field. Their success is
            # not a first-reach sample and cannot silently contribute sites.
            excluded.append(name);continue
        require(coverage['observation_passed'] is True,'caller coverage incomplete')
        require(coverage['kernel_sha256']==kernel_sha256
                and coverage['inventory_sha256']==inventory_sha256,'caller native identity')
        require(coverage['total_sites']==len(calls),'caller inventory cardinality')
        records=coverage['reached'];offsets=[row['offset'] for row in records]
        require(len(set(offsets))==len(offsets)==coverage['reached_sites']>0,
                'caller duplicate/empty samples')
        for row in records:
            require(row['offset'] in sites,'caller unknown sample')
            require(row['target']==sites[row['offset']]['target'],'caller sample target')
            require(all(type(row[k]) is int and 0<=row[k]<(1<<64) for k in ['candidate','length']),
                    'caller native argument width')
            require(type(row['phase']) is int and 0<=row['phase']<256,'caller native phase')
            seen.setdefault(row['offset'],[]).append(dict(cohort=name,
                candidate=row['candidate'],length=row['length'],phase=row['phase']))
        if 'unreached' in coverage:
            missing=[row for row in calls if row['offset'] not in offsets]
            require(coverage['unreached']==missing,'caller per-cohort missing closure')
        cohorts[name]=dict(reached=len(offsets))
    missing=[deepcopy(row) for row in calls if row['offset'] not in seen]
    return dict(passed=True,total_sites=len(calls),reached_sites=len(seen),
        missing=missing,all_sites_sampled=not missing,cohorts=cohorts,
        excluded_without_coverage=excluded,
        sites=[dict(site=deepcopy(sites[offset]),samples=seen[offset]) for offset in sorted(seen)],
        kernel_sha256=kernel_sha256,inventory_sha256=inventory_sha256,
        scope='Saved first-reach samples before helper invocation; no new VM, successful admission, return, every argument/path, unreachable-site proof, loan lifetime or production qualification')
