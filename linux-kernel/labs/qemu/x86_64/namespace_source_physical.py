"""Observe ordinary source stores and its real syscall caller without injection."""
import hashlib
import struct

import gdb

REGISTERS = ('rbx','rbp','r12','r13','r14','r15','rdi','rsi','rdx','rcx','r8','r9','r10','r11')


def reg(name):
    return int(gdb.parse_and_eval('$'+name)) & ((1 << 64)-1)


def memory(start,size):
    return bytes(gdb.selected_inferior().read_memory(start,size))


def words(start,count):
    return struct.unpack('<'+'Q'*count,memory(start,count*8))


def vectors():
    return tuple(str(gdb.parse_and_eval(f'$xmm{i}.uint128')) for i in range(16))


def desired(before,operation,seconds):
    page = bytearray(before)
    obj,old,victim,target,length,identity,victim_identity,links = struct.unpack_from('<8Q',before,3072)
    assert obj < 4 and old <= 4 and target < 4 and 0 < length <= 255
    assert operation in (0,1,2,3)

    def write(offset,value):
        struct.pack_into('<Q',page,offset,value)

    if operation == 0:
        page[1536+256*target:1792+256*target] = before[3200:3456]
        write(64+16*target,identity)
        write(72+16*target,length)
        write(312+128*obj,links+1)
        write(320+128*obj,0)
    elif operation == 1:
        assert target == old
        page[1536+256*old:1792+256*old] = before[3200:3456]
        write(72+16*old,length)
    elif operation == 2:
        assert victim < 4 and old < 4
        write(64+16*target,identity)
        page[64+16*old:80+16*old] = bytes(16)
        page[1536+256*old:1792+256*old] = bytes(256)
        write(312+128*victim,struct.unpack_from('<Q',before,312+128*victim)[0]-1)
        write(304+128*victim,seconds)
    else:
        assert victim < 4 and old < 4
        write(64+16*old,victim_identity)
        write(64+16*target,identity)
        write(304+128*victim,seconds)
    write(304+128*obj,seconds)
    write(48,seconds)
    write(56,seconds)
    return bytes(page)


