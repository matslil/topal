"""Standalone source/ABI oracle; persistent-controller and VM proof is separate."""
import gdb

registers = ("rbx", "rbp", "r12", "r13", "r14", "r15")
returns = entries = codecs = 0
inputs = None


def register(name):
    return int(gdb.parse_and_eval("$" + name)) & ((1 << 64) - 1)


def words(address, count):
    raw = bytes(gdb.selected_inferior().read_memory(address, count * 8))
    return [int.from_bytes(raw[i:i + 8], "little") for i in range(0, len(raw), 8)]


def natural(value):
    address = int(value)
    sign, count = words(address, 2)
    assert sign == 0 and count <= 3, (sign, count)
    if count == 0:
        return 0
    raw = bytes(gdb.selected_inferior().read_memory(address + 16, count * 4))
    return int.from_bytes(raw, "little")


class Returned(gdb.FinishBreakpoint):
    def __init__(self, saved, stack, accepted, control, generation):
        super().__init__(internal=True)
        self.saved, self.stack = saved, stack
        self.accepted, self.control, self.generation = accepted, control, generation

    def stop(self):
        global returns
        assert self.saved == [register(name) for name in registers], "caller registers changed"
        assert register("rsp") == self.stack + 8, "caller stack changed"
        assert register("rax") == self.accepted, "actual Boolean result differs"
        control = words(self.control, 8)
        assert control == [0, 0, self.generation, 0, 0, 0, 0, 0], control
        returns += 1
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        global inputs
        control, base, count = register("rdi"), register("rcx"), register("r8")
        fields = words(control, 8)
        assert fields[1] == register("rsi") and fields[2] + 1 == register("rdx")
        inputs = (base, count, fields[4], fields[5])
        accepted = int(fields[5] >= 8 and fields[6] == 1 and base + count <= (1 << 63) - 1)
        Returned([register(name) for name in registers], register("rsp"), accepted,
                 control, register("rdx"))
        return False


class Entry(gdb.Breakpoint):
    def stop(self):
        global entries
        frame = gdb.newest_frame()
        assert frame.find_sal().symtab.fullname().endswith("file-position.t")
        assert natural(frame.read_var("base")) == inputs[0], (natural(frame.read_var("base")), inputs)
        assert natural(frame.read_var("transferred")) == inputs[1], (natural(frame.read_var("transferred")), inputs)
        view = words(int(frame.read_var("field")), 8)
        assert view[1:4] == [0, inputs[3], words(view[0], 8)[6]]
        assert words(view[0], 8)[4] == inputs[2]
        assert frame.older().name() == "topal_kernel_region_action_apply_v1"
        entries += 1
        return False


class Codec(gdb.Breakpoint):
    def stop(self):
        global codecs
        frame = gdb.newest_frame()
        assert frame.find_sal().symtab.fullname().endswith("support/word-bytes.t")
        assert natural(frame.read_var("value")) == inputs[0] + inputs[1]
        assert "advance" in frame.older().name()
        assert frame.older().older().name() == "topal_kernel_region_action_apply_v1"
        codecs += 1
        return False


gdb.execute("set python print-stack full")
gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set detach-on-fork on")
gdb.execute("set follow-fork-mode parent")
Enter("*topal_kernel_region_action_apply_v1", internal=True)
Entry(gdb.parse_and_eval("$region_action_entry").string(), internal=True)
Codec(gdb.parse_and_eval("$region_action_codec").string(), internal=True)
gdb.execute("run")
assert gdb.selected_inferior().pid == 0
assert (returns, entries, codecs) == (168, 168, 98), (returns, entries, codecs)
assert int(gdb.parse_and_eval("$_exitcode")) == 0
print("REGION_ACTION_DEBUG_OK returns=168 entries=168 codecs=98 caller=preserved")
