"""Inspect selected native extent-result gates and three fallback read bounds.

TOPAL-SYSTEMS-USER-EXTENT-001. This is artifact/local-control-flow evidence,
not native execution or global load/store/indirect-control dominance proof.
"""
import json
from pathlib import Path
import re
import struct
import sys

from inspect_call_inventory import Elf, digest, verify_calls


CASES = (
    ('named-condition', 'static-process-shell-named-root-condition-write-profile-ready', 14),
    ('exec-signal-reset', 'static-process-execve-signal-number-ready', 32),
    ('missing-path', 'static-process-shell-root-directory-stat-profile-ready', 9),
)


def relative(body, at, size, displacement):
    return at + size + struct.unpack_from('<i', body, at + displacement)[0]


def audit(build):
    inventory = json.loads((build / 'extent-call-inventory.json').read_text())
    elf = Elf(build / 'provider.o')
    symbol = verify_calls(elf, inventory)
    _, body = elf.function(inventory['function'])
    decoded = {}
    for line in (build / 'extent-call-disassembly.stdout.log').read_text().splitlines():
        match = re.match(r'^\s*([0-9a-f]+):\s+(.*)$', line)
        if match:
            at = int(match[1], 16) - symbol['value']
            if 0 <= at < len(body):
                assert at not in decoded
                decoded[at] = match[2]
    gates = []
    for row in inventory['calls']:
        at = row['offset'] + 14
        assert body[at:at + 2] == b'\x0f\x82', ('missing immediate carry refusal', row)
        target = relative(body, at, 6, 2)
        assert row['offset'] in decoded and at in decoded and target in decoded
        assert decoded[at].startswith('jb\t')
        gates.append(dict(call_offset=row['offset'], target=row['target'],
                          carry_gate_offset=at, refusal_offset=target,
                          scope='No instruction between helper return and carry decision except verified CF-preserving return-word erase'))
    direct_edges = []
    unknown = []
    indirect = []
    for at, instruction in decoded.items():
        if '<unknown>' in instruction:
            unknown.append(at)
        if re.match(r'^(?:j\w*|callq?)\s+\*', instruction):
            indirect.append(dict(offset=at,instruction=instruction))
        match = re.match(r'^(?:j\w*|callq?)\s+0x([0-9a-f]+)', instruction)
        if match:
            direct_edges.append((at,int(match[1],16)-symbol['value']))
    results = []
    for name, label, length in CASES:
        # The write path's clean and profile labels share one offset. Retain
        # exact census identity instead of guessing a source line or address.
        matches = [r for r in inventory['calls'] if label in r['nearest_labels']]
        assert len(matches) == 1 and matches[0]['target'] == 'runtime-readable-extent'
        call = matches[0]['offset']
        start = call - 22
        materialize = bytes.fromhex('65 48 8b 34 25 40 01 00 00 49 89 f0 49 b9')
        assert body[start:call] == materialize + struct.pack('<Q',length)
        gate = call + 14
        success = gate + 6
        failure = relative(body,gate,6,2)
        assert body[failure:failure+10] == bytes.fromhex('48 b8 f2 ff ff ff ff ff ff ff')
        accesses = []
        branches = []
        if name == 'named-condition':
            prefix = bytes.fromhex('45 31 d2 42 8a 04 16 65 41 88 82 e8 02 00 00 49 ff c2 49 83 fa 0e')
            assert body[success:success+len(prefix)] == prefix
            loop = success+3
            back = success+len(prefix)
            assert body[back:back+2] == b'\x0f\x85'
            assert relative(body,back,6,2) == loop
            # Independent finite index enumeration: the 32-bit zeroing and
            # compare-after-increment admit precisely indices zero through 13.
            index = 0
            indices = []
            while True:
                indices.append(index)
                index += 1
                if index == length:
                    break
            assert indices == list(range(14))
            assert all(0 <= index < length for index in indices)
            accesses.append(dict(offset=loop,width=1,indices=indices,
                                 candidate_register='rsi',index_register='r10'))
            end = back+6
            allowed = {(back,loop)}
        elif name == 'exec-signal-reset':
            at = success
            for offset, pattern in [
                (0, bytes.fromhex('48 83 3e 00')),
                (8, bytes.fromhex('48 81 7e 08 00 00 00 04')),
                (16,bytes.fromhex('48 81 7e 10 40 97 4a 00')),
            ]:
                assert body[at:at+len(pattern)] == pattern
                accesses.append(dict(offset=at,width=8,candidate_offset=offset))
                at += len(pattern)
                assert body[at:at+2] == b'\x0f\x85'
                branches.append(relative(body,at,6,2))
                at += 6
            immediate = bytes.fromhex('48 b8 ff ff ff 7f fe ff ff ff')
            assert body[at:at+10] == immediate
            at += 10
            assert body[at:at+4] == bytes.fromhex('48 39 46 18')
            accesses.append(dict(offset=at,width=8,candidate_offset=24))
            at += 4
            assert body[at:at+2] == b'\x0f\x85'
            branches.append(relative(body,at,6,2))
            end = at+6
            allowed = set()
        else:
            prefix = bytes.fromhex('65 48 8b 0c 25 28 01 00 00 48 b8 2f 6d 69 73 73 69 6e 67')
            assert body[success:success+len(prefix)] == prefix
            at = success+len(prefix)
            assert body[at:at+3] == bytes.fromhex('48 39 06')
            accesses.append(dict(offset=at,width=8,candidate_offset=0))
            at += 3
            assert body[at:at+2] == b'\x0f\x85'
            branches.append(relative(body,at,6,2))
            at += 6
            assert body[at:at+4] == bytes.fromhex('80 7e 08 00')
            accesses.append(dict(offset=at,width=1,candidate_offset=8))
            at += 4
            assert body[at:at+2] == b'\x0f\x85'
            branches.append(relative(body,at,6,2))
            end = at+6
            allowed = set()
        for access in accesses:
            assert access['offset'] in decoded
            if 'candidate_offset' in access:
                assert 0 <= access['candidate_offset'] and access['candidate_offset']+access['width'] <= length
        for target in branches:
            assert body[target:target+2] == bytes.fromhex('fa f4'), 'content mismatch must reach invariant halt'
        incoming = [(source,target) for source,target in direct_edges if start < target < end]
        assert set(incoming) == allowed, ('unexpected decoded edge into guarded local path', name, incoming)
        assert not any(start <= offset < end for offset in unknown)
        results.append(dict(name=name,call_offset=call,candidate_setup_offset=start,
            admitted_bytes=length,success_offset=success,refusal_offset=failure,
            access_instructions=accesses,local_end=end,
            accepted_decoded_incoming_edges=sorted(allowed),
            disassembly=[dict(offset=at,instruction=decoded[at])
                         for at in sorted(decoded) if start<=at<end],
            scope='Exact local materialization/CF gate/read bounds and decoded direct-entry checks; native reachability, helper/lifetime soundness and complete indirect/exception entry proof remain separate'))
    report = dict(passed=True,function=inventory['function'],function_sha256=inventory['function_sha256'],
        provider_sha256=digest(elf.data),inventory_sha256=digest((build/'extent-call-inventory.json').read_bytes()),
        call_count=len(gates),carry_gates=gates,fallback_local_reads=results,
        decoded_instruction_rows=len(decoded),linear_decode_unknown_offsets=unknown,
        decoded_indirect_transfers=indirect,
        scope='Selected artifact result-gate and local fallback-read audit. Linear disassembly contains embedded data and unknown rows; this is not a complete direct-user-access inventory or global dominance/native/production qualification')
    (build/'transfer-guard-audit.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ['carry_gates','fallback_local_reads','linear_decode_unknown_offsets','decoded_indirect_transfers']}))


if __name__ == '__main__':
    audit(Path(sys.argv[1]).resolve())
