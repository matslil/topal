"""Read-only selector, admission and scalar staging observations for warnings."""

PROGRAM = r'''
warning_active = None
warning_records = []

def warning_control():
    return dict(cr3=reg('cr3'),cr0=reg('cr0'),cr4=reg('cr4'),
        epoch=memory(address('topal_x86_frame_mm_epoch_v1'),16))

class WarningCalled(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['call_offset']),internal=True)
        self.row=row

    def observe(self):
        global warning_active
        assert warning_active is None and reg('cs')&3==0 and reg('cr0')&(1<<16)
        assert not reg('eflags')&(1<<9)
        selector=word(context+self.row['selector_gs_offset'])
        variants=[r for r in self.row['variants'] if r['selector']==selector]
        assert len(variants)==1
        variant=variants[0]
        assert reg('rdi')==2 and reg('rdx')==reg('r9')==variant['admitted_bytes']
        assert reg('r8')==reg('rsi')==word(context+320)
        warning_active=dict(row=self.row,variant=variant,selector=selector,
            candidate=reg('r8'),size=reg('r9'),rsp=reg('rsp'),control=warning_control(),
            registers=[reg(name) for name in copy_register_names],captured=memory(context+296,120),
            admitted=False,accesses=[])
        return False

class WarningAdmitted(Checked):
    def observe(self):
        global warning_active
        before=warning_active
        assert before is not None and not before['admitted']
        assert reg('rsp')==before['rsp'] and memory(reg('rsp')-8,8)==bytes(8)
        assert [reg(copy_register_names[i]) for i in copy_preserved]==[before['registers'][i] for i in copy_preserved]
        assert memory(context+296,120)==before['captured']
        assert word(context+616)==before['selector'] and warning_control()==before['control']
        if reg('eflags')&1:
            warning_records.append(dict(selector=before['selector'],candidate=before['candidate'],
                size=before['size'],accepted=False,accesses=[]))
            warning_active=None
            return False
        before['admitted']=True
        before['mapping']=copy_range(before['candidate'],before['size'],True,False)
        before['payload']=memory(before['candidate'],before['size'])
        expected={4:b'sh: ',40:b"can't access tty; job control turned off",1:b'\n'}
        assert before['payload']==expected[before['size']]
        before['tables']={p:memory(p,4096) for m in before['mapping'] for p,_,_ in m['path']}
        return False

class WarningRead(Checked):
    def __init__(self,row,variant):
        super().__init__('*'+hex(extent_inventory['address']+row['offset']),internal=True)
        self.row,self.variant=row,variant

    def observe(self):
        before=warning_active
        assert before is not None and before['admitted'] and before['variant']==self.variant
        assert reg('rsi')==before['candidate'] and reg('rsp')==before['rsp']
        assert word(context+616)==before['selector'] and memory(context+296,120)==before['captured']
        offset,width=self.row['candidate_offset'],self.row['width']
        assert 0<=offset and offset+width<=before['size']
        assert len(before['accesses'])<len(self.variant['access_instructions'])
        assert self.row==self.variant['access_instructions'][len(before['accesses'])]
        payload=memory(reg('rsi')+offset,width)
        assert payload==before['payload'][offset:offset+width]
        before['accesses'].append(dict(offset=self.row['offset'],candidate_offset=offset,width=width,
            payload_hex=payload.hex()))
        return False

class WarningStaged(Checked):
    def __init__(self,variant):
        super().__init__('*'+hex(extent_inventory['address']+variant['local_end']),internal=True)
        self.variant=variant

    def observe(self):
        global warning_active
        before=warning_active
        assert before is not None and before['admitted'] and before['variant']==self.variant
        assert len(before['accesses'])==len(self.variant['access_instructions'])
        assert reg('rsi')==before['candidate'] and reg('rsp')==before['rsp']
        assert word(context+616)==before['selector'] and memory(context+296,120)==before['captured']
        assert warning_control()==before['control']
        assert copy_range(before['candidate'],before['size'],True,False)==before['mapping']
        assert all(memory(p,4096)==raw for p,raw in before['tables'].items())
        assert memory(before['candidate'],before['size'])==before['payload']
        staged=b''.join((reg(copy_register_names[8+i])&((1<<(8*row['width']))-1)).to_bytes(row['width'],'little')
            for i,row in enumerate(self.variant['access_instructions']))
        assert staged==before['payload']
        warning_records.append(dict(selector=before['selector'],candidate=before['candidate'],
            size=before['size'],accepted=True,payload_hex=staged.hex(),accesses=before['accesses'],
            helper_registers_preserved=True,captured_arguments_preserved=True,selector_preserved=True,
            complete_register_staging_equal=True,translation_observations_equal=True))
        warning_active=None
        return False

def install_warning_observation():
    audit=extent_inventory.get('warning_input_audit')
    if audit is None: return
    assert audit['passed'] and audit['kernel_sha256']==extent_inventory['kernel_sha256']
    assert len(audit['cases'])==1
    row=audit['cases'][0]
    WarningCalled(row)
    WarningAdmitted('*'+hex(extent_inventory['address']+row['carry_gate']),internal=True)
    for variant in row['variants']:
        for access in variant['access_instructions']: WarningRead(access,variant)
        WarningStaged(variant)

def warning_observation_complete():
    if extent_inventory.get('warning_input_audit') is None: return
    assert warning_active is None
    observed['warning_input_observation']=dict(passed=True,records=warning_records,
        audit_sha256=extent_inventory['warning_input_audit_sha256'],
        scope='Actual reached warning admissions and complete scalar register staging with retained selector, helper/captured register and boundary translation observations. Not continuous mapping lifetime, complete output/retirement, independent owner reconstruction or production qualification')
'''
