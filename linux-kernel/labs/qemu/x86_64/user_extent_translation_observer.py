"""Read-only observations of local heap publication and invalidation retirement."""
from pathlib import Path

MODEL=(Path(__file__).resolve().parents[3]/'tools/user-memory/translation_publication.py').read_text()
TRACE_MODEL=(Path(__file__).resolve().parents[3]/'tools/user-memory/qemu_publication_trace.py').read_text()
PROGRAM='translation_model={}\nexec('+repr(MODEL)+',translation_model)\n'+\
    'translation_trace_model={}\nexec('+repr(TRACE_MODEL)+',translation_trace_model)\n'+r'''
from pathlib import Path
translation_active=None
translation_request=None
translation_records=[]
translation_requests=[]
translation_return_points={}
translation_release_point=None
translation_root_switches=[]
translation_root_switch_pending=None

def translation_trace_positions():
    return {path.name:path.stat().st_size for path in Path(observation_path).parent.glob('cpu-*.log')}

def translation_progress():
    with open(observation_path+'.translation-progress.json','w') as handle:
        json.dump(dict(passed=False,full_guest_retirement_complete=False,records=translation_records,
            requests=translation_requests),handle,indent=2)

def translation_capture():
    root,low=word(context+24),word(context+112)
    assert 0<root<=(1<<30)-5*4096 and root%4096==0
    assert 0<low<=(1<<30)-4096 and low%4096==0
    return dict(context=dict(address=context,root=root,low=low,state=word(context),bootstrap=word(context+16)),state=state(),
        tokens=words(address('topal_x86_frame_mm_tokens_v1'),6),
        tables={str(at):words(at,512) for at in [root,root+4096,root+8192,root+12288,root+16384,low]},
        thread=list(gdb.selected_thread().ptid),cr0=reg('cr0'),cr3=reg('cr3'),cr4=reg('cr4'),
        cs=reg('cs'),flags=reg('eflags'))

class TranslationPublication(Checked):
    def __init__(self,kind,site):
        self.kind=kind
        super().__init__('*'+hex(site),internal=True)

    def observe(self):
        global translation_active
        assert translation_request is not None
        if self.kind=='entry':
            assert translation_active is None and translation_request['publication_index'] is None
            translation_active=dict(old_count=reg('rdi'),new_count=reg('rsi'),new_base=reg('rdx'),
                operation=translation_request['operation'],detachment=translation_request['detachment'],
                before=translation_capture(),entry_stack=reg('rsp'),entry_return_address=word(reg('rsp')),
                invalidations=[],published_before_invalidations=False,final_barrier_observed=False,
                trace_before=translation_trace_positions())
        elif self.kind=='stored':
            row=translation_active
            assert row is not None and not row['published_before_invalidations'] and not row['invalidations']
            row['stored']=translation_capture()
            translation_model['published'](row['before'],row['stored'],row['old_count'],row['new_count'],row['new_base'],row['detachment'])
            row['published_before_invalidations']=True
        else:
            assert self.kind=='returning' and translation_active is not None
            row=translation_active
            row.update(after=translation_capture(),return_stack=reg('rsp'),return_address=word(reg('rsp')),
                final_barrier_observed=True)
            row['trace_slices']=[]
            for path in sorted(Path(observation_path).parent.glob('cpu-*.log')):
                start=row['trace_before'].get(path.name,0)
                raw=path.read_bytes()[start:]
                events=translation_trace_model['parse'](raw,translation_profile['sites']['invalidated'])
                row['trace_slices'].append(dict(file=path.name,start=start,size=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
                states,cancelled=translation_trace_model['completed'](events)
                row.setdefault('cancelled_post_states',[]).extend(cancelled)
                for event in events:
                    assert event['cpu']+1==row['before']['thread'][1] and row['before']['thread'][0]==1
                    assert (event['cr0'],event['cr3'],event['cr4'])==tuple(row['before'][k] for k in ['cr0','cr3','cr4'])
                    assert event['efer']&0xd00==0xd00 and event['flags']&0x600==0
                row['invalidations'].extend(dict(event,thread=row['before']['thread']) for event in states)
            row['comparison']=translation_model['check'](row)
            translation_request['publication_index']=len(translation_records)
            translation_records.append(row)
            translation_active=None
            translation_progress()
        return False

class TranslationRequest(Checked):
    def observe(self):
        global translation_request
        assert translation_request is None and translation_active is None
        assert translation_release_point is not None and not translation_release_point.enabled
        translation_release_point.enabled=True
        stack=reg('rsp')
        site=word(stack)
        translation_request=dict(operation=reg('rdi'),request=reg('rsi'),before=translation_capture(),
            entry_stack=stack,entry_return_address=site,publication_index=None,erase=None,release=None,
            detachment=(translation_root_switches[-1] if reg('rdi')==6 and translation_root_switches else None))
        assert translation_request['operation'] in [5,6,7]
        assert (translation_request['operation']==6)==(translation_request['detachment'] is not None)
        translation_model['mode'](translation_request['before'],translation_request['detachment'])
        if site not in translation_return_points:
            translation_return_points[site]=TranslationRequestReturned('*'+hex(site),internal=True)
        translation_return_points[site].enabled=True
        return False

class TranslationRequestReturned(Checked):
    def observe(self):
        global translation_request
        row=translation_request
        assert row is not None and translation_active is None
        assert translation_release_point.enabled
        assert reg('rsp')==row['entry_stack']+8 and reg('rip')==row['entry_return_address']
        row.update(after=translation_capture(),result=reg('rax'))
        translation_model['mode'](row['after'],row['detachment'])
        index=row['publication_index']
        if index is not None:
            publication=translation_records[index]
            assert row['before']['state'][2]==publication['old_count']
            assert row['after']['state'][2]==publication['new_count']
            if publication['old_count']:
                assert row['erase'] is not None and row['release'] is not None
            else:assert row['erase'] is row['release'] is None
        else:
            assert row['before']['state'][2:]==row['after']['state'][2:]
            assert row['erase'] is row['release'] is None
        translation_requests.append(row)
        translation_request=None
        translation_release_point.enabled=False
        self.enabled=False
        translation_progress()
        return False

class TranslationErase(Checked):
    def observe(self):
        if translation_request is None:return False
        row=translation_request
        assert translation_active is None and row['publication_index'] is not None
        assert row['erase'] is None and row['release'] is None
        publication=translation_records[row['publication_index']]
        old=publication['before']['state']
        assert reg('rdi')==old[3] and reg('rsi')==old[2] and old[2]>0
        token=words(address('topal_x86_frame_mm_tokens_v1'),3)
        registry=words(address('topal_x86_frame_registry_v1')+token[2]*64,8)
        assert registry[:2]==token[:2] and registry[3]==old[2] and registry[6:]==[0,1]
        row['erase']=dict(base=reg('rdi'),count=reg('rsi'),old_token=token,old_registry=registry,
            after_publication_index=row['publication_index'],thread=list(gdb.selected_thread().ptid))
        return False

class TranslationRelease(Checked):
    def observe(self):
        if translation_request is None:return False
        row=translation_request
        assert translation_active is None and row['erase'] is not None and row['release'] is None
        assert reg('rsi')==address('topal_x86_frame_mm_tokens_v1')
        erase=row['erase']
        raw=memory(erase['base'],erase['count']*4096)
        assert raw==bytes(len(raw))
        row['release']=dict(base=erase['base'],count=erase['count'],erased_bytes=len(raw),
            erased_sha256=hashlib.sha256(raw).hexdigest(),after_publication_index=row['publication_index'],
            thread=list(gdb.selected_thread().ptid))
        return False

class TranslationRootSwitch(Checked):
    def __init__(self,kind,site):
        self.kind=kind
        super().__init__('*'+hex(site),internal=True)

    def observe(self):
        global translation_root_switch_pending
        assert translation_request is translation_active is None
        if self.kind=='before':
            assert translation_root_switch_pending is None
            before=translation_capture()
            assert reg('rax')==before['context']['bootstrap']
            translation_root_switch_pending=dict(before=before,before_site=reg('rip'),source_register=reg('rax'))
        else:
            assert self.kind=='after' and translation_root_switch_pending is not None
            row=translation_root_switch_pending
            row['after']=translation_capture()
            row['after_site']=reg('rip')
            row['comparison']=translation_model['check_switch'](row)
            translation_root_switches.append(row)
            translation_root_switch_pending=None
        return False

def install_translation_observation():
    global translation_release_point
    if translation_profile is None:return
    assert component['mode']=='metadata'
    for kind,site in translation_profile['root_switch_sites'].items():TranslationRootSwitch(kind,site)
    for kind,site in translation_profile['sites'].items():
        if kind!='invalidated':TranslationPublication(kind,site)
    TranslationRequest('*topal_frame_memory_request_v1',internal=True)
    TranslationErase('*'+hex(translation_profile['erase_entry']),internal=True)
    translation_release_point=TranslationRelease('*topal_frame_allocator_release_v1',internal=True)
    translation_release_point.enabled=False

def translation_observation_complete():
    if translation_profile is None:return
    assert translation_active is translation_request is None
    assert translation_root_switch_pending is None and translation_root_switches
    assert translation_release_point is not None and not translation_release_point.enabled
    assert translation_records and translation_requests
    assert all(not point.enabled for point in translation_return_points.values())
    observed['translation_publication_observation']=dict(passed=True,records=translation_records,
        requests=translation_requests,profile=translation_profile,root_switches=translation_root_switches,
        model_sha256=hashlib.sha256(translation_model_source.encode()).hexdigest(),
        trace_model_sha256=hashlib.sha256(translation_trace_model_source.encode()).hexdigest(),
        scope='Actual local heap publication and post-INVLPG/final-barrier boundaries, plus unpin/erase/release ordering in ordinary requests; not TLB introspection, all CPUs, continuous mode, DMA/async or complete stable lifetime qualification')
'''
