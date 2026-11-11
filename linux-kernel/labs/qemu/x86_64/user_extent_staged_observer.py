"""Read-only paired observations of indexed and unrolled staged transfers."""

PROGRAM = r'''
staged_pending = None
staged_active = None
staged_counts = {}
staged_samples = {}
staged_start_breakpoints = {}
staged_finish_breakpoints = {}

class StagedCalled(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['call_offset']),internal=True)
        self.row = row

    def observe(self):
        global staged_pending
        assert staged_pending is None and staged_active is None
        assert copy_pending is None and copy_active is None
        row = self.row
        assert reg('r9') == row['length'] and reg('gs_base') == context
        staged_counts[row['name']]['attempts'] += 1
        staged_pending = dict(name=row['name'],candidate=reg('r8'),size=reg('r9'),
            registers=[reg(r) for r in copy_register_names],mode=copy_mode(),rsp=reg('rsp'),
            captured=memory(context+296,64),accepted=False)
        return False

class StagedResult(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['carry_gate']),internal=True)
        self.row = row

    def observe(self):
        global staged_pending
        assert staged_pending is not None and staged_pending['name'] == self.row['name']
        before = staged_pending
        copy_mode(before['mode'])
        assert reg('gs_base') == context
        assert reg('rsp') == before['rsp'] and word(reg('rsp')-8) == 0
        assert all(reg(copy_register_names[i]) == before['registers'][i] for i in copy_preserved)
        assert memory(context+296,64) == before['captured']
        if reg('eflags')&1:
            staged_counts[self.row['name']]['refused'] += 1
            staged_pending = None
        else:
            staged_counts[self.row['name']]['admitted'] += 1
            staged_pending['accepted'] = True
        return False

class StagedStarted(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['start_offset']),internal=True)
        self.row = row

    def observe(self):
        global staged_active
        row = self.row
        assert staged_pending is not None and staged_pending['accepted'] and staged_active is None
        assert staged_pending['name'] == row['name']
        self.enabled = False
        staged_finish_breakpoints[row['name']].enabled = True
        before = staged_pending
        copy_mode(before['mode'])
        assert reg('gs_base') == context
        assert reg('rsp') == before['rsp'] and memory(context+296,64) == before['captured']
        assert reg(copy_register_names[row['user_register']]) == before['candidate']
        if row['index_register'] is not None:
            assert reg(copy_register_names[row['index_register']]) == 0
        counter = row['counterpart']
        counterpart = context+counter['offset'] if counter['kind']=='gs' else counter['address']
        source,target = (before['candidate'],counterpart) if row['direction']=='read' else (counterpart,before['candidate'])
        size = before['size']
        source_map = copy_range(source,size,row['direction']=='read',False)
        target_map = copy_range(target,size,row['direction']=='write',True)
        assert all(a['physical']+a['size'] <= b['physical'] or b['physical']+b['size'] <= a['physical']
                   for a in source_map for b in target_map)
        tables = {entry[0] for m in source_map+target_map for entry in m['path']}
        assert all(m['physical']+m['size'] <= table or table+4096 <= m['physical']
                   for m in source_map+target_map for table in tables)
        staged_active = dict(name=row['name'],source=source,target=target,size=size,
            source_map=source_map,target_map=target_map,payload=memory(source,size))
        return False

class StagedFinished(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['end_offset']),internal=True)
        self.row = row
        self.enabled = False

    def observe(self):
        global staged_pending,staged_active
        row = self.row
        assert staged_pending is not None and staged_active is not None
        before,active = staged_pending,staged_active
        assert before['name'] == active['name'] == row['name']
        copy_mode(before['mode'])
        assert reg('gs_base') == context
        assert reg('rsp') == before['rsp'] and memory(context+296,64) == before['captured']
        assert reg(copy_register_names[row['user_register']]) == before['candidate']
        if row['index_register'] is not None:
            assert reg(copy_register_names[row['index_register']]) == row['length']
        assert copy_range(active['source'],active['size'],row['direction']=='read',False) == active['source_map']
        assert copy_range(active['target'],active['size'],row['direction']=='write',True) == active['target_map']
        payload = memory(active['target'],active['size'])
        assert payload == active['payload']
        counts = staged_counts[row['name']]
        counts['completed'] += 1
        counts['bytes'] += active['size']
        if row['name'] not in staged_samples:
            staged_samples[row['name']] = dict(name=row['name'],call_offset=row['call_offset'],
                start_offset=row['start_offset'],end_offset=row['end_offset'],
                source=active['source'],target=active['target'],size=active['size'],mode=before['mode'],
                source_map=active['source_map'],target_map=active['target_map'],
                payload_sha256=hashlib.sha256(payload).hexdigest(),
                helper_registers_preserved=True,captured_arguments_preserved=True,
                candidate_preserved=True,exact_payload_equal=True,translation_observations_equal=True)
        staged_pending,staged_active = None,None
        self.enabled = False
        staged_start_breakpoints[row['name']].enabled = True
        return False

def install_staged_observation():
    audit = extent_inventory['staged_transfer_audit']
    assert audit['passed'] and audit['kernel_sha256'] == extent_inventory['kernel_sha256']
    for row in audit['cases']:
        staged_counts[row['name']] = dict(attempts=0,admitted=0,refused=0,completed=0,bytes=0)
        StagedCalled(row)
        StagedResult(row)
        staged_start_breakpoints[row['name']] = StagedStarted(row)
        staged_finish_breakpoints[row['name']] = StagedFinished(row)

def staged_observation_complete():
    assert staged_pending is None and staged_active is None
    assert sum(row['completed'] for row in staged_counts.values()) > 0
    assert all(row['attempts']==row['admitted']+row['refused'] and row['admitted']==row['completed']
               for row in staged_counts.values())
    observed['staged_transfer_observation'] = dict(passed=True,counts=staged_counts,
        samples=[staged_samples[name] for name in sorted(staged_samples)],
        unreached=[name for name,row in staged_counts.items() if not row['completed']],
        kernel_sha256=extent_inventory['kernel_sha256'],
        audit_sha256=extent_inventory['staged_transfer_audit_sha256'],
        scope='Every selected observed admission/transfer boundary: actual candidate/count and finite-loop indices, helper/captured argument preservation, exact payload equality, complete covered user/supervisor permissions and unchanged control/translation observations allowing hardware A/D. Observed source/destination ranges do not overlap each other or table pages on these translation paths. Not continuous stability, complete owner/counterpart authority, every syscall path or production qualification')
'''
