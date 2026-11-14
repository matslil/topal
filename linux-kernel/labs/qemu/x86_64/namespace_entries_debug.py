"""Observe the actual read-only metadata leaf, including its retired spills.

Hosted component evidence only: this grants no filesystem authority and does
not qualify a V6 kernel entry path or a complete source/controller stack.
"""

import json

import gdb


preserved = ("rbx", "rcx", "rdx", "rsi", "rdi", "rbp",
             "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15")
observations = []


def reg(name):
    return int(gdb.parse_and_eval("$" + name)) & ((1 << 64) - 1)


def memory(address, count):
    return bytes(gdb.selected_inferior().read_memory(address, count))


def vectors():
    return tuple(str(gdb.parse_and_eval(f"$xmm{i}.uint128")) for i in range(16))


class Returned(gdb.FinishBreakpoint):
    def __init__(self):
        super().__init__(internal=True)
        self.saved = tuple(reg(name) for name in preserved)
        self.simd, self.stack = vectors(), reg("rsp")
        self.owner = reg("rdi")
        self.before = memory(self.owner, 4096)
        self.return_word = memory(self.stack, 8)

    def stop(self):
        assert tuple(reg(name) for name in preserved) == self.saved, "changed caller GPR"
        assert vectors() == self.simd, "changed SIMD"
        assert reg("rsp") == self.stack + 8, "unrestored stack"
        assert memory(self.owner, 4096) == self.before, "changed read-only metadata"
        assert memory(self.stack - 64, 64) == bytes(64), "retained popped register spill"
        assert memory(self.stack, 8) == self.return_word, "changed caller return word"
        expected = (1, 0)[len(observations)]
        assert reg("rax") == expected, (reg("rax"), expected)
        observations.append(dict(result=expected, retired_spill_bytes=64,
                                 metadata_bytes_unchanged=4096,
                                 preserved_gprs=list(preserved),
                                 preserved_simd="xmm0-xmm15",
                                 stack_restored=True, caller_return_word_preserved=True))
        return False


class Entry(gdb.Breakpoint):
    def stop(self):
        # Make every caller-saved spill carry a distinct nonzero value. DI is
        # the real guarded page supplied by the C client and remains intact.
        for index, name in enumerate(("rcx", "rdx", "rsi", "r8", "r9", "r10", "r11")):
            gdb.execute(f"set ${name} = {0x123456789ABCDE00 + index}", to_string=True)
        Returned()
        return False


gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set disable-randomization off")
Entry("*topal_namespace_metadata_check_v1", internal=True)
gdb.execute("run")
assert gdb.selected_inferior().pid == 0
assert int(gdb.parse_and_eval("$_exitcode")) == 0
assert len(observations) == 2, observations
print("NAMESPACE_ENTRIES_DEBUG_RECORD " + json.dumps(observations, sort_keys=True))
print("NAMESPACE_ENTRIES_NATIVE_DEBUG_PASS checks=2 retired-spills=64 "
      "metadata=4096 preserved=all-other-gprs,rsp,xmm0-xmm15")
