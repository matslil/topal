"""Read-only observations of ordinary admission, REP copy and completion."""

PROGRAM = r'''
copy_pending = None
copy_active = None
copy_counts = {}
copy_samples = {}
copy_register_names = ['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi','r8','r9','r10','r11','r12','r13','r14','r15']
copy_preserved = [2,3,5,6,7,8,9,12,13,14,15]

def copy_mode(before=None):
    result = [reg('cr0'),reg('cr3'),reg('cr4')]
    assert reg('cs')&3 == 0 and result[0]&(1<<16) and not reg('eflags')&0x200
    assert not result[2]&(1<<12)
    if before is not None:
        assert result == before
    return result

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
        assert copy_pending['name'] in self.group['cases']
        row = copy_rows[copy_pending['name']]
        before = copy_pending
        copy_mode(before['mode'])
        assert reg('rsp') == before['rsp'] and not reg('eflags')&(1<<10)
        assert memory(context+296,64) == before['captured']
        source, target, elements = reg('rsi'),reg('rdi'),reg('rcx')
        size = elements*row['copy_width']
        assert size == before['size'] and size < 1<<64
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
        if row['name'] not in copy_samples:
            copy_samples[row['name']] = dict(name=row['name'],call_offset=row['call_offset'],
                copy_offset=row['copy_offset'],source=active['source'],target=active['target'],
                size=active['size'],elements=active['elements'],mode=before['mode'],
                source_map=active['source_map'],target_map=active['target_map'],
                payload_sha256=hashlib.sha256(payload).hexdigest(),
                helper_registers_preserved=True,captured_arguments_preserved=True,
                translation_observations_equal=True,exact_payload_equal=True)
        copy_active,copy_pending = None,None
        return False

def install_copy_observation():
    global copy_rows
    audit = extent_inventory['copy_transfer_audit']
    assert audit['passed'] and audit['kernel_sha256'] == extent_inventory['kernel_sha256']
    copy_rows = {r['name']:r for r in audit['cases']}
    for row in audit['cases']:
        copy_counts[row['name']] = dict(attempts=0,admitted=0,refused=0,completed=0,bytes=0)
        CopyAdmissionCalled(row)
        CopyAdmissionResult(row)
    for group in audit['copy_groups']:
        sizes = {copy_rows[name]['copy_size'] for name in group['cases']}
        assert len(sizes) == 1
        CopyStarted(group)
        CopyFinished(group,next(iter(sizes)))

def copy_observation_complete():
    assert copy_pending is None and copy_active is None
    assert sum(row['completed'] for row in copy_counts.values()) > 0
    assert all(row['attempts'] == row['admitted']+row['refused'] and row['admitted'] == row['completed']
               for row in copy_counts.values())
    observed['copy_transfer_observation'] = dict(passed=True,counts=copy_counts,
        samples=[copy_samples[name] for name in sorted(copy_samples)],
        unreached=[name for name,row in copy_counts.items() if not row['attempts']],
        kernel_sha256=extent_inventory['kernel_sha256'],
        audit_sha256=extent_inventory['copy_transfer_audit_sha256'],
        scope='Every selected observed admission result and REP start/completion: preserved helper registers/captured arguments, actual candidate/count/DF, equal copied bytes, user/supervisor permissions, no observed physical overlap with counterparty/translation storage and unchanged boundary translation/control observations. Read-only GDB observation, not continuous check-to-copy stability, complete table/backing ownership, every possible path or production qualification')
'''
