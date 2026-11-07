"""Physical component evidence; no owning-kernel controller/cleanup claim.

The independent caller executes one actual link, move, replace and exchange.
Inspect every source codec store and the native ABI boundary of the bound object.
"""

import gdb


prefix = "topal.region-instance.namespace-metadata."
export = "topal_kernel_region_frame_namespace_metadata_apply_v1"
registers = ("rbx", "rbp", "r12", "r13", "r14", "r15")
returns = 0
stores = 0
operations = []


def register(name):
    return int(gdb.parse_and_eval("$" + name))


def vector_registers():
    return tuple(str(gdb.parse_and_eval(f"$xmm{index}.uint128")) for index in range(16))


def natural(address):
    inferior = gdb.selected_inferior()
    header = bytes(inferior.read_memory(address, 16))
    sign = int.from_bytes(header[:8], "little")
    length = int.from_bytes(header[8:], "little")
    assert sign == 0 and length <= 2, (sign, length)
    if length == 0:
        return 0
    return int.from_bytes(bytes(inferior.read_memory(address + 16, length * 4)), "little")


class Returned(gdb.FinishBreakpoint):
    def __init__(self):
        super().__init__(internal=True)
        self.saved = tuple(register(name) for name in registers)
        self.vectors = vector_registers()
        self.stack = register("rsp")

    def stop(self):
        global returns
        assert tuple(register(name) for name in registers) == self.saved, "callee-saved GPRs"
        assert vector_registers() == self.vectors, "GPR-only source changed SIMD registers"
        assert register("rsp") == self.stack + 8, "caller stack was not restored"
        assert register("rax") == 1, "admitted source failed"
        for field in ("code", "owner", "active", "base", "count", "writable", "issued"):
            assert int(gdb.parse_and_eval("control." + field)) == 0, field
        returns += 1
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        assert register("rdi") == int(gdb.parse_and_eval("&control")), "actual caller control"
        assert register("r8") == 2, "unchanged two-Nat frame ABI"
        raw = bytes(gdb.selected_inferior().read_memory(register("rcx"), 16))
        operations.append(int.from_bytes(raw[:8], "little"))
        assert int.from_bytes(raw[8:], "little") == 2**64 - 1
        Returned()
        return False


class Store(gdb.Breakpoint):
    def stop(self):
        global stores
        assert int(gdb.parse_and_eval("control.active")) == 1
        assert int(gdb.parse_and_eval("control.count")) == 4096
        assert int(gdb.parse_and_eval("control.writable")) == 1
        offset = natural(register("rsi"))
        width = register("rcx")
        value = natural(register("rdx"))
        assert width == 8 and offset + width <= 4096, (offset, width)
        assert offset + width <= 3072 or offset >= 3584, "source wrote immutable staged input"
        assert value <= 2**64 - 1
        stores += 1
        return False


gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set disable-randomization off")
Enter("*" + export, internal=True)
Store("*'" + prefix + "primitive.unsigned.store'", internal=True)
gdb.execute("run")
assert gdb.selected_inferior().pid == 0
assert int(gdb.parse_and_eval("$_exitcode")) == 0
assert operations == [0, 1, 2, 3], operations
assert returns == 4, returns
assert stores == 121, stores
print("NAMESPACE_ACTION_DEBUG_PASS returns=4 stores=121 staged-input-stores=0 "
      "preserved=rbx,rbp,r12,r13,r14,r15,rsp,xmm0-xmm15")
