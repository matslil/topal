"""Observe selected MM publications and heap retirement without changing guests.

TOPAL-SYSTEMS-PROCESS-MM-001; TOPAL-SYSTEMS-USER-EXTENT-001.
"""
from pathlib import Path

ROOT=Path(__file__).resolve().parents[3]
MODEL=(ROOT/'tools/process-memory/publication.py').read_text()
TABLE_MODEL=(ROOT/'tools/user-memory/translation_publication.py').read_text()
TRACE_MODEL=(ROOT/'tools/user-memory/qemu_publication_trace.py').read_text()

CHECKPOINT_PROGRAM=r'''
def complete_save():
    import os
    observed.update(pending=pending,active_case=case,refusal_pending=refusal_attempt)
    with open(observation_path+'.tmp','w') as stream:
        stream.write(json.dumps(observed,separators=(',',':')))
    os.replace(observation_path+'.tmp',observation_path)

def save():
    # Every in-memory record and every native assertion is retained. Avoid
    # repeatedly encoding the complete growing history on ordinary queries.
    if observed.get('passed') or observed.get('failure') or len(observed['events'])%64==0:
        complete_save()
'''

PROGRAM=("publication_tables={}\nexec("+repr(TABLE_MODEL)+",publication_tables)\n"
    +"publication_model=dict(owner=model['owner'],require=model['require'])\n"
    +"publication_model.update({key:publication_tables[key] for key in ['HEAP','LEAF','PAGE','mode','slot']})\n"
    +"exec("+repr(MODEL)+",publication_model)\n"
    +"publication_trace={}\nexec("+repr(TRACE_MODEL)+",publication_trace)\n"+CHECKPOINT_PROGRAM+r'''
from pathlib import Path
publication_active=None
publication_retiring=None
publication_switch=None
publication_returns={}
publication_gateway_return=None
publication_data=dict(passed=False,profile=publication_profile,records=[],root_switches=[])
observed['process_publication']=publication_data

def publication_allocation(token,allocator):
    if token==[0]*3:return dict(token=token,record=[],cells=[],bytes='')
    assert token[0]==1 and 0<token[1]<allocator[2] and token[2]<allocator[3]
    record=words(allocator[6]+token[2]*64,8)
    assert record[:2]==token[:2] and record[7]==1 and 0<record[3]<=allocator[3]
    assert 0<=record[2]<record[2]+record[3]<=allocator[3]
    cells=words(allocator[4]+128+record[2]*24,record[3]*3)
    return dict(token=token,record=record,cells=cells,bytes='')

def publication_capture():
    # Read each shared metadata block once at this debugger stop. These are
    # fresh reads on every capture, never cached owner/pin/table observations.
    header=words(context,16);root,low=header[3],header[14]
    token=address('topal_x86_frame_mm_tokens_v1')
    roots=address('topal_x86_frame_root_tokens_v1')
    allocator=words(address('topal_x86_frame_context_v1'),16)
    tokens=words(token,6);root_tokens=words(roots,token_bytes//8)
    return dict(allocator=allocator,
        context=dict(address=context,state=header[0],root=root,low=low,
                     bootstrap=header[2],heap=header[15]),
        state=words(address('topal_x86_frame_mm_state_v1'),4),tokens=tokens,
        ids=words(address('topal_x86_process_mm_ids_v1'),7),epoch=words(address('topal_x86_frame_mm_epoch_v1'),2),
        inactive_state=words(address('topal_x86_process_mm_inactive_state_v1'),4),
        prepared_state=words(address('topal_x86_process_mm_prepared_state_v1'),7),
        prepared_owner=words(address('topal_x86_process_mm_prepared_owner_v1'),3),
        transition=words(address('topal_x86_process_mm_transition_v1'),3),
        active=publication_allocation(tokens[:3],allocator),candidate=publication_allocation(tokens[3:],allocator),
        inactive=publication_allocation(words(address('topal_x86_process_mm_inactive_token_v1'),3),allocator),
        prepared=publication_allocation(words(address('topal_x86_process_mm_prepared_token_v1'),3),allocator),
        root_owners=[publication_allocation(root_tokens[i:i+3],allocator)
            for i in range(0,len(root_tokens),3) if root_tokens[i:i+3]!=[0]*3],
        tables={str(at):words(at,512) for at in [root,root+4096,root+8192,root+12288,root+16384,low]},
        kernel_spans=kernel_spans,thread=list(gdb.selected_thread().ptid),
        cr0=reg('cr0'),cr3=reg('cr3'),cr4=reg('cr4'),cs=reg('cs'),flags=reg('eflags'))

class PublicationChecked(Checked):
    def stop(self):
        try:return self.observe()
        except Exception:
            traceback.print_exc()
            observed['passed']=False
            publication_data.update(failure=traceback.format_exc(),active=publication_active,
                retiring=publication_retiring,switch=publication_switch)
            observed['failure']=traceback.format_exc()
            save()
            return True

class PublicationBoundary(PublicationChecked):
    def __init__(self,kind,site):
        self.kind=kind
        super().__init__('*'+hex(site),internal=True)
    def observe(self):
        global publication_active
        assert pending is not None and publication_retiring is None
        if self.kind=='entry':
            assert publication_active is None
            operation=pending['operation']
            assert operation in [5,6,7,10,11,12]
            detached=publication_data['root_switches'][-1] if operation==6 and publication_data['root_switches'] else None
            assert (operation==6)==(detached is not None)
            publication_active=dict(operation=operation,old_count=reg('rdi'),new_count=reg('rsi'),new_base=reg('rdx'),
                before=publication_capture(),detachment=detached,entry_stack=reg('rsp'),entry_return_address=word(reg('rsp')),
                request_sp=pending['sp'],invalidations=[],published_before_invalidations=False,final_barrier_observed=False,
                trace_before={path.name:path.stat().st_size for path in Path(observation_path).parent.glob('cpu-*.log')})
        elif self.kind=='stored':
            assert publication_active is not None and not publication_active['published_before_invalidations']
            publication_active.update(stored=publication_capture(),published_before_invalidations=True)
        else:
            assert self.kind=='returning' and publication_active is not None
            row=publication_active
            row.update(after=publication_capture(),return_stack=reg('rsp'),return_address=word(reg('rsp')),
                final_barrier_observed=True,trace_slices=[],cancelled_post_states=[])
            for path in sorted(Path(observation_path).parent.glob('cpu-*.log')):
                start=row['trace_before'].get(path.name,0);raw=path.read_bytes()[start:]
                events=publication_trace['parse'](raw,publication_profile['sites']['invalidated'])
                row['trace_slices'].append(dict(file=path.name,start=start,size=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
                for event in events:
                    assert event['cpu']+1==row['before']['thread'][1] and row['before']['thread'][0]==1
                    assert (event['cr0'],event['cr3'],event['cr4'])==tuple(row['before'][key] for key in ['cr0','cr3','cr4'])
                    assert event['efer']&0xd00==0xd00 and event['flags']&0x600==0
                states,cancelled=publication_trace['completed'](events)
                row['invalidations'].extend(dict(event,thread=row['before']['thread']) for event in states)
                row['cancelled_post_states'].extend(cancelled)
            row['comparison']=publication_model['check'](row)
            pending['publication_indices'].append(len(publication_data['records']))
            publication_data['records'].append(row)
            publication_active=None
        return False

class PublicationRootSwitch(PublicationChecked):
    def __init__(self,kind,site):
        self.kind=kind
        super().__init__('*'+hex(site),internal=True)
    def observe(self):
        global publication_switch
        assert pending is publication_active is publication_retiring is None
        if self.kind=='before':
            assert publication_switch is None
            before=publication_capture()
            assert reg('rax')==before['context']['bootstrap']
            publication_switch=dict(before=before,before_site=reg('rip'),source_register=reg('rax'))
        else:
            assert self.kind=='after' and publication_switch is not None
            row=publication_switch;row.update(after=publication_capture(),after_site=reg('rip'))
            row['comparison']=publication_tables['check_switch'](row)
            publication_data['root_switches'].append(row);publication_switch=None
        return False

def publication_return(kind,site):
    key=(kind,site)
    if key not in publication_returns:publication_returns[key]=PublicationReturn(kind,site)
    point=publication_returns[key];assert not point.enabled
    point.enabled=True

class PublicationUnpin(PublicationChecked):
    def observe(self):
        global publication_retiring
        pointers={address('topal_x86_frame_mm_tokens_v1'):'active',
            address('topal_x86_process_mm_inactive_token_v1'):'inactive'}
        pointer=reg('rsi')
        if reg('rdx')&1 or pointer not in pointers:return False
        assert pending is not None and publication_active is publication_retiring is None
        assert reg('rdi')==address('topal_x86_frame_context_v1')
        before=publication_capture();role=pointers[pointer];owned=before[role]
        token=owned['token'];assert token!=[0]*3 and owned['record'][6:]==[1,1]
        indices=[i for i,row in enumerate(publication_data['records']) if row['comparison']['source_token']==token]
        assert indices
        publication_retiring=dict(operation=pending['operation'],bank=role,before=before,owner=owned,
            token_pointer=pointer,publication_index=indices[-1],detachment=(publication_data['root_switches'][-1]
            if pending['operation']==6 else None),unpin_succeeded=False,erase=None,release=None,
            release_succeeded=False,publication_count=len(publication_data['records']),
            unpin_stack=reg('rsp'),unpin_return=word(reg('rsp')))
        publication_return('unpin',word(reg('rsp')))
        return False

class PublicationErase(PublicationChecked):
    def observe(self):
        if publication_retiring is None:return False
        row=publication_retiring
        assert row['unpin_succeeded'] and row['erase'] is row['release'] is None
        assert publication_active is None and len(publication_data['records'])==row['publication_count']
        row['erase']=dict(base=reg('rdi'),count=reg('rsi'))
        assert row['erase']==dict(base=row['owner']['cells'][2]*4096,count=row['owner']['record'][3])
        return False

class PublicationRelease(PublicationChecked):
    def observe(self):
        if publication_retiring is None:return False
        row=publication_retiring
        assert row['unpin_succeeded'] and row['erase'] is not None and row['release'] is None
        assert publication_active is None and len(publication_data['records'])==row['publication_count']
        assert reg('rdi')==address('topal_x86_frame_context_v1') and reg('rsi')==row['token_pointer']
        base,count=row['erase']['base'],row['erase']['count'];raw=memory(base,count*4096)
        assert raw==bytes(len(raw))
        row['release']=dict(base=base,count=count,erased_bytes=len(raw),erased_sha256=hashlib.sha256(raw).hexdigest())
        row.update(release_stack=reg('rsp'),release_return=word(reg('rsp')))
        publication_return('release',word(reg('rsp')))
        return False

class PublicationReturn(PublicationChecked):
    def __init__(self,kind,site):
        self.kind=kind
        super().__init__('*'+hex(site),internal=True);self.enabled=False
    def observe(self):
        global publication_retiring
        self.enabled=False;row=publication_retiring
        assert row is not None and reg('rax')&1
        assert reg('rsp')==row[self.kind+'_stack']+8 and reg('rip')==row[self.kind+'_return']
        assert publication_active is None and len(publication_data['records'])==row['publication_count']
        if self.kind=='unpin':
            row.update(unpin_succeeded=True,unpinned=allocation(row['token_pointer'],False))
        else:
            assert self.kind=='release' and row['release'] is not None
            old=row['owner'];allocator=words(address('topal_x86_frame_context_v1'),16)
            row.update(release_succeeded=True,released=dict(token=words(row['token_pointer'],3),
                record=words(allocator[6]+old['token'][2]*64,8),
                cells=words(allocator[4]+128+old['record'][2]*24,old['record'][3]*3),
                erased_sha256=hashlib.sha256(memory(row['erase']['base'],row['erase']['count']*4096)).hexdigest()))
            row['comparison']=publication_model['retirement'](row,publication_data['records'])
            pending['publication_retirements'].append(row);publication_retiring=None
        return False

class Gateway(Gateway):
    def __init__(self,symbol,enter):
        global publication_gateway_return
        super().__init__(symbol,enter)
        if not enter:
            assert publication_gateway_return is None
            publication_gateway_return=self;self.enabled=False
    def observe(self):
        count=len(observed['events']);result=super().observe()
        assert publication_gateway_return is not None
        if self.enter:publication_gateway_return.enabled=pending is not None
        else:self.enabled=False
        if self.enter and pending is not None:
            assert publication_active is publication_retiring is None
            pending.update(publication_indices=[],publication_retirements=[])
        elif not self.enter and len(observed['events'])>count:
            row=observed['events'][-1]
            assert publication_active is publication_retiring is None
            assert {tuple(r['token']) for r in row['retired']}=={
                tuple(r['owner']['token']) for r in row['publication_retirements']}
            operation=row['operation'];old=row['before']['state'][2];new=row['after']['state'][2]
            expected=1 if operation in [10,11,12] else int(old!=new) if operation==5 else int(old>0) if operation in [6,7] else 0
            assert len(row['publication_indices'])==expected
        return result

class Complete(Complete):
    def observe(self):
        assert publication_active is publication_retiring is publication_switch is None
        assert publication_gateway_return is not None and not publication_gateway_return.enabled
        assert all(not point.enabled for point in publication_returns.values())
        assert publication_data['records'] and publication_data['root_switches']
        publication_data['passed']=True
        return super().observe()

def install_process_publication():
    for kind,site in publication_profile['sites'].items():
        if kind!='invalidated':PublicationBoundary(kind,site)
    for kind,site in publication_profile['root_switch_sites'].items():PublicationRootSwitch(kind,site)
    PublicationUnpin('*'+hex(publication_profile['pin_entry']),internal=True)
    PublicationErase('*'+hex(publication_profile['erase_entry']),internal=True)
    PublicationRelease('*'+hex(publication_profile['release_entry']),internal=True)
''')


def augment(original):
    marker="try:\n    Gateway('topal_x86_frame_gateway_v1',True)"
    assert original.count(marker)==1
    return original.replace(marker,PROGRAM+"\ntry:\n    install_process_publication()\n    Gateway('topal_x86_frame_gateway_v1',True)")
