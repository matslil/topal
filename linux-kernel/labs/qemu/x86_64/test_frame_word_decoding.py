"""Preserve fresh, complete unsigned observations while decoding in bulk.

TOPAL-SYSTEMS-USER-EXTENT-001: this checks observer equivalence, not native
ownership or lifetime. Both functions are extracted from the actual embedded
debugger programs without importing GDB or launching a VM.
"""
import ast
from pathlib import Path
import random
import struct
import unittest


LAB = Path(__file__).resolve().parent


def decoder(filename, memory):
    module = ast.parse((LAB / filename).read_text())
    assignment = next(node for node in module.body if isinstance(node, ast.Assign)
                      and any(isinstance(target, ast.Attribute) and target.attr == 'GDB_PROGRAM'
                              or isinstance(target, ast.Name) and target.id == 'GDB_PROGRAM'
                              for target in node.targets))
    program = ast.parse(ast.literal_eval(assignment.value))
    function = next(node for node in program.body
                    if isinstance(node, ast.FunctionDef) and node.name == 'words')
    namespace = dict(memory=memory, struct=struct)
    exec(compile(ast.Module(body=[function], type_ignores=[]), filename, 'exec'), namespace)
    return namespace['words']


class FrameWordDecodingTests(unittest.TestCase):
    def test_matches_predecessor_for_large_unsigned_little_endian_records(self):
        data = random.Random(283).randbytes(8 * 1048576)
        for count in [0, 1, 3, 8, 16, 512, 519 * 3, 1048576]:
            with self.subTest(count=count):
                calls = []
                def memory(pointer, size):
                    calls.append((pointer, size))
                    return data[1:1 + size] if pointer == 1 else data[:size]
                old = decoder('frame_root_inspect.py', memory)
                new = decoder('frame_memory_inspect.py', memory)
                self.assertEqual(new(0, count), old(0, count))
                self.assertEqual(calls, [(0, count * 8)] * 2)

    def test_all_bit_positions_and_unaligned_address(self):
        values = [0, (1 << 64) - 1] + [1 << bit for bit in range(64)]
        data = b'guard' + b''.join(value.to_bytes(8, 'little') for value in values)
        memory = lambda pointer, size: data[pointer:pointer + size]
        self.assertEqual(decoder('frame_memory_inspect.py', memory)(5, len(values)), values)

    def test_repeated_observations_read_every_byte_again(self):
        data = bytearray(24)
        calls = []
        def memory(pointer, size):
            calls.append((pointer, size))
            return bytes(data[pointer:pointer + size])
        words = decoder('frame_memory_inspect.py', memory)
        before = words(0, 3)
        data[0], data[-1] = 0x12, 0x80
        after = words(0, 3)
        self.assertEqual(before, [0, 0, 0])
        self.assertEqual(after, [0x12, 0, 0x8000000000000000])
        self.assertEqual(calls, [(0, 24), (0, 24)])

    def test_truncated_observation_cannot_silently_supply_fewer_words(self):
        words = decoder('frame_memory_inspect.py', lambda pointer, size: bytes(size - 1))
        with self.assertRaises(struct.error):
            words(0, 3)


if __name__ == '__main__':
    unittest.main()
