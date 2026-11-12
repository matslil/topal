"""Read-only bounded random output admission, extraction and byte-loop checks."""

PROGRAM = r'''
import hashlib

random_active = None
random_records = []

class RandomCalled(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['call_offset']),internal=True)
        self.row=row

    def observe(self):
        global random_active
        assert random_active is None
        assert reg('r8')==reg('rdi') and reg('r9')==reg('rsi')
        assert 1<=reg('r9')<=16
        service=word(context+256)
        assert service and memory(context+184,1)[0] in [1,2,3]
        random_active=dict(row=self.row,candidate=reg('r8'),size=reg('r9'),rsp=reg('rsp'),
            registers=[reg(name) for name in copy_register_names],captured=memory(context+296,120),
            mode=copy_mode(),service=service,marker=memory(context+184,1)[0],
            private=memory(service+168,88),admitted=False,started=False,stores=[],readbacks=[])
        return False

class RandomAdmitted(Checked):
    def observe(self):
        global random_active
        before=random_active
        assert before is not None and not before['admitted']
        assert reg('rsp')==before['rsp'] and memory(reg('rsp')-8,8)==bytes(8)
        assert [reg(copy_register_names[i]) for i in copy_preserved]==[before['registers'][i] for i in copy_preserved]
        assert memory(context+296,120)==before['captured']
        assert word(context+256)==before['service'] and memory(context+184,1)[0]==before['marker']
        assert memory(before['service']+168,88)==before['private']
        copy_mode(before['mode'])
        if reg('eflags')&1:
            random_records.append(dict(candidate=before['candidate'],size=before['size'],accepted=False,
                marker=before['marker'],service_preserved=True,helper_registers_preserved=True,
                captured_arguments_preserved=True,stores=[],readbacks=[]))
            random_active=None
        else: before['admitted']=True
        return False

class RandomLoop(Checked):
    def observe(self):
        before=random_active
        assert before is not None and before['admitted']
        assert reg('rdi')==before['candidate'] and reg('rsi')==before['size']
        assert reg('rsp')==before['rsp'] and memory(context+296,120)==before['captured']
        assert word(context+272)==0 and word(context+256)==before['service']
        assert memory(context+184,1)[0]==before['marker']
        copy_mode(before['mode'])
        index=reg('r10')
        assert index==len(before['stores'])==len(before['readbacks']) and index<=before['size']
        if not before['started']:
            assert index==0
            source=before['service']+(168 if before['marker']==1 else 224)
            assert reg('r9')==source
            before['source']=source
            before['payload']=memory(source,before['size'])
            before['source_map']=copy_range(source,before['size'],False,False)
            before['target_map']=copy_range(before['candidate'],before['size'],True,True)
            assert all(a['physical']+a['size']<=b['physical'] or b['physical']+b['size']<=a['physical']
                for a in before['source_map'] for b in before['target_map'])
            before['tables']={p:memory(p,4096) for m in before['source_map']+before['target_map'] for p,_,_ in m['path']}
            before['started']=True
        else: assert reg('r9')==before['source']
        return False

class RandomStore(Checked):
    def observe(self):
        before=random_active
        assert before is not None and before['started']
        index=reg('r10')
        assert index==len(before['stores'])==len(before['readbacks']) and index<before['size']
        assert reg('rdi')==before['candidate'] and reg('r9')==before['source']
        assert reg('rax')&255==before['payload'][index]
        before['stores'].append(index)
        return False

class RandomReadback(Checked):
    def observe(self):
        before=random_active
        assert before is not None and before['started']
        index=reg('r10')
        assert index==len(before['readbacks']) and len(before['stores'])==index+1
        assert memory(before['candidate']+index,1)==bytes([before['payload'][index]])
        assert reg('rax')&255==before['payload'][index]
        before['readbacks'].append(index)
        return False

class RandomCompleted(Checked):
    def observe(self):
        global random_active
        before=random_active
        assert before is not None and before['started']
        assert reg('r10')==reg('rsi')==before['size'] and reg('rdi')==before['candidate']
        assert reg('rsp')==before['rsp'] and memory(context+296,120)==before['captured']
        assert before['stores']==before['readbacks']==list(range(before['size']))
        assert memory(before['candidate'],before['size'])==before['payload']
        assert memory(before['source'],before['size'])==before['payload']
        assert copy_range(before['source'],before['size'],False,False)==before['source_map']
        assert copy_range(before['candidate'],before['size'],True,True)==before['target_map']
        assert all(memory(p,4096)==raw for p,raw in before['tables'].items())
        assert word(context+272)==0 and word(context+256)==before['service']
        assert memory(context+184,1)[0]==before['marker']
        copy_mode(before['mode'])
        random_records.append(dict(candidate=before['candidate'],size=before['size'],accepted=True,
            marker=before['marker'],source=before['source'],source_offset=before['source']-before['service'],
            stores=before['stores'],readbacks=before['readbacks'],
            payload_sha256=hashlib.sha256(before['payload']).hexdigest(),
            exact_payload_equal=True,captured_arguments_preserved=True,stack_restored=True,
            suspended_stack_field_cleared=True,translation_observations_equal=True))
        random_active=None
        return False

def install_random_observation():
    audit=extent_inventory.get('random_output_audit')
    if audit is None: return
    assert audit['passed'] and audit['kernel_sha256']==extent_inventory['kernel_sha256']
    assert len(audit['cases'])==1
    row=audit['cases'][0]
    RandomCalled(row)
    for cls,field in [(RandomAdmitted,'carry_gate'),(RandomLoop,'loop_offset'),
                      (RandomStore,'start_offset'),(RandomReadback,'readback_offset'),
                      (RandomCompleted,'end_offset')]:
        cls('*'+hex(extent_inventory['address']+row[field]),internal=True)

def random_observation_complete():
    if extent_inventory.get('random_output_audit') is None: return
    assert random_active is None
    observed['random_output_observation']=dict(passed=True,records=random_records,
        audit_sha256=extent_inventory['random_output_audit_sha256'],
        scope='Actual reached random admissions, per-byte output/readback order, opaque payload equality, source/destination physical exclusion, restored caller stack and cleared suspended-stack field with equal boundary mappings. Not continuous stability, independent owners, extraction scratch/outer spill retirement, full syscall/fatal/exception or production qualification')
'''
