"""Independent GDB observations of the emitted private stack/return boundary."""
import gdb

observed = {'entry': 0, 'return': 0, 'fatal': 0, 'checkpoint_entry':0, 'checkpoint_return':0, 'complete_entry':0, 'complete_return':0}
failures = []
pending = None
walk = None
walked = {'call': 0, 'checkpoint':0, 'complete':0, 'abort': 0}
preserved = ['rbx','rbp','r12','r13','r14','r15']
bad_outcome = int(gdb.parse_and_eval('$tail_bad_outcome'))
controller_failure = int(gdb.parse_and_eval('$tail_controller_failure'))
native_hooks = int(gdb.parse_and_eval('$tail_native_hooks'))
source_fault = int(gdb.parse_and_eval('$tail_source_fault'))
completion_fault = int(gdb.parse_and_eval('$tail_completion_fault'))
fatal_max = 1 if bad_outcome or source_fault or completion_fault else 26 if controller_failure and native_hooks else 2 if controller_failure or not native_hooks else 0
return_max = 0 if bad_outcome or source_fault else 2 if controller_failure else 4
abort_walk = bool(fatal_max and not bad_outcome and source_fault < 9 and completion_fault not in [19,20] and (not controller_failure or native_hooks))
checkpoint_entry_max = 1 if source_fault == 10 else 0 if not native_hooks or bad_outcome or source_fault else 25 if controller_failure else 1
checkpoint_return_max = 0 if not checkpoint_entry_max or source_fault else 1 if controller_failure else 4
complete_entry_max = int(bool(native_hooks and not (controller_failure or bad_outcome or source_fault)))
complete_return_max = 4 if complete_entry_max and not completion_fault else 0

def value(expression):
    return int(gdb.parse_and_eval(expression))

def zeros(address, length):
    assert bytes(gdb.selected_inferior().read_memory(address, length)) == bytes(length)

class Caller(gdb.Breakpoint):
    def __init__(self, name, controller=False, checkpoint=False, complete=False):
        super().__init__(name)
        self.controller = controller
        self.checkpoint = checkpoint
        self.complete = complete

    def stop(self):
        global pending, walk
        try:
            private, stack = value('PrivateExtent'), value('SourceStack')
            if value('$rdi') != private or value('$rsi') != stack or value('topal_x86_tail_boundary_active_v1'):
                return False
            if (self.checkpoint or self.complete) and value('$rdx') != value('NativeHistory'):
                return False
            if self.complete and value('$rcx') != value('NativeOutput'):
                return False
            if not self.controller and bytes(gdb.selected_inferior().read_memory(private,64)) != bytes(64):
                return False
            pending = {'rsp':value('$rsp'), 'registers':{r:value('$'+r) for r in preserved}}
            print('TAIL_BOUNDARY_CALLER_REGISTERS', pending)
            self.enabled = self.controller or (self.checkpoint and bool(controller_failure))
            if self.complete and not walked['complete']:
                walk = 'complete'
                return True
            if self.checkpoint and not walked['checkpoint']:
                walk = 'checkpoint'
                return True
            if not self.controller and not self.checkpoint and not self.complete and not walked['call']:
                walk = 'call'
                return True
        except Exception as error:
            failures.append(str(error))
        return False

caller = Caller('topal_x86_tail_boundary_call_v1')
if controller_failure:
    Caller('topal_x86_tail_boundary_fail_v1', controller=True)
if checkpoint_entry_max:
    checkpoint_caller = Caller('topal_x86_tail_checkpoint_call_v1', checkpoint=True)
if complete_entry_max:
    complete_caller = Caller('topal_x86_tail_complete_call_v1', complete=True)

class AbortStart(gdb.Breakpoint):
    def stop(self):
        global walk
        if not walked['abort']:
            walk = 'abort'
            self.enabled = False
            return True
        return False

if abort_walk:
    AbortStart('topal_x86_tail_boundary_abort_v1')

class Case(gdb.Breakpoint):
    def stop(self):
        try:
            if value('fail_copy') in [1,2] and observed['fatal'] < 2:
                caller.enabled = True
        except Exception as error:
            failures.append(str(error))
        return False

