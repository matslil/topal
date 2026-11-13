"""Independent pre-invocation contract for brk, exec reset and completion.

TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001. This is host-only model data. It neither
manufactures kernel owners nor executes inconsistent states in a VM. It checks
only selected target pre-invocation checks, not actual ownership, fresh-entry
generation, heap request legality or post-write fatal retirement. Other
operation classes remain outside this selected contract.
"""
from dataclasses import dataclass, replace

from gateway_machine import word


MINIMUM_SP = 0x109040
MAXIMUM_SP = 0x17fff8


@dataclass(frozen=True)
class Case:
    operation: int = 5
    sp: int = 0x17fdf8
    cs: int = 16
    flags: int = 2
    root: int = 0x900000
    cleanup_root: int = 0xa00000
    cr3: int = 0x900000
    gs: int = 0
    kernel_gs: int = 0
    self_pointer: int = 0
    state: int = 1
    request: int = 0x63a000
    captured_request: int = 0x63a000
    captured_sp: int = 0x17fe08
    return_pc: int = 0
    entry: int = 7
    number: int = 12
    second: int = 0
    third: int = 0
    fourth: int = 0
    recovery: tuple = (0, 0, 0, 0, 0, 0)

    def check_words(self):
        for name, value in vars(self).items():
            if name != 'recovery':
                word(value)
        if len(self.recovery) != 6:
            raise ValueError('model recovery closure')
        for value in self.recovery:
            word(value)


def predicate(case, layout):
    case.check_words()
    if case.operation not in [5, 6, 7]:
        raise ValueError('operation outside selected gateway contract')
    if not (MINIMUM_SP <= case.sp <= MAXIMUM_SP and case.sp % 16 == 8
            and case.cs & 3 == 0 and case.flags & 0x600 == 0):
        return False
    if (case.second or case.third or case.fourth or any(case.recovery)
            or case.captured_sp != case.sp + 16 or case.entry == 0):
        return False
    if case.operation == 6:
        return (case.return_pc == layout['cleanup_return'] and case.request == 0
                and case.state == 2 and case.cr3 == case.cleanup_root
                and case.kernel_gs == layout['context'] and case.number in [60, 231])
    return (case.return_pc == layout['brk_return' if case.operation == 5 else 'exec_return']
            and case.state == 1 and case.self_pointer == layout['context']
            and case.cr3 == case.root and case.gs == layout['context']
            and case.number == (12 if case.operation == 5 else 59)
            and (case.request == case.captured_request if case.operation == 5 else case.request == 0))


def seed(layout, operation=5):
    case = Case(operation=operation, gs=layout['context'], kernel_gs=layout['context'],
                self_pointer=layout['context'], return_pc=layout['brk_return'])
    if operation == 6:
        case = replace(case, request=0, state=2, cr3=case.cleanup_root,
                       return_pc=layout['cleanup_return'], number=60)
    elif operation == 7:
        case = replace(case, request=0, return_pc=layout['exec_return'], number=59)
    if operation not in [5, 6, 7]:
        raise ValueError('operation outside selected gateway contract')
    return case


def machine_inputs(case, layout):
    """Represent model fields as sparse words; never dereference host addresses."""
    case.check_words()
    registers = [0xa500000000000000 + index for index in range(16)]
    for index, value in [(4, case.sp), (7, case.operation), (6, case.request),
                         (2, case.second), (1, case.third), (8, case.fourth)]:
        registers[index] = value
    context, epoch, recovery = layout['context'], layout['epoch'], layout['recovery']
    memory = {case.sp: case.return_pc, context: case.state, context + 8: case.captured_sp,
              context + 16: case.cleanup_root, context + 24: case.root,
              context + 256: case.self_pointer, context + 312: case.captured_request,
              epoch: case.entry, epoch + 8: case.number}
    memory.update({recovery + index * 8: value for index, value in enumerate(case.recovery)})
    return dict(registers=registers, memory=memory, cs=case.cs, flags=case.flags, cr3=case.cr3,
                msrs={0xc0000101: case.gs, 0xc0000102: case.kernel_gs})
