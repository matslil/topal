"""GDB-side ordinary finite policy execution and prepublication refusal."""
import gdb
import json
from pathlib import Path
import traceback

MASK = (1 << 64) - 1

def install(config):
    calls, frames = [], []
    active = None
    context = None
    table = int(gdb.parse_and_eval("&'topal_x86_static_process_descriptor_table'"))
    entry = int(gdb.parse_and_eval("&'topal_kernel_file_open_policy_v1'"))
    def reg(name): return int(gdb.parse_and_eval('$' + name)) & MASK
    def memory(address, size): return bytes(gdb.selected_inferior().read_memory(address, size))
    def word(address): return int.from_bytes(memory(address, 8), 'little')
    def state(): return {name:reg(name) for name in ('rbx','rbp','r12','r13','r14','r15')}
    def simd():
        return [[int(gdb.parse_and_eval('$xmm%d.v2_int64[%d]' % (i,j))) for j in range(2)] for i in range(16)]
    def expected(mask):
        w,d,r,m,e,c,x,t = [bool(mask & (1 << bit)) for bit in range(8)]
        if not e:
            if not c: return 4
            if not m: return 3
            return 1 if r else 6
        if c and x: return 5
        if r and not d: return 1
        if d and (w or c or t): return 2
        if not d and not m and (w or t): return 3
        return 7 if not d and t else 0
    class Checked(gdb.Breakpoint):
        def stop(self):
            try: return self.inspect()
            except BaseException:
                Path(config['failed']).write_text(traceback.format_exc())
                return True
    class Refused(Checked):
        def inspect(self):
            store = memory(table + 0x43000, 0x9000)
            # Latest layout retains the actual initial directory mode even
            # before any file exists. Every other byte must remain initial.
            initial = bytearray(0x9000)
            initial[:8] = (7).to_bytes(8, 'little')
            initial[32:40] = (0o41777).to_bytes(8, 'little')
            assert store == bytes(initial)
            for bank in (320,0x3d000):
                assert all(word(table+bank+64*i)!=7 for i in range(16))
            assert reg('rip') == config['refusal_pc']
            assert memory(reg('rip'), 4) == bytes.fromhex('faf4ebfd')
            Path(config['result']).write_text(json.dumps(dict(
                result='expected-wrong-create-plan-refusal', calls=calls, source_helper_frames=frames,
                source_returned_open_for_create=True, native_branch_reached_fatal=True,
                object_counter_unchanged=True, object_page_and_payload_store_unchanged=True,
                no_owned_file_description_published=True, terminal_cleanup_claimed=False), indent=2))
            self.enabled=False
            return True
    class Returned(Checked):
        def inspect(self):
            nonlocal active
            assert active is not None
            assert reg('rsp') == active['stack'] + 8 and state() == active['saved']
            assert simd() == active['simd']
            assert word(active['context'] + 952) == entry
            value, facts = reg('rax'), active['facts']
            calls.append(dict(facts=facts, choice=value, stack_aligned=True,
                callee_saved_preserved=True, simd_preserved=True))
            if config.get('wrong') and facts == 41:
                assert value == 0 and expected(facts) == 6
                # The owner resumes through its stack-restoring shim, compares
                # the CreateDescription code and branches to its actual fatal.
                code = memory(reg('rip'), 96)
                comparison = bytes.fromhex('4881f8060000000f85')
                at = code.index(comparison)
                # Named creation has later capacity/parent checks containing
                # the same immediate. Bind the first comparison immediately
                # after the invocation shim restores its caller stack.
                assert code[:at].endswith(bytes.fromhex('4c89dc')), code.hex()
                displacement = int.from_bytes(code[at+9:at+13], 'little', signed=True)
                target = reg('rip') + at + 13 + displacement
                config['refusal_pc'] = target
                Refused('*' + hex(target), type=gdb.BP_HARDWARE_BREAKPOINT, internal=True)
            else:
                assert value == expected(facts), (facts,value,expected(facts))
                entering.enabled=True
            active=None
            self.enabled=False
            return False
    class Enter(Checked):
        def inspect(self):
            nonlocal active, context
            context = reg('gs_base')
            assert active is None and reg('rsp') % 16 == 8
            assert word(reg('gs_base')+952) == entry
            assert 0 <= reg('rdi') < 256
            active=dict(facts=reg('rdi'), stack=reg('rsp'), saved=state(), simd=simd(), context=reg('gs_base'))
            Returned('*'+hex(word(reg('rsp'))),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
            self.enabled=False
            return False
    class Helper(Checked):
        def inspect(self):
            assert active is not None and active['facts'] == 41
            trace=gdb.execute('bt',to_string=True)
            assert 'file-open-access.t' in trace and 'topal_kernel_file_open_policy_v1' in trace
            assert trace.count('file-open-access.t') >= 3, trace
            frames.append(trace)
            self.enabled=False
            return False
    class Terminal(Checked):
        def inspect(self):
            assert active is None and calls and frames
            assert not config.get('wrong')
            assert any(c['facts']==41 and c['choice']==6 for c in calls)
            assert {0,1,2,3,6}.issubset(c['choice'] for c in calls)
            assert not any(memory(table,0x4c000))
            assert context is not None and word(context+952)==0
            Path(config['result']).write_text(json.dumps(dict(result='pass', calls=calls,
                source_helper_frames=frames, binding_erased=True, complete_process_store_erased=True,
                actual_temporary_create_plan=True, all_native_calls_match_independent_disposition=True),indent=2))
            self.enabled=False
            return True
    entering=Enter('*'+hex(entry),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    Helper('*'+hex(int(gdb.parse_and_eval("&'"+config['helper']+"'"))),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    Terminal('topal_x86_systems_static_image_process_restore_entry',type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
