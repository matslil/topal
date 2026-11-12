"""Serial receive/count counterexamples; no native ownership or lifetime grant."""
from pathlib import Path
import runpy
import unittest

from audit_serial_prefix import prefix_step


class SerialPrefixControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_copy_observer.py'
        program = runpy.run_path(str(path))['PROGRAM']
        namespace = dict(Checked=type('Breakpoint',(),{}))
        exec(program,namespace)
        cls.transfer_size = staticmethod(namespace['copy_transfer_size'])

    def prefix(self,maximum=256):
        return dict(copy_width=1,copy_count_kind='positive-prefix',admitted_maximum=maximum)

    def test_every_bounded_receive_state_stores_inside_the_staging_extent(self):
        for count in range(1,257):
            for received in range(count):
                following = prefix_step(count,received)
                self.assertEqual(following['store_index'],received)
                self.assertEqual(following['received'],received+1)
                self.assertEqual(following['may_receive_more'],received+1<count)

    def test_zero_unbounded_and_exhausted_states_cannot_receive_a_byte(self):
        for count,received in [(0,0),(257,0),(1,1),(256,256),(4,-1)]:
            with self.subTest(count=count,received=received),self.assertRaises(AssertionError):
                prefix_step(count,received)

    def test_every_short_and_complete_prefix_fits_the_actual_admission(self):
        for count in range(1,257):
            for received in range(1,count+1):
                self.assertEqual(self.transfer_size(self.prefix(),received,count),received)

    def test_prefix_rejects_zero_overrun_and_uncapped_admission(self):
        for received,admitted in [(0,4),(5,4),(1,0),(1,257),(1,1<<64),(-1,4)]:
            with self.subTest(received=received,admitted=admitted),self.assertRaises(AssertionError):
                self.transfer_size(self.prefix(),received,admitted)

    def test_ordinary_exact_copies_keep_their_original_count_contract(self):
        row = dict(copy_width=8)
        self.assertEqual(self.transfer_size(row,2,16),16)
        self.assertEqual(self.transfer_size(row,0,0),0)
        with self.assertRaises(AssertionError): self.transfer_size(row,1,16)

    def test_unknown_count_contracts_and_target_word_overflow_fail_closed(self):
        for row,count,admitted in [(dict(copy_width=1,copy_count_kind='unchecked'),1,1),
                (dict(copy_width=8),1<<63,0),(dict(copy_width=0),0,0)]:
            with self.subTest(row=row),self.assertRaises(AssertionError):
                self.transfer_size(row,count,admitted)


if __name__ == '__main__':
    unittest.main()
