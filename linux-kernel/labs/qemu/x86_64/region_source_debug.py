"""GDB oracle for real source frames and the independent native caller ABI."""

import gdb


returns = 0
helpers = 0
registers = ("rbx", "rbp", "r12", "r13", "r14", "r15")


def register(name):
    return int(gdb.parse_and_eval("$" + name))


class Returned(gdb.FinishBreakpoint):
    def __init__(self, saved, stack):
        super().__init__(internal=True)
        self.saved = saved
        self.stack = stack

    def stop(self):
        global returns
        assert self.saved == [register(name) for name in registers], "caller registers changed"
        assert register("rsp") == self.stack + 8, "caller stack was not restored"
        returns += 1
        return False


class Enter(gdb.Breakpoint):
    def stop(self):
        Returned([register(name) for name in registers], register("rsp"))
        return False


class Helper(gdb.Breakpoint):
    def stop(self):
        global helpers
        frame = gdb.newest_frame()
        assert frame.find_sal().symtab.fullname().endswith("support/bytes.t")
        region = frame.read_var("region")
        assert "KernelByteRegion" in str(region.type), str(region.type)
        assert int(region) != 0
        assert int(frame.read_var("source")) != 0
        assert int(frame.read_var("destination")) != 0
        caller = frame.older()
        assert caller is not None and "apply" in caller.name(), caller
        assert caller.older() is not None and caller.older().name() == "topal_kernel_region_apply_v1"
        helpers += 1
        return False


gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set detach-on-fork on")
gdb.execute("set follow-fork-mode parent")
Enter("*topal_kernel_region_apply_v1", internal=True)
# The test supplies the exact checked helper symbol through a convenience value.
Helper(gdb.parse_and_eval("$region_helper").string(), internal=True)
gdb.execute("run")
assert gdb.selected_inferior().pid == 0, "native client did not exit"
assert returns == 18, returns
assert helpers == 3, helpers
assert int(gdb.parse_and_eval("$_exitcode")) == 0
print("REGION_SOURCE_DEBUG_OK returns=18 helpers=3 preserved=rbx,rbp,r12,r13,r14,r15,rsp")
