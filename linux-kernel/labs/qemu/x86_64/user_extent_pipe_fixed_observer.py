"""Read-only observation of fixed pipe outputs through ordinary syscalls."""

PROGRAM = r'''
pipe_fixed_pending = None
pipe_fixed_active = None
pipe_fixed_scope = None
pipe_fixed_counts = {}
pipe_fixed_records = []
pipe_fixed_refusals = []
pipe_fixed_points = {}
pipe_fixed_calls = {}
pipe_fixed_results = {}

def pipe_fixed_enabled():
    return extent_inventory is not None and 'pipe-fixed' in selected_extent_observers()

def pipe_fixed_disjoint(left,right):
    assert all(a['physical']+a['size'] <= b['physical'] or
               b['physical']+b['size'] <= a['physical'] for a in left for b in right)

def pipe_fixed_candidate_mapping(before):
    mappings = copy_range(before['candidate'],before['size'],True,True)
    tables = {entry[0] for mapping in mappings for entry in mapping['path']}
    assert all(mapping['physical']+mapping['size'] <= table or
               table+4096 <= mapping['physical'] for mapping in mappings for table in tables)
    return mappings

def pipe_fixed_boundary(before):
    copy_mode(before['mode'])
    assert reg('rsp') == before['rsp']
    assert reg('r8') == before['candidate'] and reg('r9') == before['size']
    assert memory(context+296,64) == before['captured']
    assert pipe_fixed_candidate_mapping(before) == before['mapping']

def pipe_fixed_syscall_entered(call):
    global pipe_fixed_scope
    if not pipe_fixed_enabled():
        return
    assert pipe_fixed_scope is pipe_fixed_pending is pipe_fixed_active is None
    pipe_fixed_scope = dict(number=call['number'],args=list(call['args']),
                            phase_before=call['phase_before'],refusals=[])
    selected = call['number'] in [16,22,293]
    for points in [pipe_fixed_calls,pipe_fixed_results,pipe_fixed_points]:
        for point in points.values():
            assert not point.enabled
            point.enabled = selected

def pipe_fixed_syscall_returned(call):
    global pipe_fixed_scope
    if not pipe_fixed_enabled():
        return
    scope = pipe_fixed_scope
    assert scope is not None and pipe_fixed_pending is pipe_fixed_active is None
    assert (scope['number'],scope['args']) == (call['number'],call['args'])
    for refusal in scope['refusals']:
        assert call['result'] == -14 and call['phase_after'] == scope['phase_before']
        assert refusal['candidate'] == scope['args'][0 if scope['number'] in [22,293] else 2]
        pipe_fixed_refusals.append(dict(**refusal,number=scope['number'],args=scope['args'],
            result=call['result'],phase_before=scope['phase_before'],phase_after=call['phase_after']))
    for points in [pipe_fixed_calls,pipe_fixed_results,pipe_fixed_points]:
        for point in points.values(): point.enabled = False
    pipe_fixed_scope = None

class PipeFixedCalled(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['call_offset']),internal=True)
        self.row = row
        self.enabled = False

    def observe(self):
        global pipe_fixed_pending
        row,scope = self.row,pipe_fixed_scope
        assert scope is not None and pipe_fixed_pending is pipe_fixed_active is None
        number,args = scope['number'],scope['args']
        if row['name'] == 'pipe-pair-output':
            assert number in [22,293]
            candidate = args[0]
        else:
            assert row['name'] == 'pipe-readable-count-output'
            assert number == 16 and args[1] == 0x541b
            candidate = args[2]
        assert reg('r8') == candidate and reg('r9') == row['admitted_bytes']
        pipe_fixed_counts[row['name']]['attempts'] += 1
        pipe_fixed_pending = dict(name=row['name'],candidate=candidate,size=reg('r9'),
            registers=[reg(name) for name in copy_register_names],mode=copy_mode(),
            rsp=reg('rsp'),captured=memory(context+296,64),accepted=False,next_access=0,
            accesses=[],number=number,args=list(args))
        return False

class PipeFixedResult(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['success_edge'][0]),internal=True)
        self.row = row
        self.enabled = False

    def observe(self):
        global pipe_fixed_pending
        before = pipe_fixed_pending
        assert before is not None and before['name'] == self.row['name']
        copy_mode(before['mode'])
        assert reg('rsp') == before['rsp'] and word(reg('rsp')-8) == 0
        assert all(reg(copy_register_names[i]) == before['registers'][i] for i in copy_preserved)
        assert memory(context+296,64) == before['captured']
        if reg('eflags')&1:
            pipe_fixed_counts[before['name']]['refused'] += 1
            pipe_fixed_scope['refusals'].append(dict(name=before['name'],
                candidate=before['candidate'],size=before['size'],call_offset=self.row['call_offset'],
                helper_registers_preserved=True,captured_arguments_preserved=True))
            pipe_fixed_pending = None
        else:
            before['accepted'] = True
            before['mapping'] = pipe_fixed_candidate_mapping(before)
            table = word(context+928)
            # These observed supervisor ranges are exclusions, not owner grants
            # or a complete inventory of every kernel/private access.
            private = [(table,8192),(table+0x3d000,1024),(table+0x3e000,0x5000),
                       (context+296,64),(before['rsp']-8,16)]
            before['private_exclusions'] = []
            for pointer,size in private:
                mapping = copy_range(pointer,size,False,True)
                pipe_fixed_disjoint(before['mapping'],mapping)
                before['private_exclusions'].append(dict(pointer=pointer,size=size,mapping=mapping))
            pipe_fixed_counts[before['name']]['admitted'] += 1
        return False

def pipe_fixed_access_started(row,index):
    global pipe_fixed_active
    before = pipe_fixed_pending
    assert before is not None and before['accepted'] and pipe_fixed_active is None
    assert before['name'] == row['name'] and before['next_access'] == index
    access = row['access_instructions'][index]
    assert access['candidate_offset'] == 0 and access['base_register'] == 8
    assert access['width'] == before['size'] and access['action'] in ['read','write']
    pipe_fixed_boundary(before)
    pointer,size = before['candidate'],before['size']
    payload = (reg('rax')&((1<<(size*8))-1)).to_bytes(size,'little') if access['action']=='write' else memory(pointer,size)
    pipe_fixed_active = dict(index=index,offset=access['offset'],action=access['action'],
                             payload=payload,rsp=reg('rsp'))

def pipe_fixed_access_finished(row,index):
    global pipe_fixed_active,pipe_fixed_pending
    before,active = pipe_fixed_pending,pipe_fixed_active
    assert before is not None and active is not None
    assert before['name'] == row['name'] and active['index'] == index == before['next_access']
    pipe_fixed_boundary(before)
    payload = memory(before['candidate'],before['size'])
    assert payload == active['payload']
    if active['action'] == 'read':
        assert reg('r11') == int.from_bytes(payload,'little')
    before['accesses'].append(dict(offset=active['offset'],action=active['action'],
        bytes=before['size'],payload_hex=payload.hex(),exact_payload_equal=True,
        boundary_mapping_equal=True,candidate_count_preserved=True,captured_arguments_preserved=True))
    before['next_access'] += 1
    pipe_fixed_active = None
    if before['next_access'] == len(row['access_instructions']):
        pipe_fixed_counts[row['name']]['completed'] += 1
        pipe_fixed_records.append(dict(name=row['name'],call_offset=row['call_offset'],
            candidate=before['candidate'],size=before['size'],number=before['number'],args=before['args'],
            mode=before['mode'],mapping=before['mapping'],private_exclusions=before['private_exclusions'],
            accesses=before['accesses'],helper_registers_preserved=True))
        pipe_fixed_pending = None

class PipeFixedPoint(Checked):
    def __init__(self,offset):
        super().__init__('*'+hex(extent_inventory['address']+offset),internal=True)
        self.finishes,self.starts = [],[]
        self.enabled = False

    def observe(self):
        # A store completion can share the readback's starting PC. One point
        # orders those events explicitly instead of relying on GDB BP order.
        for row,index in self.finishes: pipe_fixed_access_finished(row,index)
        for row,index in self.starts: pipe_fixed_access_started(row,index)
        return False

def install_pipe_fixed_observation():
    audit = extent_inventory['pipe_fixed_output_audit']
    assert audit['passed'] and not audit['complete']
    assert audit['kernel_sha256'] == extent_inventory['kernel_sha256']
    assert {row['name'] for row in audit['cases']} == {'pipe-pair-output','pipe-readable-count-output'}
    for row in audit['cases']:
        pipe_fixed_counts[row['name']] = dict(attempts=0,admitted=0,refused=0,completed=0)
        pipe_fixed_calls[row['name']] = PipeFixedCalled(row)
        pipe_fixed_results[row['name']] = PipeFixedResult(row)
        for index,access in enumerate(row['access_instructions']):
            start,end = access['offset'],access['offset']+len(bytes.fromhex(access['bytes']))
            for at in [start,end]:
                if at not in pipe_fixed_points: pipe_fixed_points[at] = PipeFixedPoint(at)
            pipe_fixed_points[start].starts.append((row,index))
            pipe_fixed_points[end].finishes.append((row,index))

def pipe_fixed_observation_complete():
    assert pipe_fixed_pending is pipe_fixed_active is pipe_fixed_scope is None
    assert all(row['attempts'] == row['admitted']+row['refused'] and
               row['admitted'] == row['completed'] and row['completed'] > 0 and row['refused'] > 0
               for row in pipe_fixed_counts.values())
    assert len(pipe_fixed_refusals) == sum(row['refused'] for row in pipe_fixed_counts.values())
    assert len(pipe_fixed_records) == sum(row['completed'] for row in pipe_fixed_counts.values())
    observed['pipe_fixed_output_observation'] = dict(passed=True,counts=pipe_fixed_counts,
        records=pipe_fixed_records,refusals=pipe_fixed_refusals,
        kernel_sha256=extent_inventory['kernel_sha256'],audit_sha256=extent_inventory['pipe_fixed_output_audit_sha256'],
        scope='Read-only ordinary source syscall observations: helper preservation/refusal, exact fixed stores and readback values, preserved arguments/count, user-writable translation and selected supervisor/table physical exclusions with equal access-boundary mode/mappings. Not continuous lifetime, complete owner/nonalias/entry/callee/exception effects, all callers or production qualification')
'''
