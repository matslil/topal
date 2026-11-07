"""Actual native helper boundaries; component evidence, not a live kernel grant.

Observe all four operations with normal, false-after-store or wrong-association
source. Check the complete private snapshot, actual unchanged owner on prepare,
non-destructive overlap refusal and physical erasure/preservation on CHECK.
"""

import gdb


preserved = ("rbx", "rbp", "r12", "r13", "r14", "r15")
fault = int(gdb.parse_and_eval("$namespace_fault"))
preparations = 0
completions = 0
operations = []
expected_address = 0
latest_expected = b""


def reg(name):
    return int(gdb.parse_and_eval("$" + name)) & ((1 << 64) - 1)


def memory(address, count=4096):
    return bytes(gdb.selected_inferior().read_memory(address, count))


def vectors():
    return tuple(str(gdb.parse_and_eval(f"$xmm{i}.uint128")) for i in range(16))


def word(data, offset):
    return int.from_bytes(data[offset:offset + 8], "little")


def desired(before, operation, seconds):
    data = bytearray(before)
    source, old, victim, target, length, identity, victim_id, links = (
        word(before, 3072 + 8 * i) for i in range(8)
    )

    def put(offset, value):
        data[offset:offset + 8] = value.to_bytes(8, "little")

    if operation <= 1:
        start = 1536 + target * 256
        data[start:start + 256] = before[3200:3456]
        put(72 + target * 16, length)
        if operation == 0:
            put(64 + target * 16, identity)
            put(256 + source * 128 + 56, links + 1)
            put(256 + source * 128 + 64, 0)
    elif operation == 2:
        put(64 + target * 16, identity)
        data[64 + old * 16:80 + old * 16] = bytes(16)
        data[1536 + old * 256:1536 + (old + 1) * 256] = bytes(256)
        put(256 + victim * 128 + 56, word(before, 256 + victim * 128 + 56) - 1)
        put(256 + victim * 128 + 48, seconds)
    else:
        put(64 + old * 16, victim_id)
        put(64 + target * 16, identity)
        put(256 + victim * 128 + 48, seconds)
    put(256 + source * 128 + 48, seconds)
    put(48, seconds)
    put(56, seconds)
    return bytes(data)


class Returned(gdb.FinishBreakpoint):
    def __init__(self, kind, owner, before, snapshot, result):
        super().__init__(internal=True)
        self.kind, self.owner, self.before = kind, owner, before
        self.snapshot, self.result = snapshot, result
        self.saved = tuple(reg(name) for name in preserved)
        self.simd, self.stack = vectors(), reg("rsp")

    def stop(self):
        global preparations, completions
        assert tuple(reg(name) for name in preserved) == self.saved, "callee-saved GPR changes"
        assert vectors() == self.simd, "native helper changed SIMD"
        assert reg("rsp") == self.stack + 8, "native helper did not restore stack"
        assert reg("rax") == self.result, (self.kind, reg("rax"), self.result)
        if self.kind == "prepare":
            assert memory(self.owner) == self.before, "prepare wrote actual owner"
            assert memory(expected_address) == self.snapshot, "comparison snapshot differs"
            preparations += 1
        else:
            cleaned = bytearray(self.before)
            cleaned[3072:3584] = bytes(512)
            assert memory(self.owner) == cleaned, "CHECK changed persistent state or missed input cleanup"
            assert memory(expected_address) == bytes(4096), "private comparison snapshot not erased"
            completions += 1
        return False


class Prepare(gdb.Breakpoint):
    def stop(self):
        global expected_address, latest_expected
        expected_address = int(gdb.parse_and_eval("&topal_x86_systems_namespace_expected"))
        owner, operation, seconds = reg("rdi"), reg("rsi"), reg("rdx")
        before, snapshot = memory(owner), memory(expected_address)
        if snapshot == bytes(4096):
            operations.append(operation)
            latest_expected = desired(before, operation, seconds)
            Returned("prepare", owner, before, latest_expected, 1)
        else:
            assert snapshot == latest_expected
            Returned("prepare", owner, before, snapshot, 0)
        return False


class Check(gdb.Breakpoint):
    def stop(self):
        owner, result = reg("rdi"), reg("rsi")
        before = memory(owner)
        assert memory(expected_address) == latest_expected
        actual = bytearray(latest_expected)
        if fault == 2:
            actual[64:72] = (999).to_bytes(8, "little")
        assert before == actual, "actual source stores not the expected normal/fault effect"
        assert result == (0 if fault == 1 else 1), "source result boundary"
        Returned("check", owner, before, b"", 1 if fault == 0 else 0)
        return False


gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set disable-randomization off")
Prepare("*topal_x86_systems_namespace_prepare_v1", internal=True)
Check("*topal_x86_systems_namespace_check_v1", internal=True)
gdb.execute("run")
assert gdb.selected_inferior().pid == 0
assert int(gdb.parse_and_eval("$_exitcode")) == 0
assert operations == [0, 1, 2, 3], operations
assert preparations == 8 and completions == 4, (preparations, completions)
print("NAMESPACE_NATIVE_ORACLE_DEBUG_PASS prepare=8 check=4 erased=4096+512 "
      "preserved=rbx,rbp,r12,r13,r14,r15,rsp,xmm0-xmm15 "
      f"fault={fault}")
