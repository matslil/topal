"""Whole-byte round trips and failure checks for external proof dependencies."""
import gzip
import json
from pathlib import Path
import tempfile
import unittest

import archive_publication as PUB


class ArchivePublication(unittest.TestCase):
    def fixture(self,root):
        archive=root/'archive'
        archive.mkdir()
        (archive/'blobs').mkdir()
        payload=b'EXTERNAL-TEST-PAYLOAD\x00\xff'
        data={'external':payload,'duplicate':payload,
            'bundle':b'owned header\x00'+payload+b'owned separator'+payload+b'owned trailer',
            'observed':b'owned physical evidence\x00'}
        index={'schema':'unit-archive','files':{}}
        for name,value in data.items():
            key=PUB.digest(value)
            blob='blobs/'+key+'.gz'
            (archive/blob).write_bytes(gzip.compress(value,mtime=0))
            index['files'][name]=dict(sha256=key,bytes=len(value),blob=blob)
        PUB.write_json(archive/'files.json',index)
        PUB.write_json(archive/'qualification.json',dict(schema='unit-record',
            scope='unit fixture only',archive_index_sha256=PUB.digest((archive/'files.json').read_bytes()),
            archived_files=4,unique_blobs=3))
        return archive,payload,data

    def test_full_bytes_and_producer_records_survive_external_cache_loss(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            archive,payload,originals=self.fixture(root)
            index_before=(archive/'files.json').read_bytes()
            record_before=(archive/'qualification.json').read_bytes()
            cache=root/'external-cache'
            result=PUB.publish(archive,payload,cache,PUB.digest(payload))
            self.assertEqual((result['logical_files'],result['logical_blobs'],len(result['external_blobs'])),(4,3,2))
            self.assertEqual((archive/'raw-files.json').read_bytes(),index_before)
            self.assertEqual((archive/'raw-qualification.json').read_bytes(),record_before)
            self.assertEqual(json.loads((archive/'qualification.json').read_text())['scope'],'unit fixture only')
            for blob in (archive/'blobs').glob('*.gz'):
                self.assertNotIn(payload,gzip.decompress(blob.read_bytes()))
            # Recreate a clean checkout's external cache using only the exact
            # dependency plus committed first-party literal wrapper metadata.
            cache.rename(root/'old-cache')
            PUB.hydrate(archive,payload,cache)
            index=json.loads((archive/'files.json').read_text())
            self.assertEqual(set(index['files']),set(originals))
            for name,entry in index['files'].items():
                self.assertEqual(gzip.decompress((archive/entry['blob']).read_bytes()),originals[name])
            self.assertEqual(len(list(cache.glob('*.gz'))),2)

    def test_wrong_payload_or_raw_blob_digest_fails_before_publication(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            archive,payload,_=self.fixture(root)
            before=(archive/'files.json').read_bytes()
            with self.assertRaises(AssertionError):
                PUB.publish(archive,b'wrong input',root/'cache',PUB.digest(payload))
            first=next((archive/'blobs').glob('*.gz'))
            first.write_bytes(gzip.compress(b'corrupted raw evidence',mtime=0))
            with self.assertRaises(AssertionError):
                PUB.publish(archive,payload,root/'cache',PUB.digest(payload))
            self.assertEqual((archive/'files.json').read_bytes(),before)
            self.assertFalse((archive/'publication-projection.json').exists())
            self.assertFalse((archive/'raw-qualification.json').exists())

    def test_altered_reconstruction_or_cache_destination_is_refused(self):
        for alteration in ('recipe','destination'):
            with self.subTest(alteration=alteration), tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp)
                archive,payload,_=self.fixture(root)
                cache=root/'cache'
                PUB.publish(archive,payload,cache,PUB.digest(payload))
                projection_path=archive/'publication-projection.json'
                projection=json.loads(projection_path.read_text())
                key=next(iter(projection['external_blobs']))
                entry=projection['external_blobs'][key]
                if alteration=='recipe':
                    entry['parts'].append(dict(literal_gzip_base64=PUB.encode(b'forged suffix')))
                else:
                    entry['blob']='../outside-cache/'+key+'.gz'
                PUB.write_json(projection_path,projection)
                record=json.loads((archive/'qualification.json').read_text())
                record['publication_projection_sha256']=PUB.digest(projection_path.read_bytes())
                PUB.write_json(archive/'qualification.json',record)
                with self.assertRaises(AssertionError): PUB.hydrate(archive,payload,cache)
                self.assertFalse((root/'outside-cache').exists())


if __name__=='__main__': unittest.main()