Case('run_case')

class Probe(gdb.Breakpoint):
    def __init__(self, name, kind, maximum):
        super().__init__(name)
        self.kind, self.maximum = kind, maximum

    def stop(self):
        try:
            if observed[self.kind] >= self.maximum:
                self.enabled=False
                return False
            stack = value('SourceStack')
            private = value('PrivateExtent')
            if self.kind in ['entry','checkpoint_entry','complete_entry']:
                assert stack <= value('$rsp') < stack + 4096
                trace = gdb.execute('bt', to_string=True)
                expected = 'topal_x86_tail_complete_call_v1' if self.kind=='complete_entry' else 'topal_x86_tail_checkpoint_call_v1' if self.kind=='checkpoint_entry' else 'topal_x86_tail_boundary_call_v1'
                boundary = gdb.newest_frame().older()
                assert boundary is not None and boundary.type() == gdb.SIGTRAMP_FRAME, trace
                native_symbol = gdb.execute('info symbol ' + hex(boundary.pc()), to_string=True)
                assert native_symbol.split()[0] == expected and 'run_case' in trace, (native_symbol, trace)
                print('TAIL_BOUNDARY_DISCONTINUOUS_FRAME', expected, hex(boundary.pc()))
                if self.kind in ['checkpoint_entry','complete_entry']:
                    assert 'topal_tail_step_component_v1' not in trace, trace
                print('TAIL_BOUNDARY_SOURCE_BACKTRACE\n' + trace)
            else:
                assert pending is not None
                assert value('$rsp') == pending['rsp']
                for register in preserved:
                    assert value('$'+register) == pending['registers'][register], register
                assert not (stack <= value('$rsp') < stack + 4096)
                assert value('topal_x86_tail_boundary_active_v1') == 0
                zeros(stack, 4096)
                zeros(private, 65528 if self.kind in ['fatal','complete_return'] else 64)
                if self.kind == 'complete_return':
                    assert not value('NativeHistory->active') and not value('NativeHistory->poisoned')
                    assert value('NativeOutput->kind') in [2,3,4]
                for register in ['rcx','rdx','rsi','rdi','r8','r9','r10','r11']:
                    assert value('$' + register) == 0, register
                if self.kind in ['return','checkpoint_return']:
                    assert value('$rax') in [0,1]
                else:
                    assert value('$rax') == 0
                trace = gdb.execute('bt', to_string=True)
                assert 'run_case' in trace, trace
                print('TAIL_BOUNDARY_RECLAIMED_REGISTERS', self.kind,
                      {r:value('$'+r) for r in preserved + ['rax','rcx','rdx','rsi','rdi','r8','r9','r10','r11','rsp']})
                print('TAIL_BOUNDARY_' + self.kind.upper() + '_BACKTRACE\n' + trace)
            observed[self.kind] += 1
            if observed[self.kind] >= self.maximum:
                self.enabled = False
            elif self.kind == 'return':
                caller.enabled = True
            elif self.kind == 'checkpoint_return':
                checkpoint_caller.enabled=True
            elif self.kind == 'complete_return':
                complete_caller.enabled=True
        except Exception as error:
            failures.append(str(error))
            print('TAIL_BOUNDARY_PROBE_FAILURE', repr(error))
            self.enabled = False
        return False

Probe('*topal_tail_step_component_v1', 'entry', 1)
if return_max:
    Probe('topal_x86_tail_boundary_return_reclaimed_v1', 'return', return_max)
fatal_marker = ('topal_x86_tail_boundary_controller_reclaimed_v1' if controller_failure else
                'topal_x86_tail_complete_fatal_v1' if completion_fault in [19,20] else
                'topal_x86_tail_complete_abort_reclaimed_v1' if completion_fault else
                'topal_x86_tail_checkpoint_fatal_v1' if source_fault == 10 else
                'topal_x86_tail_boundary_return_fatal_v1' if bad_outcome or source_fault == 9 else
                'topal_x86_tail_boundary_abort_reclaimed_v1')
