"""Host-only interpretation of a closed x86-64 gateway admission prefix.

TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001. Numeric inputs are model data, never
kernel capabilities. Saved machine code is never executed. Only CF/ZF and
IF/DF observations are modeled; the PUSHF scratch word is a flag projection,
not an exact native-stack observation. Faults, paging and asynchronous effects
are not modeled. Unsupported code or absent memory is a coverage gap.
"""
from dataclasses import dataclass


WORD = (1 << 64) - 1


class ModelGap(ValueError):
    """The selected model cannot classify this input or instruction."""


def require(condition, message):
    if not condition:
        raise ModelGap(message)


def word(value):
    require(type(value) is int and 0 <= value <= WORD, 'model word width')
    return value


@dataclass(frozen=True)
class Instruction:
    offset: int
    size: int
    operation: str
    arguments: tuple = ()


def decode(raw, offset):
    """Strict decoder for selected emitted forms, independent of producer code."""
    start = offset

    def take(count):
        nonlocal offset
        require(0 <= offset <= offset + count <= len(raw), 'truncated model instruction')
        result = raw[offset:offset + count]
        offset += count
        return result

    def result(operation, *arguments):
        return Instruction(start, offset - start, operation, arguments)

    first = take(1)[0]
    if first == 0xe9:
        delta = int.from_bytes(take(4), 'little', signed=True)
        return result('jump', offset + delta)
    if first == 0x0f:
        second = take(1)[0]
        if second in [0x82, 0x84, 0x85, 0x86, 0x87]:
            delta = int.from_bytes(take(4), 'little', signed=True)
            return result('condition', second, offset + delta)
        if second == 0x32:
            return result('read-msr')
        require(second == 0x20 and take(1) == b'\xd8', 'unsupported privileged instruction')
        return result('read-cr3')
    if first == 0x9c:
        return result('push-flags-projection')
    if first == 0x58:
        return result('pop-rax')
    if first == 0x8c:
        require(take(1) == b'\xc8', 'unsupported segment-register move')
        return result('read-cs')
    require(0x48 <= first <= 0x4f and not first & 2, 'unsupported model prefix')
    rex = first
    opcode = take(1)[0]
    if 0xb8 <= opcode <= 0xbf:
        register = opcode - 0xb8 + (8 if rex & 1 else 0)
        return result('constant', register, int.from_bytes(take(8), 'little'))
    require(opcode in [0x81, 0x89, 0x8b, 0x39, 0x09, 0x29, 0xc1], 'unsupported model opcode')
    modrm = take(1)[0]
    mode, middle, low = modrm >> 6, (modrm >> 3) & 7, modrm & 7
    register = middle + (8 if rex & 4 else 0)
    other = low + (8 if rex & 1 else 0)
    if opcode == 0x8b and mode == 2:
        if low == 4:
            require(take(1) == b'\x24', 'unsupported indexed memory operand')
        displacement = int.from_bytes(take(4), 'little', signed=True)
        return result('load', register, other, displacement)
    require(mode == 3, 'model refuses stores or unsupported addressing')
    if opcode == 0x81:
        require(not rex & 4 and middle in [0, 4, 7], 'unsupported immediate operation')
        immediate = int.from_bytes(take(4), 'little', signed=True) & WORD
        return result({0: 'add-constant', 4: 'mask', 7: 'compare-constant'}[middle], other, immediate)
    if opcode == 0xc1:
        require(not rex & 4 and middle == 4, 'unsupported shift')
        return result('shift-left', other, take(1)[0] & 63)
    return result({0x89: 'move', 0x8b: 'reverse-move', 0x39: 'compare',
                   0x09: 'or', 0x29: 'subtract'}[opcode], other, register)


