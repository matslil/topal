"""Read-only observations of ordinary admission, REP copy and completion."""

PROGRAM = r'''
copy_pending = None
copy_active = None
copy_counts = {}
copy_samples = {}
copy_start_breakpoints = {}
copy_finish_breakpoints = {}
copy_call_breakpoints = {}
copy_result_breakpoints = {}
copy_source_scope = None
copy_source_refusals = []
copy_source_completions = []
copy_register_names = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15']
copy_preserved = [2,3,5,6,7,8,9,12,13,14,15]

def copy_source_enabled():
    return extent_inventory is not None and extent_inventory.get('source_scoped_copies',False)

def copy_source_syscall_entered(call):
    global copy_source_scope
    if not copy_source_enabled():
        return
    assert copy_source_scope is None and copy_pending is copy_active is None
    copy_source_scope = dict(number=call['number'],args=list(call['args']),
        phase_before=call['phase_before'],refusals=[],completed=[])
    for points in [copy_call_breakpoints,copy_result_breakpoints,copy_start_breakpoints]:
        for point in points.values():
            assert not point.enabled
            point.enabled = call['number'] != 12
    assert not any(point.enabled for point in copy_finish_breakpoints.values())

def copy_source_syscall_returned(call):
    global copy_source_scope
    if not copy_source_enabled():
        return
    assert copy_source_scope is not None and copy_pending is copy_active is None
    scope = copy_source_scope
    assert (scope['number'],scope['args']) == (call['number'],call['args'])
    for event in scope['refusals']:
        assert call['result'] == -14 and call['phase_after'] == scope['phase_before']
        assert event['candidate'] in call['args']
        copy_source_refusals.append(dict(**event,number=call['number'],args=call['args'],
            result=call['result'],phase_before=scope['phase_before'],phase_after=call['phase_after']))
    for event in scope['completed']:
        assert call['result'] >= 0 and event['candidate'] in call['args']
        copy_source_completions.append(dict(**event,number=call['number'],args=call['args'],
            result=call['result'],phase_before=scope['phase_before'],phase_after=call['phase_after']))
    for points in [copy_call_breakpoints,copy_result_breakpoints,copy_start_breakpoints]:
        for point in points.values(): point.enabled = False
    assert not any(point.enabled for point in copy_finish_breakpoints.values())
    copy_source_scope = None

def copy_mode(before=None):
    result = [reg('cr0'),reg('cr3'),reg('cr4')]
    assert reg('cs')&3 == 0 and result[0]&(1<<16) and not reg('eflags')&0x200
    assert not result[2]&(1<<12)
    if before is not None:
        assert result == before
    return result

def copy_transfer_size(row,elements,admitted):
    assert row['copy_width'] > 0 and 0 <= elements < 1<<64 and 0 <= admitted < 1<<64
    size = elements*row['copy_width']
    assert size < 1<<64
    kind = row.get('copy_count_kind','exact')
    if kind == 'positive-prefix':
        assert 0 < size <= admitted <= row['admitted_maximum']
    else:
        assert kind == 'exact' and size == admitted
    return size

def copy_mapping(pointer):
    table = reg('cr3') & 0x000ffffffffff000
    path, user, writable = [], True, True
    for shift in [39,30,21,12]:
        assert table % 4096 == 0 and table < 0x40000000
        entry = word(table+((pointer>>shift)&511)*8)
        assert entry&1
        user, writable = user and bool(entry&4), writable and bool(entry&2)
        path.append([table,shift,entry&~0x60])
        if shift == 12 or (shift in [30,21] and entry&128):
            physical = (entry & (0x000fffffffffffff & ~((1<<shift)-1))) | (pointer&((1<<shift)-1))
            return dict(path=path,physical=physical,user=user,writable=writable,leaf_shift=shift)
        table = entry & 0x000ffffffffff000
    raise AssertionError('missing copy translation')

def copy_range(pointer, size, user, writable):
    # Observation budget only, not a kernel limit or a grant of access.
    assert 0 <= size <= 0x40000 and pointer+size < 1<<64
    result = []
    remaining, at = size, pointer
    while remaining:
        amount = min(remaining,4096-(at&4095))
        mapping = copy_mapping(at)
        assert mapping['user'] == user and (not writable or mapping['writable'])
        if user:
            assert mapping['leaf_shift'] == 12
        else:
            assert mapping['physical'] == at
        assert mapping['physical']+amount <= 0x40000000
        result.append(dict(pointer=at,size=amount,**mapping))
        remaining, at = remaining-amount, at+amount
    return result

class CopyAdmissionCalled(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['call_offset']),internal=True)
        self.row = row

    def observe(self):
        global copy_pending
        assert copy_pending is None and copy_active is None
        row = self.row
        copy_counts[row['name']]['attempts'] += 1
        copy_pending = dict(name=row['name'],candidate=reg('r8'),size=reg('r9'),
            registers=[reg(r) for r in copy_register_names],mode=copy_mode(),rsp=reg('rsp'),
            captured=memory(context+296,64),accepted=False)
        if copy_source_enabled():
            assert copy_source_scope is not None
            if row['call_offset'] not in extent_first_reach:
                extent_first_reach[row['call_offset']] = dict(offset=row['call_offset'],
                    target=next(r['target'] for r in extent_inventory['calls'] if r['offset']==row['call_offset']),
                    candidate=reg('r8'),length=reg('r9'),phase=memory(context+104,1)[0])
        return False

class CopyAdmissionResult(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['carry_gate']),internal=True)
        self.row = row

    def observe(self):
        global copy_pending
        assert copy_pending is not None and copy_pending['name'] == self.row['name']
        before = copy_pending
        copy_mode(before['mode'])
        assert reg('rsp') == before['rsp'] and word(reg('rsp')-8) == 0
        assert all(reg(copy_register_names[i]) == before['registers'][i] for i in copy_preserved)
        assert memory(context+296,64) == before['captured']
        if reg('eflags')&1:
            copy_counts[self.row['name']]['refused'] += 1
            if copy_source_enabled():
                copy_source_scope['refusals'].append(dict(name=before['name'],
                    call_offset=self.row['call_offset'],candidate=before['candidate'],size=before['size'],
                    helper_registers_preserved=True,captured_arguments_preserved=True))
            copy_pending = None
        else:
            copy_counts[self.row['name']]['admitted'] += 1
            copy_pending['accepted'] = True
        return False

class CopyStarted(Checked):
    def __init__(self,group):
        super().__init__('*'+hex(extent_inventory['address']+group['copy_offset']),internal=True)
        self.group = group

    def observe(self):
        global copy_active
        assert copy_pending is not None and copy_pending['accepted'] and copy_active is None
        # A debugger can revisit the REP PC between iterations. Observe its
        # initial state once, then resume ordinary execution to the completion
        # boundary; re-enable this breakpoint for the next invocation there.
        self.enabled = False
        copy_finish_breakpoints[self.group['copy_offset']].enabled = True
        assert copy_pending['name'] in self.group['cases']
        row = copy_rows[copy_pending['name']]
        before = copy_pending
        copy_mode(before['mode'])
        assert reg('rsp') == before['rsp'] and not reg('eflags')&(1<<10)
        assert memory(context+296,64) == before['captured']
        source, target, elements = reg('rsi'),reg('rdi'),reg('rcx')
        size = copy_transfer_size(row,elements,before['size'])
        if row.get('copy_count_kind') == 'positive-prefix':
            assert reg(copy_register_names[row['copy_count_register']]) == size
        assert (source if row['admitted_direction'] == 'read' else target) == before['candidate']
        source_map = copy_range(source,size,row['admitted_direction']=='read',False)
        target_map = copy_range(target,size,row['admitted_direction']=='write',True)
        assert all(a['physical']+a['size'] <= b['physical'] or b['physical']+b['size'] <= a['physical']
                   for a in source_map for b in target_map)
        tables = {entry[0] for m in source_map+target_map for entry in m['path']}
        assert all(m['physical']+m['size'] <= table or table+4096 <= m['physical']
                   for m in source_map+target_map for table in tables)
        payload = memory(source,size) if size else b''
        copy_active = dict(name=row['name'],source=source,target=target,size=size,
            elements=elements,source_map=source_map,target_map=target_map,payload=payload)
        return False

class CopyFinished(Checked):
    def __init__(self,group,size):
        super().__init__('*'+hex(extent_inventory['address']+group['copy_offset']+size),internal=True)
        self.group = group
        self.enabled = False

    def observe(self):
        global copy_active,copy_pending
        assert copy_active is not None and copy_pending is not None
        active, before = copy_active,copy_pending
        assert active['name'] == before['name'] and active['name'] in self.group['cases']
        row = copy_rows[active['name']]
        copy_mode(before['mode'])
        assert reg('rsp') == before['rsp'] and not reg('eflags')&(1<<10)
        assert reg('rcx') == 0 and reg('rsi') == active['source']+active['size']
        assert reg('rdi') == active['target']+active['size']
        assert memory(context+296,64) == before['captured']
        assert copy_range(active['source'],active['size'],row['admitted_direction']=='read',False) == active['source_map']
        assert copy_range(active['target'],active['size'],row['admitted_direction']=='write',True) == active['target_map']
        payload = memory(active['target'],active['size']) if active['size'] else b''
        assert payload == active['payload']
        counts = copy_counts[row['name']]
        counts['completed'] += 1
        counts['bytes'] += active['size']
        if copy_source_enabled():
            assert copy_source_scope is not None
            copy_source_scope['completed'].append(dict(name=row['name'],call_offset=row['call_offset'],
                copy_offset=row['copy_offset'],candidate=before['candidate'],size=active['size'],
                payload_hex=payload.hex(),helper_registers_preserved=True,
                captured_arguments_preserved=True,translation_observations_equal=True))
        if row['name'] not in copy_samples:
            copy_samples[row['name']] = dict(name=row['name'],call_offset=row['call_offset'],
                copy_offset=row['copy_offset'],source=active['source'],target=active['target'],
                size=active['size'],elements=active['elements'],mode=before['mode'],
                source_map=active['source_map'],target_map=active['target_map'],
                payload_sha256=hashlib.sha256(payload).hexdigest(),
                helper_registers_preserved=True,captured_arguments_preserved=True,
                translation_observations_equal=True,exact_payload_equal=True)
            if row.get('copy_count_kind') == 'positive-prefix':
                copy_samples[row['name']]['admitted_size'] = before['size']
                copy_samples[row['name']]['copy_count_kind'] = 'positive-prefix'
        copy_active,copy_pending = None,None
        self.enabled = False
        copy_start_breakpoints[self.group['copy_offset']].enabled = True
        return False

def install_copy_observation():
    global copy_rows
    audits = [extent_inventory['copy_transfer_audit']]
    if extent_inventory.get('staging_copy_audit') is not None:
        audits.append(extent_inventory['staging_copy_audit'])
    if copy_source_enabled():
        assert extent_inventory.get('staging_copy_audit') is not None
        audits = [extent_inventory['staging_copy_audit']]
    if extent_inventory.get('stat_output_audit') is not None:
        audits.append(extent_inventory['stat_output_audit'])
    if extent_inventory.get('serial_prefix_audit') is not None:
        audits.append(extent_inventory['serial_prefix_audit'])
    assert all(a['passed'] and a['kernel_sha256'] == extent_inventory['kernel_sha256'] for a in audits)
    rows = [r for a in audits for r in a['cases']]
    groups = [g for a in audits for g in a['copy_groups']]
    copy_rows = {r['name']:r for r in rows}
    assert len(copy_rows) == len(rows) and len({g['copy_offset'] for g in groups}) == len(groups)
    for row in rows:
        copy_counts[row['name']] = dict(attempts=0,admitted=0,refused=0,completed=0,bytes=0)
        copy_call_breakpoints[row['name']] = CopyAdmissionCalled(row)
        copy_result_breakpoints[row['name']] = CopyAdmissionResult(row)
    for group in groups:
        sizes = {copy_rows[name]['copy_size'] for name in group['cases']}
        assert len(sizes) == 1
        copy_start_breakpoints[group['copy_offset']] = CopyStarted(group)
        copy_finish_breakpoints[group['copy_offset']] = CopyFinished(group,next(iter(sizes)))
    if copy_source_enabled():
        for points in [copy_call_breakpoints,copy_result_breakpoints,copy_start_breakpoints]:
            for point in points.values(): point.enabled = False

def copy_observation_complete():
    assert copy_pending is None and copy_active is None
    assert copy_source_scope is None
    assert sum(row['completed'] for row in copy_counts.values()) > 0
    assert all(row['attempts'] == row['admitted']+row['refused'] and row['admitted'] == row['completed']
               for row in copy_counts.values())
    observed['copy_transfer_observation'] = dict(passed=True,counts=copy_counts,
        samples=[copy_samples[name] for name in sorted(copy_samples)],
        unreached=[name for name,row in copy_counts.items() if not row['attempts']],
        kernel_sha256=extent_inventory['kernel_sha256'],
        audit_sha256=extent_inventory['staging_copy_audit_sha256'] if copy_source_enabled() else extent_inventory['copy_transfer_audit_sha256'],
        audit_kind='staging' if copy_source_enabled() else ('legacy-and-staging' if extent_inventory.get('staging_copy_audit') is not None else 'legacy'),
        scope='Every selected observed admission result and REP start/completion: preserved helper registers/captured arguments, actual candidate/count/DF, equal copied bytes, user/supervisor permissions, no observed physical overlap with counterparty/translation storage and unchanged boundary translation/control observations. Read-only GDB observation, not continuous check-to-copy stability, complete table/backing ownership, every possible path or production qualification')
    if extent_inventory.get('staging_copy_audit') is not None:
        observed['copy_transfer_observation']['staging_audit_sha256'] = extent_inventory['staging_copy_audit_sha256']
    if extent_inventory.get('stat_output_audit') is not None:
        observed['copy_transfer_observation']['stat_audit_sha256'] = extent_inventory['stat_output_audit_sha256']
        observed['copy_transfer_observation']['audit_kind'] += '-and-stat'
    if extent_inventory.get('serial_prefix_audit') is not None:
        observed['copy_transfer_observation']['serial_prefix_audit_sha256'] = extent_inventory['serial_prefix_audit_sha256']
        observed['copy_transfer_observation']['audit_kind'] += '-and-serial-prefix'
    if copy_source_enabled():
        assert len(copy_source_refusals) == sum(row['refused'] for row in copy_counts.values())
        observed['copy_transfer_observation']['source_syscall_refusals'] = copy_source_refusals
        assert len(copy_source_completions) == sum(row['completed'] for row in copy_counts.values())
        observed['copy_transfer_observation']['source_syscall_completions'] = copy_source_completions
        observed['copy_transfer_observation']['source_syscall_scoped'] = True
'''
