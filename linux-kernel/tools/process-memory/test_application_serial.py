"""Existing application contracts checked against retained real VM consoles.

TOPAL-SYSTEMS-PROCESS-MM-001: saved console controls are host-only evidence for
the shared checker, not fresh application execution or native memory authority.
"""
import gzip
from pathlib import Path
import sys
import unittest

LINUX=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(LINUX/'labs/qemu/x86_64'))
import topal_gate

CORPUS=LINUX/'tools/tail-traversal/results/stage-4-frame-production/components'


def console(name):
    return gzip.decompress((CORPUS/'production-busybox/ownership'/('application-'+name)/'serial.log.gz').read_bytes())


class ExistingApplicationSerialContract(unittest.TestCase):
    def check(self,serial,name='true'):
        topal_gate.verify_process_serial(serial,external=True,process_workload=name)

    def test_all_unchanged_recorded_workloads_keep_their_contract(self):
        self.assertEqual(len(topal_gate.PROCESS_WORKLOADS),51)
        for name in topal_gate.PROCESS_WORKLOADS:
            with self.subTest(workload=name):self.check(console(name),name)

    def test_recorded_clean_abi_keeps_its_distinct_contract(self):
        serial=gzip.decompress((CORPUS/'production-abi/clean-abi/serial.log.gz').read_bytes())
        topal_gate.verify_process_serial(serial,external=False,process_workload='true')

    def test_changed_output_is_rejected_without_changing_line_count(self):
        serial=console('echo')
        self.assertIn(b'alpha beta\n',serial)
        with self.assertRaisesRegex(topal_gate.GateError,'output or observed exit status'):
            self.check(serial.replace(b'alpha beta\n',b'alpha zeta\n',1),'echo')

    def test_changed_exit_status_is_rejected(self):
        serial=console('false')
        self.assertIn(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=01',serial)
        with self.assertRaisesRegex(topal_gate.GateError,'output or observed exit status'):
            self.check(serial.replace(b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=01',b'TOPAL_KERNEL_PROCESS_EXIT_STATUS=00'),'false')

    def test_extra_successful_exec_is_rejected(self):
        with self.assertRaisesRegex(topal_gate.GateError,'image-replacement count'):
            self.check(console('true')+topal_gate.EXEC_COMMIT_MARKER)

    def test_missing_prelude_or_extra_output_is_rejected(self):
        serial=console('true')
        with self.assertRaisesRegex(topal_gate.GateError,'BusyBox read and close output'):
            self.check(serial.replace(b'busybox-read\n',b'busybox-xxxx\n',1))
        with self.assertRaisesRegex(topal_gate.GateError,'newline count'):
            self.check(serial+b'extra\n')


if __name__=='__main__':unittest.main()
