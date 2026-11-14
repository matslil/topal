"""Observe real registered controller targets and effects; never assign state."""

PROGRAM = r'''
import hashlib

callback_active = None
callback_registered = None
callback_counts = {}
callback_samples = {}
callback_live_sections = {}
callback_callee_registers = ['rbx','rbp','r12','r13','r14','r15']

def callback_translation(pointer):
    assert not (reg('cr4') & (1<<12))
    table = reg('cr3') & 0x000ffffffffff000
    path = []
    user, writable, noexecute = True, True, False
    for shift in [39,30,21,12]:
        assert table % 4096 == 0 and table < 0x40000000
        entry = word(table + ((pointer>>shift)&511)*8)
        assert entry & 1
        user = user and bool(entry&4)
        writable = writable and bool(entry&2)
        noexecute = noexecute or bool(entry&(1<<63))
        path.append(dict(shift=shift,table=table,entry=entry))
        if shift == 12 or (shift in [30,21] and entry&128):
            physical = (entry & (0x000fffffffffffff & ~((1<<shift)-1))) | (pointer&((1<<shift)-1))
            assert physical == pointer and not user and not noexecute
            return dict(path=path,physical=physical,effective_user=user,
                        effective_write=writable,effective_execute=not noexecute)
        table = entry & 0x000ffffffffff000
    raise AssertionError('missing translation leaf')

class CallbackRegistration(Checked):
    def observe(self):
        global callback_registered
        self.enabled = False
        assert callback_registered is None and reg('cs')&3 == 0
        assert word(reg('rsp')+8) == context
        rows = []
        for row in extent_inventory['callback_bindings']['callbacks']:
            actual = word(reg('rsp')+row['entry_stack_offset'])
            assert actual == row['target'] == address(row['native_export'])
            rows.append(dict(gs_slot=row['gs_slot'],argument=row['argument'],target=actual))
        callback_registered = rows
        return False

class CallbackReturned(Checked):
    def __init__(self, site):
        super().__init__('*'+hex(site['return_address']),internal=True)
        self.site = site

    def observe(self):
        global callback_active
        assert callback_active is not None and callback_active['site']==self.site['offset']
        before = callback_active
        assert reg('cs')&3 == 0 and reg('cr0')&(1<<16)
        assert reg('cr3') == before['cr3'] and reg('rsp') == before['rsp']
        assert [reg(r) for r in callback_callee_registers] == before['callee_registers']
        assert memory(context+296,64) == before['captured']
        assert memory(context+952,56) == before['registry']
        assert memory(before['table'],8192) == before['table_bytes']
        assert memory(context+104,1)[0] == before['phase']
        slot = self.site['gs_slot']
        callback_counts[self.site['offset']] = callback_counts.get(self.site['offset'],0)+1
        if self.site['offset'] not in callback_samples:
            callback_samples[self.site['offset']] = dict(
                offset=self.site['offset'],gs_slot=slot,target=before['target'],
                arguments=before['arguments'],result=reg('rax'),
                supervisor_translation=before['translation'],
                callee_registers_preserved=True,captured_arguments_preserved=True,
                callback_registry_preserved=True,complete_process_table_preserved=True)
        callback_active = None
        return False

class CallbackEntered(Checked):
    def __init__(self, site, row):
        super().__init__('*'+hex(site['address']),internal=True)
        self.site, self.row = site, row

    def observe(self):
        global callback_active
        assert callback_registered is not None and callback_active is None
        assert reg('cs')&3 == 0 and reg('cr0')&(1<<16) and not (reg('eflags')&0x200)
        row, site = self.row, self.site
        assert reg('rax') == word(context+row['gs_slot']) == row['target']
        assert reg('rsp') % 16 == 0
        translation = callback_translation(reg('rax'))
        if row['gs_slot'] not in callback_live_sections:
            checked = []
            for section in row['sections']:
                actual = hashlib.sha256(memory(section['address'],section['bytes'])).hexdigest()
                assert actual == section['linked_sha256']
                checked.append(dict(address=section['address'],bytes=section['bytes'],sha256=actual))
            callback_live_sections[row['gs_slot']] = checked
        if row['gs_slot'] == 952:
            assert reg('rdi') < 32
        elif row['gs_slot'] == 984:
            assert reg('rdi') <= 0xffffffff
        elif row['gs_slot'] == 976:
            table = word(context+928)
            assert word(table)==1 and word(table+48)==1 and word(table+24)>=64
            assert reg('rdi') == word(table+16)
            assert [reg(r) for r in ['rsi','rdx','rcx']] == [64,62,16]
            assert memory(reg('rdi'),64).hex() == extent_inventory['callback_header_hex']
        elif row['gs_slot'] == 992:
            assert reg('rsi') == word(context+1000) and reg('rsi')
            assert reg('rdx') == word(context+256) == context and reg('rcx') == row['target']
        table = word(context+928)
        assert table
        callback_active = dict(site=site['offset'],target=reg('rax'),cr3=reg('cr3'),rsp=reg('rsp'),
            callee_registers=[reg(r) for r in callback_callee_registers],
            arguments=[reg(r) for r in ['rdi','rsi','rdx','rcx','r8','r9']],
            captured=memory(context+296,64),registry=memory(context+952,56),
            table=table,table_bytes=memory(table,8192),phase=memory(context+104,1)[0],
            translation=translation)
        return False

def install_callback_observation():
    if extent_inventory is None:
        return
    bindings = extent_inventory['callback_bindings']
    assert bindings['passed'] and bindings['kernel_sha256'] == extent_inventory['kernel_sha256']
    rows = {row['gs_slot']:row for row in bindings['callbacks']}
    CallbackRegistration('*'+hex(extent_inventory['address']),internal=True)
    for site in bindings['sites']:
        CallbackEntered(site,rows[site['gs_slot']])
        CallbackReturned(site)

def callback_coverage_complete():
    if extent_inventory is None:
        return
    assert callback_registered is not None and callback_active is None and callback_counts
    sites = extent_inventory['callback_bindings']['sites']
    observed['callback_observation'] = dict(passed=True,registered=callback_registered,
        counts=[dict(offset=offset,count=callback_counts[offset]) for offset in sorted(callback_counts)],
        samples=[callback_samples[offset] for offset in sorted(callback_samples)],
        live_sections=[dict(gs_slot=slot,sections=callback_live_sections[slot]) for slot in sorted(callback_live_sections)],
        unreached=[site for site in sites if site['offset'] not in callback_counts],
        scope='Actual callback registration/targets, supervisor-executable mapping, first-role live bytes and observed ABI/private-state preservation; not every callee path, hardware read-only protection, arbitrary pointer authority, complete memory effects or lifetime proof')
'''
