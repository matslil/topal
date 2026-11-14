"""Reconstruct selected image publication checks from saved native records/logs."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re

from image_publication import check
from inspect_call_inventory import Elf,digest
from inspect_image_publication import inspect
from qemu_publication_trace import parse,completed


def readback(build,observation=None):
    build=Path(build)
    work=build/'source-user-extents'
    raw=(work/'observations.json').read_bytes() if observation is None else observation
    observed=json.loads(raw)
    assert observed['passed'] and len(observed['executions'])==4
    native=observed['image_publication_observation']
    assert native['passed']
    profile=inspect(build)
    assert profile==native['profile'],'image publication layout/source binding changed'
    owner=observed['owner_boundary_observation']
    assert owner['passed'] and owner['profile']['kernel_sha256']==profile['kernel_sha256']
    bindings,spans=owner['profile']['bindings'],owner['profile']['kernel_spans']
    assert bindings==json.loads((build/'user-owners.json').read_bytes())
    elf=Elf(build/'kernel.elf')
    context=elf.symbols['topal_bootstrap_storage']['value']+512
    model=Path(__file__).with_name('image_publication.py').read_bytes()
    trace_model=Path(__file__).with_name('qemu_publication_trace.py').read_bytes()
    assert native['model_sha256']==digest(model) and native['trace_model_sha256']==digest(trace_model)
    gdb=(work/'inspect.gdb').read_text()
    assert 'exec('+repr(model.decode())+',image_publication_model)' in gdb
    assert "snapshot['image_generation']=word(context+896)" in gdb
    records=native['records']
    assert records
    pages=cancelled_total=0
    coverage={}
    for row in records:
        for boundary in ['before','stored','after']:
            snapshot=row[boundary]
            assert snapshot['context']['address']==context
            assert snapshot['allocator'][4]==elf.symbols['topal_x86_frame_table_v1']['value']
            assert snapshot['allocator'][6]==elf.symbols['topal_x86_frame_registry_v1']['value']
        assert row['kind'] in profile['profiles']
        site=profile['profiles'][row['kind']]['sites']['invalidated']
        states=[]
        cancelled=[]
        for segment in row['trace_slices']:
            name=segment['file']
            assert Path(name).name==name and name.startswith('cpu-') and name.endswith('.log')
            data=(work/name).read_bytes()
            start,size=segment['start'],segment['size']
            assert type(start) is int and type(size) is int and 0<=start<=start+size<=len(data)
            chunk=data[start:start+size]
            assert digest(chunk)==segment['sha256']
            events=parse(chunk,site)
            assert sum(line.startswith(b'Trace ') for line in chunk.splitlines())==len(events),'unparsed CPU header'
            completed_states,retries=completed(events)
            states.extend(completed_states)
            cancelled.extend(retries)
            coverage.setdefault(name,[]).append((start,start+size,site))
        assert states==row['invalidations'] and cancelled==row['cancelled_post_states']
        result=check(row,bindings,spans)
        assert result==row['comparison']
        pages+=result['pages_invalidated']
        cancelled_total+=len(cancelled)
    counts=Counter(row['kind'] for row in records)
    assert counts['relro']==len(observed['executions']) and counts['exec']==counts['relro']-1
    # The complete ordinary cohort replaces each RELRO image except its final
    # image, then retires it. Do not infer a missing final restore operation.
    assert [row['kind'] for row in records]==['relro','exec']*counts['exec']+['relro']
    image_sites={row['sites']['invalidated'] for row in profile['profiles'].values()}
    raw_logs={}
    for path in sorted(work.glob('cpu-*.log')):
        data=path.read_bytes()
        headers=list(re.finditer(rb'^Trace ([0-9]+): (0x[0-9a-f]+) \[([0-9a-f]+)/([0-9a-f]+)/[^\]]+\][^\n]*\n',data,re.M))
        assert len(headers)==sum(line.startswith(b'Trace ') for line in data.splitlines()),'unparsed full-log CPU header'
        selected=0
        for header in headers:
            site=int(header[4],16)
            if site not in image_sites:continue
            matches=[(start,end,pc) for start,end,pc in coverage.get(path.name,[]) if start<=header.start()<end]
            assert len(matches)==1 and matches[0][2]==site,'missing, reused or mismatched image trace interval'
            selected+=1
        raw_logs[path.name]=dict(sha256=digest(data),bytes=len(data),selected_image_headers=selected)
    assert sum(row['selected_image_headers'] for row in raw_logs.values())==pages+cancelled_total
    return dict(passed=True,kernel_sha256=profile['kernel_sha256'],observation_sha256=digest(raw),
        model_sha256=digest(model),trace_model_sha256=digest(trace_model),
        reader_sha256=digest(Path(__file__).read_bytes()),layout=profile['profiles'],
        publication_records=len(records),relro_publications=counts['relro'],exec_publications=counts['exec'],
        post_invlpg_states=pages,cancelled_block_attempts=cancelled_total,
        selected_owner_boundaries=len(records)*3,
        raw_logs=raw_logs,all_selected_image_headers_covered_once=True,
        scope='Selected complete ordinary image RELRO/exec publication records, exact layout/owner identity boundaries and recreated raw post-INVLPG logs. Not every mapping editor, continuous paging mode, AP/DMA/asynchronous exclusion, general stable lifetime or production qualification')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    report=readback(args.build)
    args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
