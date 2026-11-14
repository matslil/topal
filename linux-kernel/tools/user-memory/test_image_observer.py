"""Replay the observer's real repeated loop-head protocol without a guest."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest

from qemu_publication_trace import completed
from test_image_publication import fixture


class ImageObserverTests(unittest.TestCase):
    def test_store_loop_is_one_publication_and_entry_rearms_at_completion(self):
        path=Path(__file__).resolve().parents[2]/'labs/qemu/x86_64/user_extent_image_observer.py'
        spec=importlib.util.spec_from_file_location('image_observer_under_test',path)
        module=importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        class Checked:
            def __init__(self,expression,internal):
                self.enabled=True
                self.expression=expression
        row,bindings,spans=fixture('relro')
        registers={}
        captures=[]
        events=[]
        profile={kind:dict(final_barrier=kind=='relro',sites=dict(entry=base,stored=base+10,
            invalidated=base+20,finished=base+30)) for kind,base in [('relro',0x100),('exec',0x200)]}
        with tempfile.TemporaryDirectory() as directory:
            work=Path(directory)
            (work/'cpu-1.log').write_bytes(b'')
            namespace=dict(Checked=Checked,Path=Path,hashlib=hashlib,json=json,
                reg=registers.__getitem__,owner_capture=lambda:captures.pop(0),
                word=lambda at:0,context=0x200000,
                translation_active=None,translation_request=None,
                translation_trace_positions=lambda:{'cpu-1.log':0},
                translation_trace_model=dict(parse=lambda raw,site:list(events),completed=completed),
                owner_profile=dict(bindings=bindings,kernel_spans=spans),component=dict(mode='metadata'),
                image_publication_profile=dict(profiles=profile),image_publication_model_source=module.MODEL,
                translation_trace_model_source='test protocol only',observed={},observation_path=str(work/'observations.json'),
                gdb=SimpleNamespace(selected_thread=lambda:SimpleNamespace(ptid=[1,1,0])))
            exec(module.PROGRAM,namespace)
            namespace['install_image_publication_observation']()
            for kind,count in [('relro',8),('exec',18)]:
                row,_,_=fixture(kind)
                captures[:]=[row['before'],row['stored'],row['after']]
                events[:]=row['invalidations']
                entry=namespace['image_publication_entries'][kind]
                self.assertTrue(entry.enabled)
                for index in range(count if kind=='relro' else 1):
                    registers.update(rip=profile[kind]['sites']['entry'],rcx=count-index,
                        r8=row['before']['context']['low']+(485+index)*8)
                    if entry.enabled:entry.observe()
                    self.assertFalse(entry.enabled)
                for site_kind in ['stored','finished']:
                    registers.update(rip=profile[kind]['sites'][site_kind],rcx=0)
                    point=namespace['ImagePublication'](kind,site_kind,registers['rip'])
                    point.observe()
                self.assertTrue(entry.enabled)
                self.assertIsNone(namespace['image_publication_active'])
                self.assertEqual(captures,[])
            namespace['image_publication_observation_complete']()
            self.assertEqual([r['kind'] for r in namespace['image_publication_records']],['relro','exec'])


if __name__=='__main__':unittest.main()
