"""Read-only image-leaf publication observations during ordinary RELRO/exec."""
from pathlib import Path

MODEL=(Path(__file__).resolve().parents[3]/'tools/user-memory/image_publication.py').read_text()
PROGRAM='image_publication_model={}\nexec('+repr(MODEL)+',image_publication_model)\n'+r'''
image_publication_active=None
image_publication_records=[]
image_publication_entries={}

def image_publication_capture():
    snapshot=owner_capture()
    snapshot['image_generation']=word(context+896)
    return snapshot

def image_publication_progress():
    with open(observation_path+'.image-progress.json','w') as handle:
        json.dump(dict(passed=False,full_guest_retirement_complete=False,
            records=image_publication_records),handle,indent=2)

class ImagePublication(Checked):
    def __init__(self,kind,site_kind,site):
        self.kind,self.site_kind=kind,site_kind
        super().__init__('*'+hex(site),internal=True)

    def observe(self):
        global image_publication_active
        profile=image_publication_profile['profiles'][self.kind]
        assert reg('rip')==profile['sites'][self.site_kind]
        assert translation_active is translation_request is None
        if self.site_kind=='entry':
            assert image_publication_active is None
            row=dict(kind=self.kind,before=image_publication_capture(),thread_before=list(gdb.selected_thread().ptid),
                invalidations=[],cancelled_post_states=[],publication_barrier_observed=False,
                final_barrier_observed=False,trace_before=translation_trace_positions())
            if self.kind=='relro':
                assert reg('rcx')==8 and reg('r8')==row['before']['context']['low']+485*8
            image_publication_active=row
            image_publication_model['owners'](row['before'],owner_profile['bindings'],owner_profile['kernel_spans'],self.kind)
            # RELRO's entry is also the store-loop head. Observe one complete
            # publication, then re-arm only after its final invalidation.
            self.enabled=False
        elif self.site_kind=='stored':
            row=image_publication_active
            assert row is not None and row['kind']==self.kind and not row['publication_barrier_observed']
            row.update(stored=image_publication_capture(),thread_stored=list(gdb.selected_thread().ptid),publication_barrier_observed=True)
            image_publication_model['published'](row,owner_profile['bindings'],owner_profile['kernel_spans'])
        else:
            assert self.site_kind=='finished'
            row=image_publication_active
            assert row is not None and row['kind']==self.kind and reg('rcx')==0
            row.update(after=image_publication_capture(),thread_after=list(gdb.selected_thread().ptid),
                final_barrier_observed=profile['final_barrier'],trace_slices=[])
            for path in sorted(Path(observation_path).parent.glob('cpu-*.log')):
                start=row['trace_before'].get(path.name,0)
                raw=path.read_bytes()[start:]
                events=translation_trace_model['parse'](raw,profile['sites']['invalidated'])
                row['trace_slices'].append(dict(file=path.name,start=start,size=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
                states,cancelled=translation_trace_model['completed'](events)
                row['invalidations'].extend(states)
                row['cancelled_post_states'].extend(cancelled)
            row['comparison']=image_publication_model['check'](row,owner_profile['bindings'],owner_profile['kernel_spans'])
            image_publication_records.append(row)
            image_publication_active=None
            image_publication_entries[self.kind].enabled=True
            image_publication_progress()
        return False

def install_image_publication_observation():
    if image_publication_profile is None:return
    assert owner_profile is not None and component['mode']=='metadata'
    for kind,profile in image_publication_profile['profiles'].items():
        for site_kind,site in profile['sites'].items():
            if site_kind!='invalidated':
                point=ImagePublication(kind,site_kind,site)
                if site_kind=='entry':image_publication_entries[kind]=point

def image_publication_observation_complete():
    if image_publication_profile is None:return
    assert image_publication_active is None
    assert len(image_publication_entries)==2 and all(point.enabled for point in image_publication_entries.values())
    assert {r['kind'] for r in image_publication_records}=={'relro','exec'}
    observed['image_publication_observation']=dict(passed=True,records=image_publication_records,
        profile=image_publication_profile,model_sha256=hashlib.sha256(image_publication_model_source.encode()).hexdigest(),
        trace_model_sha256=hashlib.sha256(translation_trace_model_source.encode()).hexdigest(),
        scope='Selected ordinary RELRO and exec image-leaf publication, actual owner boundaries and local invalidation order; not all mapping editors, continuous mode, AP/DMA/asynchronous exclusion or production qualification')
'''