class LoanObserver:
    def __init__(self,parent):
        self.parent = parent
        self.binding = parent.binding
        self.owner = parent.owner
        self.metadata = self.owner+parent.geometry['metadata_offset']
        self.storage = {s['symbol'].rsplit('_',1)[-1]:s for s in parent.storage}
        self.control = self.storage['control']['address']
        self.arena = self.storage['arena']['address']
        self.history = self.storage['history']['address']
        self.current = None
        self.kernel_context = None
        self.verified = False
        self.fault_mode = parent.config.get('fault_mode')
        assert self.fault_mode in (None, 'source-false', 'source-wrong-association', 'runtime-abort')
        observer = self

        class Source(gdb.Breakpoint):
            def stop(self):
                try:
                    current = observer.current
                    assert current and current['source_entries'] == 0
                    current['source_entries'] += 1
                    generation = current['generation']
                    assert (reg('rdi'),reg('rsi'),reg('rdx'),reg('rcx'),reg('r8')) == (
                        observer.control,observer.owner,generation,current['frame'],2)
                    assert words(observer.storage['recovery']['address'],4) == (
                        current['frame'],generation,current['operation'],current['seconds'])
                    assert words(observer.history,1) == (generation,)
                    assert memory(observer.storage['expected']['address'],4096) == current['desired']
                    assert memory(observer.owner,parent.owner_bytes) == current['before']
                    observer.watch_return('source')
                    return False
                except BaseException:
                    return parent.failed()

        class Store(gdb.Breakpoint):
            def stop(self):
                try:
                    current = observer.current
                    assert current and current['source_entries'] == 1
                    assert words(observer.control+24,2) == (1,observer.metadata)
                    offset,value = observer.natural(reg('rsi')),observer.natural(reg('rdx'))
                    assert reg('rcx') == 8 and offset % 8 == 0 and 0 <= offset <= 4088
                    assert offset < 3072 or offset >= 3584
                    observer.watch_store_return(offset,value)
                    return False
                except BaseException:
                    return parent.failed()

        export = parent.config['artifact']['namespace_library']['native_export']
        Source("*'"+export+"'",internal=True)
        Store("*'topal.region-instance.namespace-metadata.primitive.unsigned.store'",internal=True)

    def natural(self,pointer):
        cursor = words(self.arena+8,1)[0]
        extents = [(self.arena+32,self.arena+cursor)]
        extents += [(s['address'],s['address']+s['bytes']) for s in self.binding['source_data']]
        assert any(a <= pointer and pointer+16 <= b for a,b in extents)
        sign,count = words(pointer,2)
        assert sign == 0 and count <= 2
        assert any(a <= pointer and pointer+16+count*4 <= b for a,b in extents)
        return int.from_bytes(memory(pointer+16,count*4),'little') if count else 0

    def begin(self):
        assert self.current is None
        if not self.verified:
            for bound in self.binding['code']+self.binding['source_code']+self.binding['source_data']+[self.binding['controller_code']]:
                assert hashlib.sha256(memory(bound['address'],bound['bytes'])).hexdigest() == bound['sha256'],bound['symbol']
            self.verified = True
        assert not reg('eflags') & 0x200
        self.kernel_context = reg('gs_base')
        assert words(self.kernel_context+928,1) == (self.owner,)
        entry = self.stack()
        stack = reg('rsp')
        assert stack == entry-16, ('owning and APPLY return words',stack,entry)
        frame = stack-176
        before = memory(self.owner,self.parent.owner_bytes)
        operation,seconds = reg('rdx'),reg('rcx')
        page = desired(memory(self.metadata,4096),operation,seconds)
        self.current = dict(stack=stack,frame=frame,operation=operation,seconds=seconds,
            generation=words(self.history,1)[0]+1,before=before,replayed=bytearray(before),
            desired=page,source_entries=0,stores=[],source_returned=False)
        self.watch_return('owner')

    def watch_store_return(self,offset,value):
        observer = self

        class Stored(gdb.FinishBreakpoint):
            def stop(self):
                try:
                    assert reg('rax') == 1
                    assert words(observer.metadata+offset,1) == (value,)
                    current = observer.current
                    current['stores'].append((offset,value))
                    struct.pack_into('<Q',current['replayed'],observer.metadata-observer.owner+offset,value)
                    assert memory(observer.owner,observer.parent.owner_bytes) == bytes(current['replayed'])
                    if observer.fault_mode == 'runtime-abort' and len(current['stores']) == 1:
                        # Controlled test-only exhaustion of the actual source
                        # arena: its next allocator call must take its own
                        # linked noreturn recovery hook, after a real store.
                        gdb.selected_inferior().write_memory(observer.arena+8,
                            struct.pack('<Q',observer.storage['arena']['bytes']))
                        current['allocator_fault_after_store'] = True
                    return False
                except BaseException:
                    return observer.parent.failed()

        Stored(internal=True)

    def watch_return(self,kind):
        observer = self
        stack = reg('rsp')
        registers = REGISTERS if kind == 'owner' else REGISTERS[:6]
        saved = [reg(name) for name in registers]
        simd = vectors()

        class Returned(gdb.Breakpoint):
            def stop(self):
                try:
                    self.enabled = False
                    current = observer.current
                    expected_result = 0 if kind == 'source' and observer.fault_mode == 'source-false' else 1
                    assert reg('rsp') == stack+8 and reg('rax') == expected_result
                    assert [reg(name) for name in registers] == saved and vectors() == simd
                    assert words(observer.history,1) == (current['generation'],)
                    if kind == 'source':
                        assert memory(observer.owner,observer.parent.owner_bytes) == bytes(current['replayed'])
                        expected_page = bytearray(current['desired'])
                        if observer.fault_mode == 'source-wrong-association':
                            struct.pack_into('<Q',expected_page,64,999)
                        assert memory(observer.metadata,4096) == bytes(expected_page)
                        assert words(current['frame'],2) == (current['operation'],current['seconds'])
                        assert words(observer.control,8) == (0,0,current['generation'],0,0,0,0,0)
                        current['source_returned'] = True
                        current['source_result'] = expected_result
                    else:
                        assert observer.fault_mode is None, 'failed source resumed its owning adapter'
                        assert current['source_returned'] and current['source_entries'] == 1
                        expected = bytearray(current['before'])
                        at = observer.metadata-observer.owner
                        expected[at:at+4096] = current['desired']
                        expected[at+3072:at+3584] = bytes(512)
                        assert memory(observer.owner,observer.parent.owner_bytes) == bytes(expected)
                        start = current['frame']-4096
                        assert memory(start,4096+176) == bytes(4096+176)
                        observer.parent.current['source_effect'] = dict(operation=current['operation'],
                            seconds=current['seconds'],generation=current['generation'],
                            ordinary_entries=1,source_stores=current['stores'],
                            exact_complete_owner_delta=True,gprs_and_simd_preserved=True,
                            source_frame_and_retired_stack_erased=4096+176)
                        observer.current = None
                    return False
                except BaseException:
                    return observer.parent.failed()

        Returned('*'+hex(words(stack,1)[0]),internal=True)

    def stack(self):
        retirement = self.parent.geometry['admission_retirement']
        pointer = words(self.kernel_context+retirement['kernel_stack_context_offset'],1)[0]
        assert retirement['minimum_stack_pointer'] <= pointer <= retirement['maximum_stack_pointer']
        assert pointer % retirement['stack_alignment'] == 0
        return pointer

    def retired_stack(self):
        assert self.current is None and self.kernel_context is not None
        count = self.parent.geometry['admission_retirement']['retired_stack_bytes']
        assert memory(self.stack()-count,count) == bytes(count)
        return count
