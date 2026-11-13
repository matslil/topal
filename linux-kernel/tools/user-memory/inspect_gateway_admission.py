"""Compare saved compiled gateway admission with a separate host contract.

TOPAL-SYSTEMS-FRAME-POLICY-BINDING-001. This is model interpretation, not native
fault injection. It changes no guest, binary, register, owner or production gate.
"""
import argparse
from dataclasses import replace
import json
from pathlib import Path
import random
import struct

from gateway_contract import MINIMUM_SP, MAXIMUM_SP, machine_inputs, predicate, seed
from gateway_machine import Program, WORD, require
from inspect_call_inventory import Elf, digest


SYMBOLS = {
    'context': 'topal_bootstrap_storage',
    'epoch': 'topal_x86_frame_mm_epoch_v1',
    'recovery': 'topal_x86_frame_gateway_recovery_v1',
    'brk_return': 'topal_x86_frame_mm_brk_return_v1',
    'cleanup_return': 'topal_x86_frame_mm_cleanup_return_v1',
    'exec_return': 'topal_x86_frame_mm_exec_return_v1',
}
CALLEE = [3, 5, 12, 13, 14, 15]
SAVES = bytes.fromhex('555341544155415641574989e4')


def storage(elf, name, needed):
    symbol = elf.symbols[name]
    index = symbol['section']
    require(symbol['info'] & 15 == 1 and 0 < index < len(elf.sections), 'gateway data symbol class')
    section = elf.sections[index]
    require(section[2] & 3 == 3 and not section[2] & 4
            and symbol['size'] >= needed and section[3] <= symbol['value']
            and symbol['value'] + symbol['size'] <= section[3] + section[5],
            'gateway data symbol allocated writable extent')
    return symbol['value']


def caller(elf, name, gateway):
    symbol = elf.symbols[name]
    require(symbol['section'] == gateway['section'] and symbol['info'] & 15 == 2,
            'compiled caller symbol identity')
    section = elf.sections[symbol['section']]
    at = symbol['value'] - section[3]
    require(5 <= at < section[5], 'compiled caller return extent')
    raw = elf.section_bytes(symbol['section'])[at - 5:at]
    require(len(raw) == 5 and raw[0] == 0xe8
            and symbol['value'] + struct.unpack_from('<i', raw, 1)[0] == gateway['value'],
            'compiled caller return does not follow its actual gateway call')
    return symbol['value']


def linked(kernel):
    elf = Elf(kernel)
    require(elf.kind == 2, 'linked gateway executable required')
    symbol, body = elf.function('topal_x86_frame_gateway_v1')
    require(elf.sections[symbol['section']][2] & 6 == 6, 'gateway executable allocation')
    refusal_symbol = elf.symbols['topal_x86_frame_gateway_refused_v1']
    require(refusal_symbol['section'] == symbol['section'], 'gateway refusal section')
    refusal = refusal_symbol['value'] - symbol['value']
    boundaries = [at for at in range(len(body)) if body[at:at + len(SAVES)] == SAVES]
    require(len(boundaries) == 1, 'unique compiled pre-save boundary')
    program = Program(body, boundaries[0], refusal)
    layout = dict(context=storage(elf, SYMBOLS['context'], 512 + 320) + 512,
                  epoch=storage(elf, SYMBOLS['epoch'], 16),
                  recovery=storage(elf, SYMBOLS['recovery'], 48))
    layout.update({key: caller(elf, SYMBOLS[key], symbol)
                   for key in ['brk_return', 'cleanup_return', 'exec_return']})
    intervals = [(layout['context'], layout['context'] + 320),
                 (layout['epoch'], layout['epoch'] + 16), (layout['recovery'], layout['recovery'] + 48)]
    require(all(end <= MINIMUM_SP or begin > MAXIMUM_SP + 16 for begin, end in intervals),
            'modeled metadata aliases bootstrap continuation range')
    require(all(end <= other or other_end <= begin for i, (begin, end) in enumerate(intervals)
                for other, other_end in intervals[i + 1:]), 'modeled metadata overlaps')
    return program, layout, dict(kernel_sha256=digest(elf.data), function=symbol,
        function_sha256=digest(body), boundary_offset=program.boundary, refusal_offset=refusal,
        prefix_sha256=digest(body[:program.boundary]), prefix_instruction_count=len(program.instructions),
        save_bytes=SAVES.hex(), refusal_bytes=body[refusal:refusal + 4].hex(), layout=layout)


def check(program, layout, case):
    expected = predicate(case, layout)
    inputs = machine_inputs(case, layout)
    result = program.run(**inputs)
    require(result['admitted'] == expected, 'compiled admission differs from independent predicate')
    require(all(result['registers'][reg] == inputs['registers'][reg] for reg in CALLEE),
            'pre-admission callee register changed')
    require(not set(result['scratch_writes']) - {case.sp - 8}, 'unexpected pre-admission store')
    return result


