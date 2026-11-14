"""Read-only observations of reached private stack zeroing blocks."""

PROGRAM = r'''
stack_retirement_active = None
stack_retirement_records = {}
stack_retirement_registers = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi',
    'r8','r9','r10','r11','r12','r13','r14','r15']

class StackRetirementStart(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['start']),internal=True)
        self.row=row

    def observe(self):
        global stack_retirement_active
        assert stack_retirement_active is None and reg('cs')&3==0
        assert reg('cr0')&(1<<16) and not reg('eflags')&(1<<9)
        rsp=reg('rsp')
        size=self.row['bytes']
        stack_retirement_active=dict(row=self.row,rsp=rsp,
            registers=[reg(name) for name in stack_retirement_registers],flags=reg('eflags'),
            lower_guard=memory(rsp-size-8,8),upper_guard=memory(rsp,8))
        return False

class StackRetirementEnd(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['end']),internal=True)
        self.row=row

    def observe(self):
        global stack_retirement_active
        before=stack_retirement_active
        assert before is not None and before['row']==self.row
        rsp,size=before['rsp'],self.row['bytes']
        assert [reg(name) for name in stack_retirement_registers]==before['registers'], 'retirement changes GPRs/RSP'
        assert reg('eflags')==before['flags'], 'retirement changes flags'
        assert memory(rsp-size,size)==bytes(size), 'retirement leaves nonzero words'
        assert memory(rsp-size-8,8)==before['lower_guard'] and memory(rsp,8)==before['upper_guard'], 'retirement crosses guards'
        name=self.row['name']
        record=stack_retirement_records.setdefault(name,dict(name=name,start=self.row['start'],
            end=self.row['end'],words=self.row['words'],completed=0,zero_bytes_observed=0))
        record['completed']+=1
        record['zero_bytes_observed']+=size
        stack_retirement_active=None
        return False

def install_stack_retirement_observation():
    audit=extent_inventory.get('stack_retirement_audit')
    if audit is None: return
    assert audit['passed'] and audit['kernel_sha256']==extent_inventory['kernel_sha256']
    assert len(audit['blocks'])==13
    for row in audit['blocks']:
        StackRetirementStart(row)
        StackRetirementEnd(row)

def stack_retirement_observation_complete():
    audit=extent_inventory.get('stack_retirement_audit')
    if audit is None: return
    assert stack_retirement_active is None
    observed['stack_retirement_observation']=dict(passed=True,
        records=[stack_retirement_records[name] for name in sorted(stack_retirement_records)],
        unreached=[row['name'] for row in audit['blocks'] if row['name'] not in stack_retirement_records],
        audit_sha256=extent_inventory['stack_retirement_audit_sha256'],
        scope='Actual reached zeroing blocks: all sixteen GPRs, RSP and flags equal at boundaries, exact retired words zero, adjacent words unchanged. Adjacent-word reads are debugger observations, not extra kernel writes. No proof of ownership/nonalias, continuous lifetime, unreached paths, exception/terminal cleanup or production qualification')
'''
