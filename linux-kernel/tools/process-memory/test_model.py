"""Reference conformance tests for TOPAL-SYSTEMS-PROCESS-MM-001.

The byte oracle has no physical placement, extent tokens or event protocol.
Saved-history controls mutate model data only; none run in a guest.
"""
from dataclasses import replace
import random
import unittest

import model as m
from trace_check import check


class ModelTests(unittest.TestCase):
    def call(self, w, operation, *args, **kwargs):
        before, entry = m.enter(w)
        after, value, events = operation(before, entry, *args, **kwargs)
        check(before, after, events)
        return after, value, events

    def resize(self, w, pages, extra=0):
        return self.call(w, m.brk, w.geometry.start + pages * w.geometry.page + extra)[0]

    def write(self, w, offset, data):
        w, entry = m.enter(w)
        w, loan = m.borrow(w, entry, w.geometry.start + offset, len(data), True)
        w, _ = m.access(w, loan, data)
        return m.leave(m.end_loan(w, loan), entry)

    def contents(self, w, pid=None):
        p = w.process(pid)
        return w.extent(p.heap).data if p.heap else b''

    def test_actual_heap_sizes_and_zero_new_tail(self):
        for count in [0, 1, 33, 34, 35, 67, 519]:
            with self.subTest(pages=count):
                w = self.resize(m.initial(), count)
                if count:
                    w = self.write(w, count * 4096 - 5, b'last!')
                old = self.contents(w)
                w, pid, _ = self.call(w, m.fork)
                self.assertEqual(pid, 2)
                self.assertEqual(self.contents(w, 1), old)
                self.assertEqual(self.contents(w), old)
                self.assertNotEqual(w.process(1).address_space, w.process(2).address_space)
                self.assertFalse(set(w.frames(1)).intersection(w.frames(2)))
                if count:
                    self.assertNotEqual(w.process(1).heap, w.process(2).heap)
                    w = self.write(w, count * 4096 - 5, b'child')
                    self.assertEqual(self.contents(w, 1), old)
                w, _, _ = self.call(w, m.child_exit, 257)
                self.assertEqual(self.contents(w), old)
                self.assertEqual(w.status, 1)
                w, value, _ = self.call(w, m.wait)
                self.assertEqual(value, 2)
                w, value, _ = self.call(w, m.wait)
                self.assertEqual(value, -10)
                w, _, _ = self.call(w, m.terminate_outer)
                self.assertEqual(w.extents, ())

    def test_partial_break_and_within_page_retention(self):
        w = self.resize(m.initial(), 34, 3)
        target = w.geometry.start + 34 * 4096 + 3
        w = self.write(w, 35 * 4096 - 1, b'P')  # admitted mapped-page slack
        w, _, _ = self.call(w, m.fork)
        parent = w.process(1)
        w = self.write(w, 35 * 4096 - 1, b'C')
        token = w.process().heap
        w, _, events = self.call(w, m.brk, target + 1)
        self.assertEqual(events, ())
        self.assertEqual(w.process().heap, token)
        self.assertEqual(self.contents(w)[-1:], b'C')
        self.assertEqual(self.contents(w, 1)[-1:], b'P')
        self.assertEqual(w.process(1), parent)

    def test_child_exec_and_resize_preserve_inactive_parent(self):
        w = self.write(self.resize(m.initial(), 67), 66 * 4096, b'parent')
        w, _, _ = self.call(w, m.fork)
        parent, data = w.process(1), self.contents(w, 1)
        child_old = w.process().heap
        w, _, events = self.call(w, m.exec_reset)
        self.assertEqual(w.process().generation, 1)
        self.assertEqual(self.contents(w), b'')
        self.assertNotIn(child_old, [e.token for e in w.extents])
        self.assertEqual([e.token for e in events if e.kind == 'release'], [child_old])
        w = self.resize(w, 101)
        self.assertFalse(any(self.contents(w)))
        self.assertEqual((w.process(1), self.contents(w, 1)), (parent, data))
        w, _, _ = self.call(w, m.handoff)
        self.assertEqual(w.process().program_break, parent.program_break)
        self.assertEqual(self.contents(w), data)

    def test_parent_exec_preserves_inactive_child(self):
        w = self.resize(m.initial(), 35)
        w, _, _ = self.call(w, m.fork)
        w = self.write(w, 34 * 4096, b'child')
        child, data = w.process(), self.contents(w)
        w, _, _ = self.call(w, m.handoff)
        w, _, _ = self.call(w, m.exec_reset)
        self.assertEqual(w.process().generation, 1)
        self.assertEqual(w.process().program_break, w.geometry.start)
        self.assertEqual((w.process(2), self.contents(w, 2)), (child, data))

    def test_failure_preserves_all_owned_state_and_identity_history(self):
        g = m.Geometry(pages=67, frames=70)
        w = self.write(self.resize(m.initial(g), 35), 34 * 4096, b'keep')
        w, _, _ = self.call(w, m.fork)
        w1, value, events = self.call(w, m.brk, g.start + 36 * g.page)
        self.assertEqual(value, w.process().program_break)
        self.assertEqual(events, ())
        self.assertEqual(replace(w1, next_epoch=w.next_epoch), w)
        w1, value, events = self.call(w, m.fork)
        self.assertEqual(value, -11)
        self.assertEqual(events, ())
        self.assertEqual(replace(w1, next_epoch=w.next_epoch), w)
        single = self.resize(m.initial(m.Geometry(frames=60)), 35)
        failed, value, events = self.call(single, m.fork)
        self.assertEqual(value, -12)
        self.assertEqual(events, ())
        self.assertEqual(replace(failed, next_epoch=single.next_epoch), single)

    def test_raw_brk_query_failure_and_zero_heap(self):
        w = self.resize(m.initial(), 35)
        for request in [0, 1, w.geometry.start - 1, w.geometry.start + 520 * 4096, (1 << 64) - 1]:
            w1, result, events = self.call(w, m.brk, request)
            self.assertEqual(result, w.process().program_break)
            self.assertEqual(events, ())
            self.assertEqual(replace(w1, next_epoch=w.next_epoch), w)
        w = self.resize(w, 0)
        self.assertEqual(w.extents, ())
        w, _, _ = self.call(w, m.fork)
        self.assertEqual(w.extents, ())
        w, _, _ = self.call(w, m.child_exit, 0)
        self.assertEqual(w.extents, ())

    def test_prepared_child_abandonment_keeps_extent_issuance_consumed(self):
        w = self.write(self.resize(m.initial(), 35), 34 * 4096, b'kept')
        failed, value, events = self.call(w, m.fork, preparation_unavailable=True)
        self.assertEqual(value, -12)
        self.assertEqual(failed.next_token, w.next_token + 1)
        self.assertEqual((failed.next_pid, failed.next_mm), (w.next_pid, w.next_mm))
        self.assertEqual(replace(failed, next_token=w.next_token, next_epoch=w.next_epoch), w)
        allocated = [e.token for e in events if e.kind == 'allocate']
        released = [e.token for e in events if e.kind == 'release']
        self.assertEqual(allocated, released)
        w, _, _ = self.call(failed, m.fork)
        self.assertGreater(w.process().heap, allocated[0])

    def test_fragmented_pool_miss_preserves_active_and_inactive_heaps(self):
        w = self.resize(m.initial(m.Geometry(frames=100)), 20)
        w, _, _ = self.call(w, m.fork)
        w = self.resize(w, 30)
        self.assertEqual(sum(e.count for e in w.extents), 50)
        after, result, events = self.call(w, m.brk, w.geometry.start + 40 * 4096)
        self.assertEqual(result, w.process().program_break)
        self.assertEqual(events, ())
        self.assertEqual(replace(after, next_epoch=w.next_epoch), w)

    def test_exec_invalidates_old_mm_entry_even_for_same_pid(self):
        w, entry = m.enter(self.resize(m.initial(), 35))
        old_mm = entry.address_space
        w, _, _ = m.exec_reset(w, entry)
        self.assertNotEqual(w.process().address_space, old_mm)
        w, current = m.enter(w)
        self.assertEqual(current.pid, entry.pid)
        with self.assertRaises(m.InvalidState):
            m.brk(w, entry, 0)

    def test_wait_parking_eligibility_fault_retry_and_recreation(self):
        w = self.resize(m.initial(), 35)
        w, first, _ = self.call(w, m.fork)
        w, _, _ = self.call(w, m.handoff)
        w1, result, events = self.call(w, m.wait, nohang=True)
        self.assertEqual(result, 0)
        self.assertEqual(events, ())
        w, result, _ = self.call(w1, m.wait)
        self.assertIsNone(result)
        self.assertEqual(w.active, 2)
        self.assertTrue(w.waiting)
        w, _, events = self.call(w, m.handoff)
        self.assertEqual(w.active, 2)
        self.assertEqual(events, ())
        w, _, _ = self.call(w, m.child_exit, 511)
        w1, result, events = self.call(w, m.wait, output_valid=False)
        self.assertEqual(result, -14)
        self.assertEqual(events, ())
        self.assertEqual((w1.child, w1.status), (first, 255))
        w, result, _ = self.call(w1, m.wait)
        self.assertEqual(result, first)
        w, second, _ = self.call(w, m.fork)
        self.assertEqual(second, first + 1)

    def test_outer_cancellation_releases_both_heaps_once(self):
        w = self.resize(m.initial(), 67)
        w, _, _ = self.call(w, m.fork)
        w = self.resize(w, 101)
        w, _, _ = self.call(w, m.handoff)
        tokens = {e.token for e in w.extents}
        w, _, events = self.call(w, m.terminate_outer)
        released = [e.token for e in events if e.kind == 'release']
        self.assertEqual(set(released), tokens)
        self.assertEqual(len(released), len(tokens))
        self.assertEqual((w.processes, w.extents, w.active), ((), (), None))
        with self.assertRaises(m.InvalidState):
            m.enter(w)

    def test_entry_and_copy_loans_cannot_cross_switch_resize_or_exec(self):
        w = self.resize(m.initial(), 35)
        w, _, _ = self.call(w, m.fork)
        w, entry = m.enter(w)
        w, loan = m.borrow(w, entry, w.geometry.start + 34 * 4096, 4, True)
        for operation, args in [(m.handoff, ()), (m.exec_reset, ()),
                                (m.child_exit, (0,)), (m.brk, (w.geometry.start,))]:
            with self.subTest(operation=operation.__name__), self.assertRaises(m.InvalidState):
                operation(w, entry, *args)
        with self.assertRaises(m.InvalidState):
            m.access(w, replace(loan))
        w = m.end_loan(w, loan)
        after, _, events = m.handoff(w, entry)
        check(w, after, events)
        after, new_entry = m.enter(after)
        with self.assertRaises(m.InvalidState):
            m.brk(after, entry, 0)
        with self.assertRaises(m.InvalidState):
            m.borrow(after, replace(new_entry), after.geometry.start, 1)
        with self.assertRaises(m.InvalidState):
            m.access(after, loan)

    def test_zero_length_no_touch_and_real_buffer_bounds(self):
        w, entry = m.enter(m.initial())
        w, loan = m.borrow(w, entry, (1 << 64) - 1, 0)
        self.assertEqual(m.access(w, loan)[1], b'')
        w = m.end_loan(w, loan)
        for address, count in [(w.geometry.start, 1), ((1 << 64) - 1, 2)]:
            with self.assertRaises(m.InvalidState):
                m.borrow(w, entry, address, count)
        w = m.leave(w, entry)
        w = self.resize(w, 35)
        w, entry = m.enter(w)
        with self.assertRaises(m.InvalidState):
            m.borrow(w, entry, w.geometry.start + 35 * 4096 - 1, 2)
        w, loan = m.borrow(w, entry, w.geometry.start, 1)
        with self.assertRaises(m.InvalidState):
            m.access(w, loan, b'x')

    def test_wrong_state_controls_are_model_only(self):
        w = self.resize(m.initial(), 35)
        w, _, _ = self.call(w, m.fork)
        parent, child = w.process(1), w.process(2)
        bad = [replace(w, processes=(parent, replace(child, heap=parent.heap))),
               replace(w, processes=(parent, replace(child, program_break=child.program_break + 4096))),
               replace(w, extents=w.extents + (w.extents[0],)),
               replace(w, next_token=1), replace(w, active=99), replace(w, child=None),
               replace(w, extents=(w.extents[0], replace(w.extents[1], first=w.extents[0].first)))]
        for state in bad:
            with self.subTest(state=bad.index(state)), self.assertRaises(m.InvalidState):
                m.validate(state)

    def test_saved_event_controls_reject_missing_reordered_or_foreign_effects(self):
        w = self.resize(m.initial(), 35)
        before, entry = m.enter(w)
        after, _, events = m.brk(before, entry, w.geometry.start + 67 * 4096)
        check(before, after, events)
        # These controls only alter immutable model histories, never guest code.
        kinds = [e.kind for e in events]
        self.assertEqual(kinds, ['allocate', 'zero', 'pin', 'publish', 'invalidate', 'unpin', 'erase', 'release'])
        bad = [events[:i] + events[i + 1:] for i in range(len(events))]
        bad += [events[:i] + (events[i + 1], events[i]) + events[i + 2:] for i in range(len(events) - 1)]
        bad += [events + (events[-1],),
                events[:3] + (replace(events[3], pid=99),) + events[4:],
                events[:4] + (replace(events[4], frames=events[4].frames[:-1]),) + events[5:],
                events[:6] + (replace(events[6], data=b'x' + events[6].data[1:]),) + events[7:]]
        for i, history in enumerate(bad):
            with self.subTest(control=i), self.assertRaises(m.InvalidState):
                check(before, after, history)

    def test_geometry_is_not_x86_snapshot_or_page_size_policy(self):
        for page, count in [(4096, 67), (16384, 73), (65536, 35)]:
            g = m.Geometry(page=page, start=page * 100, pages=80, frames=320)
            w = self.write(self.resize(m.initial(g), count), (count - 1) * page, b'end')
            w, _, _ = self.call(w, m.fork)
            self.assertEqual(self.contents(w), self.contents(w, 1))
            w = self.resize(w, count + 1)
            self.assertEqual(self.contents(w)[-page:], bytes(page))
            self.assertEqual(w.process(1).program_break, g.start + count * page)

    def test_seeded_application_oracle_across_independent_mm_lifetimes(self):
        # Oracle tracks only process-local bytes, break and generation. No
        # frame placement, token, pin, allocator or event algorithm is shared.
        for seed in range(32):
            random_source = random.Random(seed)
            g = m.Geometry(page=64, start=6400, pages=80, frames=400)
            w = self.resize(m.initial(g), 35)
            oracle = {1: [bytearray(35 * g.page), g.start + 35 * g.page, 0]}
            w, child_pid, _ = self.call(w, m.fork)
            oracle[child_pid] = [oracle[1][0].copy(), oracle[1][1], 0]
            for step in range(100):
                pid = w.active
                data, old_break, generation = oracle[pid]
                action = random_source.randrange(4)
                if action == 0:
                    count = random_source.randrange(81)
                    new_break = g.start + count * g.page
                    w, result, _ = self.call(w, m.brk, new_break)
                    self.assertEqual(result, new_break)
                    wanted = count * g.page
                    oracle[pid] = [data[:wanted] + bytearray(max(0, wanted - len(data))), new_break, generation]
                elif action == 1 and data:
                    offset = random_source.randrange(len(data))
                    value = random_source.randrange(256)
                    w = self.write(w, offset, bytes([value]))
                    data[offset] = value
                elif action == 2:
                    w, _, _ = self.call(w, m.exec_reset)
                    oracle[pid] = [bytearray(), g.start, generation + 1]
                else:
                    w, _, _ = self.call(w, m.handoff)
                for expected_pid, expected in oracle.items():
                    process = w.process(expected_pid)
                    self.assertEqual([self.contents(w, expected_pid), process.program_break, process.generation], expected,
                                     (seed, step, expected_pid))
            if w.active == 1:
                w, _, _ = self.call(w, m.handoff)
            w, _, _ = self.call(w, m.child_exit, seed)
            self.assertEqual(self.contents(w), oracle[1][0])
            w, _, _ = self.call(w, m.wait)
            w, _, _ = self.call(w, m.terminate_outer)
            self.assertEqual(w.extents, ())


if __name__ == '__main__':
    unittest.main()
