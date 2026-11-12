"""Host-only fresh-read controls for the publication metadata snapshot.

TOPAL-SYSTEMS-PROCESS-MM-001. The fake memory is test data, not native allocation,
mapping or access authority.
"""
from copy import deepcopy
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest

from test_publication import fixture

sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'labs/qemu/x86_64'))
from process_mm_publication_observer import PROGRAM


class PublicationCaptureControls(unittest.TestCase):
    def setup_capture(self):
        value=fixture()['before'];value['allocator'][4]=0x700000;value['allocator'][6]=0x800000
        names=['frame_mm_tokens','frame_root_tokens','frame_context','frame_mm_state',
            'process_mm_ids','frame_mm_epoch','process_mm_inactive_state','process_mm_prepared_state',
            'process_mm_prepared_owner','process_mm_transition','process_mm_inactive_token','process_mm_prepared_token']
        symbols={'topal_x86_'+name+'_v1':0x300000+i*0x1000 for i,name in enumerate(names)}
        memory={};reads=[]
        def put(pointer,words):memory.update({pointer+i*8:word for i,word in enumerate(words)})
        def words(pointer,count):
            reads.append((pointer,count))
            return [memory.get(pointer+i*8,0) for i in range(count)]
        context=value['context']['address'];header=[0]*16
        for index,key in [(0,'state'),(2,'bootstrap'),(3,'root'),(14,'low')]:header[index]=value['context'][key]
        header[15]=value['state'][3];put(context,header)
        put(symbols['topal_x86_frame_context_v1'],value['allocator'])
        for symbol,key in [('frame_mm_tokens','tokens'),('frame_mm_state','state'),('process_mm_ids','ids'),
                ('frame_mm_epoch','epoch'),('process_mm_inactive_state','inactive_state'),
                ('process_mm_prepared_state','prepared_state'),('process_mm_prepared_owner','prepared_owner'),
                ('process_mm_transition','transition')]:put(symbols['topal_x86_'+symbol+'_v1'],value[key])
        put(symbols['topal_x86_process_mm_inactive_token_v1'],value['inactive']['token'])
        put(symbols['topal_x86_process_mm_prepared_token_v1'],value['prepared']['token'])
        root_tokens=[0]*42
        for i,row in enumerate(value['root_owners']):root_tokens[i*3:i*3+3]=row['token']
        put(symbols['topal_x86_frame_root_tokens_v1'],root_tokens)
        for row in [value['active'],value['candidate'],value['inactive'],value['prepared']]+value['root_owners']:
            if row['token']==[0]*3:continue
            put(value['allocator'][6]+row['token'][2]*64,row['record'])
            put(value['allocator'][4]+128+row['record'][2]*24,row['cells'])
        for name,data in value['tables'].items():put(int(name),data)
        registers={key:value[key] for key in ['cr0','cr3','cr4','cs']};registers['eflags']=value['flags']
        scope=dict(context=context,token_bytes=336,kernel_spans=value['kernel_spans'],words=words,
            address=lambda name:symbols[name],reg=lambda name:registers[name],
            gdb=SimpleNamespace(selected_thread=lambda:SimpleNamespace(ptid=value['thread'])))
        marker='def publication_allocation('
        assert PROGRAM.count(marker)==1
        source=marker+PROGRAM.split(marker)[1].split('class PublicationChecked')[0]
        exec(source,scope)
        return value,scope,memory,reads,symbols

    def test_bulk_snapshot_preserves_every_selected_owner_and_table(self):
        value,scope,_,reads,symbols=self.setup_capture()
        actual=scope['publication_capture']()
        expected=deepcopy(value);expected['context']['heap']=value['state'][3]
        self.assertEqual(actual,expected)
        self.assertEqual(reads.count((symbols['topal_x86_frame_context_v1'],16)),1)
        self.assertEqual(reads.count((symbols['topal_x86_frame_root_tokens_v1'],42)),1)

    def test_later_snapshot_reads_changed_live_metadata(self):
        value,scope,memory,_,symbols=self.setup_capture()
        first=scope['publication_capture']()
        memory[symbols['topal_x86_process_mm_ids_v1']+16]=19
        memory[scope['context']+120]=0x3500000
        table=next(iter(value['tables']));memory[int(table)+8*8]=0x123456
        second=scope['publication_capture']()
        self.assertEqual(first['ids'][2],0);self.assertEqual(second['ids'][2],19)
        self.assertNotEqual(first['context']['heap'],second['context']['heap'])
        self.assertEqual(second['tables'][table][8],0x123456)

    def test_invalid_registry_index_is_rejected_before_its_dereference(self):
        value,scope,_,reads,_=self.setup_capture()
        token=[1,2,value['allocator'][3]]
        forbidden=value['allocator'][6]+token[2]*64
        with self.assertRaises(AssertionError):scope['publication_allocation'](token,value['allocator'])
        self.assertFalse(any(pointer==forbidden for pointer,_ in reads))