if fatal_max:
    Probe(fatal_marker, 'fatal', fatal_max)
    if native_hooks and controller_failure:
        Probe('topal_x86_tail_boundary_abort_reclaimed_v1','fatal',fatal_max)
if checkpoint_entry_max:
    Probe('*topal_tail_checkpoint_body_v1','checkpoint_entry',checkpoint_entry_max)
    if checkpoint_return_max:
        Probe('topal_x86_tail_checkpoint_reclaimed_v1','checkpoint_return',checkpoint_return_max)
if complete_entry_max:
    Probe('*topal_tail_complete_body_v1','complete_entry',complete_entry_max)
    if complete_return_max:
        Probe('topal_x86_tail_complete_reclaimed_v1','complete_return',complete_return_max)
def caller_registers(expected):
    older = gdb.newest_frame().older()
    assert older is not None
    for register in preserved:
        assert int(older.read_register(register)) == expected[register], register

def walk_call(role):
    entry = value({'call':'&topal_tail_step_component_v1', 'checkpoint':'&topal_tail_checkpoint_body_v1', 'complete':'&topal_tail_complete_body_v1'}[role])
    instructions = 0
    while value('$pc') != entry:
        assert instructions < 200
        caller_registers(pending['registers'])
        gdb.execute('stepi', to_string=True)
        instructions += 1
    # ENTRY is now the newest frame; unwind through the boundary to the owner.
    owner = gdb.newest_frame().older().older()
    assert owner is not None
    for register in preserved:
        assert int(owner.read_register(register)) == pending['registers'][register], register
    print({'call':'TAIL_BOUNDARY_CALL_CFI_INSTRUCTIONS','checkpoint':'TAIL_BOUNDARY_CHECKPOINT_CFI_INSTRUCTIONS','complete':'TAIL_BOUNDARY_COMPLETE_CFI_INSTRUCTIONS'}[role], instructions)

def walk_abort():
    initial = {r:value('$'+r) for r in preserved}
    clobbered = set()
    instructions = 0
    saved_sp = pending['rsp'] - 48
    while value('$rsp') != saved_sp:
        assert instructions < 20
        older = gdb.newest_frame().older()
        assert older is not None
        for register in preserved:
            if register in clobbered:
                try:
                    recovered = int(older.read_register(register))
                except (gdb.error, ValueError):
                    pass
                else:
                    raise AssertionError('clobbered register fabricated: ' + register + '=' + str(recovered))
            else:
                assert int(older.read_register(register)) == initial[register], register
        instruction = gdb.selected_frame().architecture().disassemble(value('$pc'), count=1)[0]['asm']
        if instruction.startswith('mov'):
            destination = instruction.rsplit(',', 1)[-1].strip()
            if destination in ['%r12', '%r13', '%r14']:
                clobbered.add(destination[1:])
        gdb.execute('stepi', to_string=True)
        instructions += 1
    assert clobbered == {'r12', 'r13', 'r14'}, clobbered
    caller_registers(pending['registers'])
    assert 'run_case' in gdb.execute('bt', to_string=True)
    print('TAIL_BOUNDARY_ABORT_CFI_INSTRUCTIONS', instructions)

gdb.execute('set disassembly-flavor att')
gdb.execute('run')
while gdb.selected_inferior().pid:
    assert walk in walked, walk
    if walk in ['call','checkpoint','complete']:
        walk_call(walk)
    else:
        walk_abort()
    walked[walk] += 1
    walk = None
    gdb.execute('continue')
assert not failures, failures
assert observed == {'entry':1, 'return':return_max, 'fatal':fatal_max,
                    'checkpoint_entry':checkpoint_entry_max,'checkpoint_return':checkpoint_return_max,
                    'complete_entry':complete_entry_max,'complete_return':complete_return_max}, observed
assert walked == {'call':1, 'checkpoint':int(bool(checkpoint_entry_max)), 'complete':complete_entry_max, 'abort':int(abort_walk)}, walked
print('TAIL_BOUNDARY_GDB_PASS', observed, walked)