class Program:
    def __init__(self, body, boundary, refusal):
        require(type(boundary) is int and 0 < boundary <= len(body), 'model admission boundary')
        require(type(refusal) is int and boundary < refusal <= len(body) - 4
                and body[refusal:refusal + 4] == bytes.fromhex('faf4ebfd'), 'model refusal marker')
        self.body, self.boundary, self.refusal = bytes(body), boundary, refusal
        self.instructions = {}
        cursor = 0
        while cursor < boundary:
            row = decode(body, cursor)
            require(cursor + row.size <= boundary, 'model boundary splits instruction')
            self.instructions[cursor] = row
            cursor += row.size
        require(cursor == boundary, 'model prefix coverage')
        for at, row in self.instructions.items():
            if row.operation == 'push-flags-projection':
                following = self.instructions.get(at + row.size)
                require(following is not None and following.operation == 'pop-rax',
                        'flags projection needs immediate matched pop')
                mask = self.instructions.get(following.offset + following.size)
                require(mask is not None and mask.operation == 'mask' and mask.arguments == (0, 0x600),
                        'flags projection observed outside IF/DF')
        successors = {}
        for at, row in self.instructions.items():
            following = at + row.size
            targets = ([row.arguments[0]] if row.operation == 'jump' else
                       [following, row.arguments[1]] if row.operation == 'condition' else [following])
            require(all(target in self.instructions or target in [boundary, refusal] for target in targets),
                    'model branch enters an instruction or leaves the selected prefix')
            successors[at] = targets
        active, visited = set(), set()

        def visit(at):
            if at in [boundary, refusal] or at in visited:
                return
            require(at not in active, 'cyclic admission prefix')
            active.add(at)
            for target in successors[at]:
                visit(target)
            active.remove(at)
            visited.add(at)

        # Inspect all prefix components, including paths not selected by a case.
        for at in successors:
            visit(at)

    def run(self, registers, memory, *, cs, flags, cr3, msrs):
        require(len(registers) == 16, 'model register bank')
        registers = [word(value) for value in registers]
        original_sp = registers[4]
        memory = {word(address): word(value) for address, value in memory.items()}
        cs, flags, cr3 = word(cs), word(flags), word(cr3)
        msrs = {word(key): word(value) for key, value in msrs.items()}
        reads, scratch, trace, branches = [], {}, [], []
        carry, zero = bool(flags & 1), bool(flags & 0x40)
        at = 0

        def load(address):
            address &= WORD
            reads.append(address)
            require(address in scratch or address in memory, 'unmodeled memory read at ' + hex(address))
            return scratch[address] if address in scratch else memory[address]

        for _ in range(len(self.instructions) + 1):
            if at in [self.boundary, self.refusal]:
                require(registers[4] == original_sp, 'model prefix changed continuation stack')
                return dict(admitted=at == self.boundary, stop_offset=at, registers=registers,
                            reads=reads, scratch_writes=dict(scratch), instructions=trace, branches=branches,
                            scope='CF/ZF/IF/DF projection of a saved admission prefix; no native execution, owner creation or hardware/lifetime qualification.')
            require(at in self.instructions, 'unmodeled instruction start')
            row = self.instructions[at]
            trace.append(at)
            operation, args = row.operation, row.arguments
            following = at + row.size
            if operation == 'jump':
                following = args[0]
            elif operation == 'condition':
                code, target = args
                taken = {0x82: carry, 0x84: zero, 0x85: not zero,
                         0x86: carry or zero, 0x87: not carry and not zero}[code]
                branches.append(dict(offset=at, taken=taken, target=target))
                if taken:
                    following = target
            elif operation == 'constant':
                registers[args[0]] = args[1]
            elif operation == 'load':
                target, base, displacement = args
                registers[target] = load(registers[base] + displacement)
            elif operation in ['move', 'reverse-move']:
                target, source = args if operation == 'move' else args[::-1]
                registers[target] = registers[source]
            elif operation in ['compare', 'compare-constant']:
                left = registers[args[0]]
                right = args[1] if operation == 'compare-constant' else registers[args[1]]
                carry, zero = left < right, left == right
            elif operation in ['mask', 'or', 'subtract', 'add-constant']:
                target = args[0]
                left = registers[target]
                right = args[1] if operation in ['mask', 'add-constant'] else registers[args[1]]
                value = (left & right if operation == 'mask' else left | right if operation == 'or'
                         else left - right if operation == 'subtract' else left + right)
                carry = left < right if operation == 'subtract' else value > WORD if operation == 'add-constant' else False
                registers[target] = value & WORD
                zero = registers[target] == 0
            elif operation == 'shift-left':
                target, count = args
                if count:
                    carry = bool((registers[target] >> (64 - count)) & 1)
                    registers[target] = (registers[target] << count) & WORD
                    zero = registers[target] == 0
            elif operation == 'read-cs':
                registers[0] = cs & 0xffff
            elif operation == 'read-cr3':
                require(cs & 3 == 0, 'unmodeled privileged read before CPL refusal')
                registers[0] = cr3
            elif operation == 'read-msr':
                require(cs & 3 == 0, 'unmodeled privileged read before CPL refusal')
                key = registers[1] & 0xffffffff
                require(key in msrs, 'unmodeled MSR read')
                registers[0], registers[2] = msrs[key] & 0xffffffff, msrs[key] >> 32
            elif operation == 'push-flags-projection':
                address = (registers[4] - 8) & WORD
                require(address == original_sp - 8 and not scratch, 'unmodeled stack store')
                scratch[address] = (flags & 0x600) | int(carry) | (int(zero) << 6)
                registers[4] = address
            elif operation == 'pop-rax':
                require(registers[4] in scratch, 'unmatched model stack pop')
                registers[0] = load(registers[4])
                registers[4] = (registers[4] + 8) & WORD
            else:
                raise ModelGap('unmodeled operation ' + operation)
            at = following
        raise ModelGap('admission interpretation step bound')
