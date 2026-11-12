"""Observe MM boundaries alongside the unchanged ordinary syscall observers."""
from process_mm_process_inspect import PROGRAM as PROCESS_PROGRAM

# Reuse the exact model, capture, checked-breakpoint and gateway definitions.
# Their separate namespace keeps MM pending state distinct from user/syscall
# observers. No specialized MM application or terminal-case expectations run.
_BOUNDARY='class CaseReturned(Checked):'
assert PROCESS_PROGRAM.count(_BOUNDARY)==1
BOUNDARY_PROGRAM=PROCESS_PROGRAM.split(_BOUNDARY)[0]

PROGRAM=r'''
process_regression_namespace=None
process_regression_current=None

def install_process_regression_observation():
    global process_regression_namespace
    if process_regression_profile is None:return
    profile=process_regression_profile
    assert profile['process_memory'] is True
    ns=dict(component=component,token_bytes=token_bytes,kernel_spans=profile['kernel_spans'],
            observation_path=observation_path.rsplit('/',1)[0]+'/process-mm-observations.json')
    exec(process_regression_boundary_program,ns)
    process_regression_namespace=ns
    def complete_save():
        import os
        evidence=ns['observed']
        evidence.update(pending=ns['pending'],active_case=ns['case'],refusal_pending=ns['refusal_attempt'])
        path=ns['observation_path']
        with open(path+'.tmp','w') as stream:stream.write(json.dumps(evidence,separators=(',',':')))
        os.replace(path+'.tmp',path)
    def checkpoint():
        # Keep complete checkpoints and every failure, without serializing the
        # entire growing history after each query in a long interface fixture.
        if ns['observed'].get('failure') or len(ns['observed']['events'])%64==0:complete_save()
    ns['save']=checkpoint
    ns['complete_save']=complete_save
    class Returned(ns['Gateway']):
        def observe(self):
            result=super().observe()
            self.enabled=False
            return result
    returned=Returned('topal_x86_frame_gateway_return_reclaimed_v1',False)
    returned.enabled=False
    class Entry(ns['Gateway']):
        def observe(self):
            result=super().observe()
            # Non-MM operations have no pending MM observation. Their common
            # gateway return need not stop the guest a second time. Every MM
            # entry still enables the exact original return/ABI observation.
            returned.enabled=ns['pending'] is not None
            return result
    Entry('topal_x86_frame_gateway_v1',True)

def process_regression_case_entered():
    global process_regression_current
    if process_regression_namespace is None:return
    ns=process_regression_namespace
    assert process_regression_current is None and ns['pending'] is None
    value=ns['capture']();ns['model']['snapshot'](value)
    assert value['cs']&3==3 and value['ids'][0]==1 and value['ids'][3:6]==[0]*3
    process_regression_current=dict(before=value)

def process_regression_case_returned():
    global process_regression_current
    if process_regression_namespace is None:return
    ns=process_regression_namespace
    assert process_regression_current is not None and ns['pending'] is None
    value=ns['capture']();ns['model']['snapshot'](value)
    before=process_regression_current['before']
    assert value['cs']&3==3 and value['ids']==before['ids']
    assert value['state'][:3]==before['state'][:3] and value['active']['bytes']==before['active']['bytes']
    process_regression_current['after']=value
    ns['observed']['cases'].append(process_regression_current)
    process_regression_current=None

def process_regression_observation_complete():
    if process_regression_namespace is None:return
    ns=process_regression_namespace
    assert process_regression_current is None and ns['pending'] is None
    assert 'failure' not in ns['observed'] and len(ns['observed']['cases'])==4
    assert words(address('topal_x86_frame_mm_tokens_v1'),6)==[0]*6
    assert words(address('topal_x86_frame_mm_state_v1'),4)==[0]*4
    final={}
    for suffix,count in [('ids',7),('inactive_token',3),('inactive_state',4),('prepared_token',3),
                         ('prepared_state',7),('prepared_owner',3),('transition',3)]:
        actual=words(address('topal_x86_process_mm_'+suffix+'_v1'),count)
        assert actual==[0]*count
        final[suffix]=actual
    ns['observed'].update(passed=True,final=final,mode=component['mode'])
    ns['complete_save']()
    observed['process_memory_regression']=dict(passed=True,mode=component['mode'],
        cases=len(ns['observed']['cases']),boundaries=len(ns['observed']['events']),
        kernel_sha256=process_regression_profile['kernel_sha256'])
'''
