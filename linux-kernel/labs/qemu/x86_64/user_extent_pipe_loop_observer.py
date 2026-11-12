"""Read-only ordinary FIFO byte-loop boundary observations."""

PROGRAM = r'''
pipe_loop_scope = None
pipe_loop_pending = None
pipe_loop_active = None
pipe_loop_counts = {}
pipe_loop_records = []
pipe_loop_refusals = []
pipe_loop_points = []

def pipe_loop_enabled():
    return extent_inventory is not None and 'pipe-loops' in selected_extent_observers()

def pipe_loop_syscall_entered(call):
    global pipe_loop_scope
    if not pipe_loop_enabled(): return
    assert pipe_loop_scope is pipe_loop_pending is pipe_loop_active is None
    pipe_loop_scope = dict(number=call['number'],args=list(call['args']),
        phase_before=call['phase_before'],records=[],refusals=[])
    selected = call['number'] in [0,1] and call['args'][0] >= 3
    for point in pipe_loop_points:
        assert not point.enabled
        point.enabled = selected and point.kind in ['call','result','start']

def pipe_loop_syscall_returned(call):
    global pipe_loop_scope
    if not pipe_loop_enabled(): return
    scope = pipe_loop_scope
    assert scope is not None and pipe_loop_pending is pipe_loop_active is None
    assert (scope['number'],scope['args']) == (call['number'],call['args'])
    for row in scope['records']:
        assert call['result'] == row['size'] and row['candidate'] == call['args'][1]
        assert row['size'] <= call['args'][2]
        pipe_loop_records.append(dict(**row,number=call['number'],args=call['args'],result=call['result']))
    for row in scope['refusals']:
        assert call['result'] == -14 and row['candidate'] == call['args'][1]
        assert scope['phase_before'] == call['phase_after']
        pipe_loop_refusals.append(dict(**row,number=call['number'],args=call['args'],result=call['result']))
    for point in pipe_loop_points: point.enabled = False
    pipe_loop_scope = None

def pipe_loop_boundary(before):
    copy_mode(before['mode'])
    assert reg('rsp') == before['rsp'] and reg('r8') == before['candidate'] and reg('r9') == before['size']
    assert memory(context+296,64) == before['captured']
    assert reg('r13') == before['object']
    assert memory(before['object'],128) == before['object_before']
    assert copy_range(before['candidate'],before['size'],True,before['direction']=='write') == before['mapping']

class PipeLoopPoint(Checked):
    def __init__(self,row,kind):
        field = dict(call='call_offset',result='carry_gate',start='start_offset',finish='end_offset')[kind]
        super().__init__('*'+hex(extent_inventory['address']+row[field]),internal=True)
        self.row,self.kind = row,kind
        self.enabled = False

    def observe(self):
        global pipe_loop_pending,pipe_loop_active
        row,kind = self.row,self.kind
        counts = pipe_loop_counts[row['name']]
        if kind == 'call':
            assert pipe_loop_scope is not None and pipe_loop_pending is pipe_loop_active is None
            n,args = pipe_loop_scope['number'],pipe_loop_scope['args']
            assert n == (0 if row['admitted_direction']=='write' else 1)
            table,obj = word(context+928),reg('r13')
            index = (obj-table-0x3e000)//128
            assert 0 <= index < 4 and obj == table+0x3e000+index*128 and word(obj)==1
            cursor,queued = word(obj+32),word(obj+40)
            assert cursor < 4096 and queued <= 4096
            assert reg('r8') == args[1]
            expected = min(args[2],queued if n==0 else 4096-queued)
            assert reg('r9') == expected and 1 <= expected <= 4096
            counts['attempts'] += 1
            pipe_loop_pending = dict(name=row['name'],candidate=reg('r8'),size=reg('r9'),
                direction=row['admitted_direction'],registers=[reg(r) for r in copy_register_names],
                rsp=reg('rsp'),mode=copy_mode(),captured=memory(context+296,64),
                object=obj,object_before=memory(obj,128),ring=table+0x3f000+index*4096,
                cursor=cursor if n==0 else (cursor+queued)&4095,accepted=False)
        elif kind == 'result':
            before = pipe_loop_pending
            assert before is not None and before['name']==row['name'] and pipe_loop_active is None
            copy_mode(before['mode'])
            assert reg('rsp') == before['rsp'] and word(reg('rsp')-8)==0
            assert all(reg(copy_register_names[i]) == before['registers'][i] for i in copy_preserved)
            assert memory(context+296,64)==before['captured']
            if reg('eflags')&1:
                counts['refused'] += 1
                pipe_loop_scope['refusals'].append(dict(name=row['name'],candidate=before['candidate'],size=before['size']))
                pipe_loop_pending = None
            else:
                before['mapping'] = copy_range(before['candidate'],before['size'],True,before['direction']=='write')
                for pointer,size in [(word(context+928),8192),(before['object'],128),
                                     (before['ring'],4096),(context+296,64),(before['rsp']-8,16)]:
                    other = copy_range(pointer,size,False,True)
                    assert all(a['physical']+a['size'] <= b['physical'] or b['physical']+b['size'] <= a['physical']
                        for a in before['mapping'] for b in other)
                tables = {entry[0] for m in before['mapping'] for entry in m['path']}
                assert all(m['physical']+m['size']<=table or table+4096<=m['physical'] for m in before['mapping'] for table in tables)
                before['accepted'] = True
                counts['admitted'] += 1
        elif kind == 'start':
            before = pipe_loop_pending
            assert before is not None and before['accepted'] and pipe_loop_active is None
            assert before['name']==row['name'] and reg('rcx')==before['size']
            pipe_loop_boundary(before)
            assert reg('r11')==before['ring'] and reg('r15')==before['cursor']
            user = reg('rdi' if before['direction']=='write' else 'rsi')
            assert user==before['candidate']
            ring = memory(before['ring'],4096)
            payload = (bytes(ring[(before['cursor']+i)&4095] for i in range(before['size']))
                if before['direction']=='write' else memory(user,before['size']))
            expected = bytearray(ring)
            for i in range(before['size']):
                expected[(before['cursor']+i)&4095] = 0 if before['direction']=='write' else payload[i]
            pipe_loop_active = dict(payload=payload,expected_ring=bytes(expected),
                ring_mapping=copy_range(before['ring'],4096,False,True))
            self.enabled = False
            next(p for p in pipe_loop_points if p.row is row and p.kind=='finish').enabled = True
        else:
            assert kind=='finish'
            before,active = pipe_loop_pending,pipe_loop_active
            assert before is not None and active is not None and before['name']==row['name']
            pipe_loop_boundary(before)
            assert reg('rcx')==0 and reg('r15')==(before['cursor']+before['size'])&4095
            assert reg('rdi' if before['direction']=='write' else 'rsi')==before['candidate']+before['size']
            assert reg('r11')==before['ring']
            assert copy_range(before['ring'],4096,False,True)==active['ring_mapping']
            assert memory(before['ring'],4096)==active['expected_ring']
            assert memory(before['candidate'],before['size'])==active['payload']
            counts['completed'] += 1
            counts['bytes'] += before['size']
            pipe_loop_scope['records'].append(dict(name=row['name'],candidate=before['candidate'],size=before['size'],
                ring=before['ring'],initial_cursor=before['cursor'],payload_hex=active['payload'].hex(),
                helper_registers_preserved=True,captured_arguments_preserved=True,exact_user_and_ring_bytes=True,
                boundary_mappings_equal=True,ring_wrap=before['cursor']+before['size']>4096))
            pipe_loop_pending,pipe_loop_active = None,None
            self.enabled = False
            next(p for p in pipe_loop_points if p.row is row and p.kind=='start').enabled = True
        return False

def install_pipe_loop_observation():
    audit = extent_inventory['pipe_loop_audit']
    assert audit['passed'] and not audit['complete'] and audit['kernel_sha256']==extent_inventory['kernel_sha256']
    assert {r['name'] for r in audit['cases']}=={'pipe-read-loop','pipe-write-loop'}
    for row in audit['cases']:
        pipe_loop_counts[row['name']] = dict(attempts=0,admitted=0,refused=0,completed=0,bytes=0)
        for kind in ['call','result','start','finish']: pipe_loop_points.append(PipeLoopPoint(row,kind))

def pipe_loop_observation_complete():
    assert pipe_loop_scope is pipe_loop_pending is pipe_loop_active is None
    assert all(c['attempts']==c['admitted']+c['refused'] and c['admitted']==c['completed'] and
        c['completed']>0 and c['refused']>0 for c in pipe_loop_counts.values())
    assert len(pipe_loop_records)==sum(c['completed'] for c in pipe_loop_counts.values())
    assert len(pipe_loop_refusals)==sum(c['refused'] for c in pipe_loop_counts.values())
    observed['pipe_loop_observation'] = dict(passed=True,counts=pipe_loop_counts,records=pipe_loop_records,
        refusals=pipe_loop_refusals,kernel_sha256=extent_inventory['kernel_sha256'],
        audit_sha256=extent_inventory['pipe_loop_audit_sha256'],
        scope='Read-only source syscall/helper and byte-loop entry/completion observations: exact admitted size, captured arguments, complete payload/ring effects, selected physical exclusions and equal boundary translations/control. No per-byte native transcript, continuous lifetime, complete owner/nonalias/helper/entry/callee/exception or production claim')
'''
