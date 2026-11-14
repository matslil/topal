"""Join saved native local invalidations to heap requests and old-backing release.

TOPAL-SYSTEMS-USER-EXTENT-001. This reader executes no guest code and cannot
establish unobserved CPU/TLB, DMA, asynchronous or general lifetime properties.
"""
import argparse
import hashlib
import json
from pathlib import Path

from inspect_translation_publication import inspect
from translation_publication import check,mode,check_switch
from qemu_publication_trace import parse as parse_trace,completed


def digest(raw):return hashlib.sha256(raw).hexdigest()


def readback(build,observation=None):
    build=Path(build)
    raw=(build/'source-user-extents/observations.json').read_bytes() if observation is None else observation
    observed=json.loads(raw)
    assert observed['passed'] and len(observed['executions'])==4
    native=observed['translation_publication_observation']
    assert native['passed']
    profile=inspect(build)
    assert profile==native['profile'],'native publisher layout/source binding changed'
    model=Path(__file__).with_name('translation_publication.py').read_bytes()
    assert native['model_sha256']==digest(model)
    switches=native['root_switches']
    assert switches,'missing observed terminal root handoff'
    for switch in switches:
        assert switch['comparison']==check_switch(switch)
        assert switch['before_site']==profile['root_switch_sites']['before']
        assert switch['after_site']==profile['root_switch_sites']['after']
        assert switch['source_register']==switch['before']['context']['bootstrap']
    trace_model=Path(__file__).with_name('qemu_publication_trace.py').read_bytes()
    assert native['trace_model_sha256']==digest(trace_model)
    gdb=(build/'source-user-extents/inspect.gdb').read_text()
    assert 'exec('+repr(model.decode())+',translation_model)' in gdb
    records=native['records']
    assert records
    pages=0
    transitions=[]
    for row in records:
        traced=[]
        cancelled=[]
        for segment in row['trace_slices']:
            name=segment['file']
            assert Path(name).name==name and name.startswith('cpu-') and name.endswith('.log')
            data=(build/'source-user-extents'/name).read_bytes()
            start,size=segment['start'],segment['size']
            assert 0<=start<=start+size<=len(data)
            chunk=data[start:start+size]
            assert digest(chunk)==segment['sha256']
            events=parse_trace(chunk,profile['sites']['invalidated'])
            assert sum(line.startswith(b'Trace ') for line in chunk.splitlines())==len(events),'unparsed CPU trace header'
            for event in events:
                assert event['cpu']+1==row['before']['thread'][1] and row['before']['thread'][0]==1
                assert (event['cr0'],event['cr3'],event['cr4'])==tuple(row['before'][k] for k in ['cr0','cr3','cr4'])
                assert event['efer']&0xd00==0xd00 and event['flags']&0x600==0
            states,retries=completed(events)
            cancelled.extend(retries)
            traced.extend(dict(event,thread=row['before']['thread']) for event in states)
        assert traced==row['invalidations'],'QEMU CPU log and invalidation record differ'
        assert cancelled==row['cancelled_post_states'],'cancelled CPU states differ'
        result=check(row)
        assert result==row['comparison']
        pages+=result['pages_invalidated']
        transitions.append([row['old_count'],row['new_count']])
    requests=native['requests']
    assert requests
    used=[]
    releases=0
    erased=0
    for request in requests:
        before,after=request['before'],request['after']
        detachment=request['detachment']
        if request['operation']==6:assert detachment in switches
        else:assert detachment is None
        assert mode(before,detachment)==mode(after,detachment) and before['thread']==after['thread']
        assert before['context']==after['context']
        assert request['operation'] in [5,6,7]
        index=request['publication_index']
        if index is None:
            assert before['state'][2:]==after['state'][2:]
            assert request['erase'] is request['release'] is None
            continue
        assert index==len(used),'missing, duplicated or reordered publication request'
        used.append(index)
        row=records[index]
        assert row['detachment']==detachment and row['operation']==request['operation']
        assert row['before']['thread']==before['thread'] and mode(row['before'],detachment)==mode(before,detachment)
        assert before['state'][2]==row['old_count'] and after['state'][2]==row['new_count']
        assert before['state'][3]==row['before']['state'][3]
        assert after['state'][3]==row['new_base']
        assert before['tokens'][:3]==row['before']['tokens'][:3]
        assert before['tokens'][3:]==after['tokens'][3:]==[0,0,0]
        candidate=row['before']['tokens'][3:]
        assert after['tokens'][:3]==candidate
        if row['new_count']:
            assert candidate[0]==1 and candidate[1]>0 and candidate!=before['tokens'][:3]
        else:assert candidate==[0,0,0]
        # Only hardware status sets may appear after publication before request return.
        assert set(row['after']['tables'])==set(after['tables'])
        for table,words in row['after']['tables'].items():
            assert len(words)==len(after['tables'][table])==512
            for a,b in zip(words,after['tables'][table]):
                assert a&~0x60==b&~0x60 and (a|b)==b,'translation changed after publication'
        erase,release=request['erase'],request['release']
        if not row['old_count']:
            assert erase is release is None
            continue
        assert erase is not None and release is not None,'missing old backing retirement event'
        assert erase['after_publication_index']==release['after_publication_index']==index
        assert erase['thread']==release['thread']==before['thread']
        assert erase['base']==release['base']==before['state'][3]
        assert erase['count']==release['count']==row['old_count']
        token,registry=erase['old_token'],erase['old_registry']
        assert token==before['tokens'][:3]
        assert len(token)==3 and token[0]==1 and token[1]>0 and token[2]>=0
        assert len(registry)==8 and registry[:2]==token[:2] and registry[3]==row['old_count']
        assert registry[6:]==[0,1],'old backing was not unpinned after publication'
        assert release['erased_bytes']==row['old_count']*4096
        assert release['erased_sha256']==digest(bytes(release['erased_bytes'])),'old backing not completely erased before release'
        releases+=1
        erased+=release['erased_bytes']
    assert used==list(range(len(records)))
    # Native evidence must include both allocation and revocation, not only growth.
    assert any(a==0 and b>0 for a,b in transitions)
    assert any(a>b>0 for a,b in transitions) and any(a>0 and b==0 for a,b in transitions)
    assert any(a<9<b for a,b in transitions),'native run did not cross the low/high table split'
    assert any(row['detachment'] is not None for row in records),'missing detached whole-heap retirement'
    cpus={tuple(row['before']['thread']) for row in records}
    return dict(passed=True,kernel_sha256=profile['kernel_sha256'],observation_sha256=digest(raw),
        publisher_function_sha256=profile['function_sha256'],model_sha256=digest(model),
        reader_sha256=digest(Path(__file__).read_bytes()),publication_records=len(records),
        observed_flushing_root_handoffs=len(switches),
        active_publications=sum(row['detachment'] is None for row in records),
        detached_whole_heap_retirements=sum(row['detachment'] is not None for row in records),
        memory_requests=len(requests),post_invlpg_boundaries=pages,old_backing_releases=releases,
        cancelled_block_attempts=sum(len(row['cancelled_post_states']) for row in records),
        old_backing_erased_bytes=erased,transitions=transitions,observed_cpu_threads=[list(c) for c in sorted(cpus)],
        full_changed_page_interval_invalidated=True,old_backing_retired_after_local_publication=True,
        scope='Exact saved local heap publication/invalidation and old unpin/erase/release order in the actual ordinary cohort. No TLB introspection, continuous mode, AP/DMA/async exclusion, all mapping editors, general stable lifetime or production activation')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=readback(args.build)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
