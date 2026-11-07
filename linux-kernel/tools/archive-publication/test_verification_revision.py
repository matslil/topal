"""A verifier revision cannot authorize altered runtime experiment inputs."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import verification_revision as REV


class VerificationRevision(unittest.TestCase):
    def fixture(self,root):
        cohort=root/'cohort'
        (cohort/'sources').mkdir(parents=True)
        archive=root/'archive'
        archive.mkdir()
        entries=[]
        for name in [*REV.ALLOWED,REV.PREFIX+'namespace_gate.py']:
            original=('original '+name).encode()
            current=original+b' corrected verifier' if name in REV.ALLOWED else original
            target=root/name
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes(current)
            snapshot='sources/'+Path(name).name
            (cohort/snapshot).write_bytes(original)
            entries.append(dict(source=name,snapshot=snapshot,sha256=REV.sha(original)))
        (cohort/'cohort.json').write_text(json.dumps(dict(complete=True,passed=True,
            head='unit-only-fixture',sources=entries)))
        failed=root/'failed.log'
        failed.write_bytes(b'unit-only previous verifier failure')
        return cohort,archive,failed,entries

    def test_preserves_original_sources_and_binds_only_postrun_corrections(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            cohort,archive,failed,entries=self.fixture(root)
            with patch.object(REV,'ROOT',root): REV.create(cohort,archive,failed)
            manifest=archive/'verification-revision.json'
            value=REV.validate(entries,root,lambda name:(cohort/name).read_bytes(),manifest)
            self.assertEqual(set(value['changes']),set(REV.ALLOWED))
            self.assertEqual((archive/'verification-failures.log').read_bytes(),failed.read_bytes())
            target=root/next(iter(REV.ALLOWED))
            target.write_bytes(target.read_bytes()+b' unrecorded change')
            with self.assertRaises(AssertionError):
                REV.validate(entries,root,lambda name:(cohort/name).read_bytes(),manifest)

    def test_runtime_change_or_damaged_original_snapshot_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            cohort,archive,failed,entries=self.fixture(root)
            gate=root/REV.PREFIX/'namespace_gate.py'
            original=gate.read_bytes()
            gate.write_bytes(original+b' altered actual VM driver')
            with patch.object(REV,'ROOT',root), self.assertRaises(AssertionError):
                REV.create(cohort,archive,failed)
            self.assertFalse((archive/'verification-revision.json').exists())
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            cohort,archive,failed,entries=self.fixture(root)
            with patch.object(REV,'ROOT',root): REV.create(cohort,archive,failed)
            snapshot=cohort/entries[-1]['snapshot']
            snapshot.write_bytes(b'damaged original')
            with self.assertRaises(AssertionError):
                REV.validate(entries,root,lambda name:(cohort/name).read_bytes(),archive/'verification-revision.json')


if __name__=='__main__': unittest.main()
