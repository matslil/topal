"""Read-only prepared heap observations; no guest register or memory edits."""
from pathlib import Path

MODEL = (Path(__file__).resolve().parents[3] / 'tools/process-memory/preparation_native.py').read_text()
PROGRAM = 'process_preparation_model={}\nexec(' + repr(MODEL) + ',process_preparation_model)\n' + r'''
process_preparation_active=None
process_preparation_records=[]

def preparation_allocation(symbol,capture_bytes=True):
    token=words(address(symbol) if isinstance(symbol,str) else symbol,3)
    if token==[0,0,0]:return dict(token=token,record=[],cells=[],bytes='')
    allocator=words(address('topal_x86_frame_context_v1'),16)
    assert token[0]==1 and 0<token[1]<allocator[2] and token[2]<allocator[3]
    record=words(allocator[6]+token[2]*64,8)
    assert record[:2]==token[:2] and 0<record[3]<=519 and record[7]==1
    cells=words(allocator[4]+128+record[2]*24,record[3]*3)
    base=cells[2]*4096
    return dict(token=token,record=record,cells=cells,bytes=memory(base,record[3]*4096).hex() if capture_bytes else '')

def preparation_capture():
    root,low=word(context+24),word(context+112)
    allocator=words(address('topal_x86_frame_context_v1'),16)
    return dict(allocator=allocator,state=state(),tokens=words(address('topal_x86_frame_mm_tokens_v1'),6),
        epoch=words(address('topal_x86_frame_mm_epoch_v1'),2),
        recovery=words(address('topal_x86_frame_gateway_recovery_v1'),6),
        prepared_state=words(address('topal_x86_process_mm_prepared_state_v1'),7),
        active=preparation_allocation('topal_x86_frame_mm_tokens_v1'),
        prepared=preparation_allocation('topal_x86_process_mm_prepared_token_v1'),
        static_owners=[preparation_allocation(address('topal_x86_frame_root_tokens_v1')+role['slot']*24,False)
                       for role in process_preparation_profile['bindings']['roles']],
        context=dict(address=context,state=word(context),self=word(context+256),root=root,low=low,heap=word(context+120)),
        context_bytes=memory(context,1024).hex(),reserved_spans=process_preparation_profile['reserved_spans'],
        tables={str(a):words(a,512) for a in [root,root+4096,root+8192,root+12288,root+16384,low]},
        details=words(address('topal_x86_frame_details_v1'),4),
        cr0=reg('cr0'),cr3=reg('cr3'),cr4=reg('cr4'),gs_base=reg('gs_base'),cs=reg('cs'),flags=reg('eflags'))

def preparation_progress():
    with open(observation_path+'.preparation-progress.json','w') as handle:
        json.dump(dict(passed=False,records=process_preparation_records,active=process_preparation_active),handle)

class ProcessPreparation(Checked):
    def __init__(self,kind,symbol):
        self.kind=kind
        super().__init__("*'"+symbol+"'",internal=True)

    def observe(self):
        global process_preparation_active
        if self.kind=='before':
            assert process_preparation_active is None
            process_preparation_active=dict(before=preparation_capture(),ready=None,erased=None,after=None)
            process_preparation_model['snapshot'](process_preparation_active['before'])
        elif self.kind=='ready':
            assert process_preparation_active is not None and process_preparation_active['ready'] is None
            process_preparation_active['ready']=preparation_capture()
        elif self.kind=='erased':
            row=process_preparation_active
            assert row is not None and row['ready'] is not None and row['erased'] is None
            candidate=preparation_allocation('topal_x86_process_mm_prepared_token_v1')
            row['erased']=dict(base=reg('rdi'),count=reg('rsi'),token=candidate['token'],record=candidate['record'],bytes=candidate['bytes'])
        elif self.kind=='unavailable':
            # This cohort must retain an explicit failure rather than silently
            # claim unexecuted successful preparation. Resource misses are a
            # separate required native cohort, not modeled as success here.
            raise AssertionError('ordinary preparation unexpectedly unavailable')
        else:
            row=process_preparation_active
            assert self.kind=='after' and row is not None and row['ready'] is not None
            after=preparation_capture()
            candidate=row['ready']['prepared']
            if candidate['token']!=[0,0,0]:
                allocator=after['allocator']
                after['released_record']=words(allocator[6]+candidate['token'][2]*64,8)
                r=candidate['record']
                after['released_cells']=words(allocator[4]+128+r[2]*24,r[3]*3)
            else:
                after['released_record']=after['released_cells']=[]
            row['after']=after
            row['comparison']=process_preparation_model['check'](row)
            process_preparation_records.append(row)
            process_preparation_active=None
        preparation_progress()
        return False

def install_process_preparation_observation():
    if process_preparation_profile is None:return
    for kind,symbol in [('before','pm.prepare'),('ready','pm.at_prepared'),('erased','pm.at_erased'),
                        ('after','pm.at_released'),('unavailable','pm.at_unavailable')]:
        ProcessPreparation(kind,symbol)

def process_preparation_observation_complete():
    if process_preparation_profile is None:return
    assert process_preparation_active is None and process_preparation_records
    assert any(row['comparison']['pages']>34 for row in process_preparation_records)
    assert any(row['comparison']['pages']==0 for row in process_preparation_records)
    assert words(address('topal_x86_process_mm_prepared_token_v1'),3)==[0]*3
    assert words(address('topal_x86_process_mm_prepared_state_v1'),7)==[0]*7
    observed['process_preparation_observation']=dict(passed=True,records=process_preparation_records,
        model_sha256=hashlib.sha256(process_preparation_model_source.encode()).hexdigest(),
        scope='Real whole-heap prepare/pin/copy/cancel/erase/release in ordinary brk invocations, no fork/switch or production qualification')
'''