def cases(layout):
    """Complete-word and independent field counterexamples, with valid controls."""
    original = seed(layout)
    legacy = {
        'return-site': replace(original, return_pc=original.return_pc + 1),
        'gs-context': replace(original, gs=0),
        'address-space': replace(original, root=original.cleanup_root),
        'entry-identity': replace(original, entry=0),
        'abi-number': replace(original, number=59),
        'captured-stack': replace(original, captured_sp=original.captured_sp + 16),
        'request': replace(original, request=1),
        'busy': replace(original, recovery=(0, 85, 0, 0, 0, 0)),
        'direction-flag': replace(original, flags=original.flags | 1024),
    }
    for name, case in legacy.items():
        yield 'legacy-' + name, case
    for op in [5, 6, 7]:
        original = seed(layout, op)
        yield 'valid-' + str(op), original
        for sp in [MINIMUM_SP - 1, MINIMUM_SP, MINIMUM_SP + 8, MAXIMUM_SP,
                   MAXIMUM_SP + 1, 0, WORD]:
            yield f'stack-{op}-{sp}', replace(original, sp=sp, captured_sp=(sp + 16) & WORD)
        # Flip every bit independently; do not assume each bit is invalid.
        # The independent predicate determines validity for each complete word.
        for name in ['sp', 'cs', 'flags', 'root', 'cleanup_root', 'cr3', 'gs', 'kernel_gs',
                     'self_pointer', 'state', 'request', 'captured_request', 'captured_sp',
                     'return_pc', 'entry', 'number', 'second', 'third', 'fourth']:
            for bit in range(64):
                yield f'{name}-{op}-{bit}', replace(original, **{name: getattr(original, name) ^ (1 << bit)})
        for index in range(6):
            for bit in range(64):
                values = list(original.recovery)
                values[index] ^= 1 << bit
                yield f'busy-{op}-{index}-{bit}', replace(original, recovery=tuple(values))
        for request in [0, 1, (1 << 32) - 1, 1 << 32, 1 << 63, WORD]:
            yield f'captured-request-{op}-{request}', replace(original, request=request, captured_request=request)
        for entry in [0, 1, (1 << 32) - 1, 1 << 32, 1 << 63, WORD]:
            yield f'complete-entry-word-{op}-{entry}', replace(original, entry=entry)
        if op == 6:
            yield 'exit-group-number', replace(original, number=231)
    rng = random.Random(0x47574159)
    for index in range(256):
        original = seed(layout, [5, 6, 7][index % 3])
        changes = {name: rng.getrandbits(64) for name in ['cs', 'flags', 'entry', 'number', 'request']}
        yield 'combined-' + str(index), replace(original, **changes)


def inspect(kernel):
    program, layout, binding = linked(kernel)
    admitted = refused = 0
    seen = set()
    branches = set()
    results = []
    for name, case in cases(layout):
        result = check(program, layout, case)
        admitted += result['admitted']
        refused += not result['admitted']
        seen.update(result['instructions'])
        branches.update((row['offset'], row['taken']) for row in result['branches'])
        results.append(dict(name=name, admitted=result['admitted'], stop_offset=result['stop_offset'],
                            interpreted_instructions=len(result['instructions']),
                            scratch_word_written=bool(result['scratch_writes'])))
    require(admitted > 0 and refused > 0, 'vacuous gateway model matrix')
    return dict(passed=True, schema='topal.gateway-admission-model/1', binding=binding,
                cases=len(results), admitted=admitted, refused=refused, interpreted_offsets=sorted(seen),
                branch_outcomes=[dict(offset=at, taken=taken) for at, taken in sorted(branches)],
                unvisited_prefix_offsets=sorted(set(program.instructions) - seen), results=results,
                legacy_preentry_modes=['return-site', 'gs-context', 'address-space', 'entry-identity',
                    'abi-number', 'captured-stack', 'request', 'busy', 'direction-flag'],
                qualification_complete=False,
                assumptions=['Saved code and modeled metadata stay unchanged throughout one sequential interpretation.',
                    'Selected long-mode instruction forms and sparse metadata are readable; paging, CPU faults and asynchronous entry remain separate.',
                    'CF/ZF/IF/DF are the relevant flags; the temporary PUSHF word is a projection, not native stack contents.',
                    'Real stack/context ownership, legitimate caller origin and fresh nonwrapping entry generation are external obligations; a matching modeled number cannot establish them.'],
                scope='Host-only interpretation of exact saved brk/exec-reset/completion pre-invocation paths, compared with an independent predicate. Other gateway operations, post-admission effects/fatal retirement, real owner/stack/hardware authority, native fault coverage and production qualification remain open. No test, check or activation gate is removed.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kernel', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    report = inspect(args.kernel)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ['passed', 'cases', 'admitted', 'refused',
                                                  'qualification_complete']}))
