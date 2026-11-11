"""Observe ordinary UART consumption and user copies without guest assignments."""
import importlib.util
import json
from pathlib import Path
import re

DIALOGUE = ((b'TOPAL_SOURCE_SERIAL_A\n',b'Uabcdefg'),
            (b'TOPAL_SOURCE_SERIAL_B\n',b'12345678'))


def native_sites(build):
    path=Path(__file__).resolve().parents[3]/'tools/user-memory/inspect_call_inventory.py'
    spec=importlib.util.spec_from_file_location('serial_elf_reader',path)
    reader=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    inventory=json.loads((build/'extent-call-inventory.json').read_text())
    rows=[r for r in inventory['calls'] if 'fd-serial-count-ready' in r['nearest_labels']]
    assert len(rows)==1 and rows[0]['target']=='runtime-writable-extent'
    elf=reader.Elf(build/'kernel.elf')
    symbol,body=elf.function(inventory['function'])
    call=rows[0]['offset']
    signature=bytes.fromhex('66 ba f8 03 ec 88 06')
    matches=[i for i in range(call+14,min(call+1024,len(body)-len(signature)+1))
             if body[i:i+len(signature)]==signature]
    assert len(matches)==1
    at=matches[0]
    provider=reader.Elf(build/'provider.o')
    original,_=provider.function(inventory['function'])
    decoded={int(m.group(1),16):m.group(2) for line in
             (build/'extent-call-disassembly.stdout.log').read_text().splitlines()
             if (m:=re.match(r'^\s*([0-9a-f]+):\s+(.*)$',line))}
    assert 'inb' in decoded[original['value']+at+4]
    assert '%dx' in decoded[original['value']+at+4]
    assert 'movb' in decoded[original['value']+at+5]
    assert '%al' in decoded[original['value']+at+5] and '%rsi' in decoded[original['value']+at+5]
    result=dict(admission=symbol['value']+call,byte_received=symbol['value']+at+5,
                admission_offset=call,byte_received_offset=at+5,
                kernel_sha256=reader.digest(elf.data),
                decoded_in=decoded[original['value']+at+4],
                decoded_store=decoded[original['value']+at+5])
    (build/'serial-native-sites.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


PROGRAM = r'''
class SerialAdmission(Checked):
    def observe(self):
        assert current and pending and pending['number']==0 and pending['args'][0]==0
        assert reg('cs')&3==0 and reg('cr0')&(1<<16)
        assert reg('r8')==pending['args'][1] and reg('r9')==min(pending['args'][2],256)
        assert 'extent_call' not in pending
        pending['extent_call']=dict(candidate=reg('r8'),length=reg('r9'))
        return False

class SerialByte(Checked):
    def observe(self):
        assert current and pending and pending['number']==0 and pending['args'][0]==0
        assert reg('cs')&3==0 and reg('cr0')&(1<<16) and reg('rdx')&0xffff==0x3f8
        assert 'extent_call' in pending
        assert reg('rsi')==word(context+928)+1408+len(pending['uart_bytes'])
        pending['uart_bytes'].append(reg('rax')&255)
        assert len(pending['uart_bytes'])<=pending['extent_call']['length']
        return False

def serial_request(call):
    table=word(context+928)
    assert word(table)==1 and memory(table+1408,256)==bytes(256)
    call['table_before_sha256']=hashlib.sha256(memory(table,8192)).hexdigest()
    call['context_before']=pipe_context()
    n,args=call['number'],call['args']
    assert n in [0,1]
    call['uart_bytes']=[]
    size=min(args[2],256)
    call['buffer_before']=pipe_visible(args[1],size).hex()
    call['window_pointer']=args[1]-1
    call['window_before']=pipe_visible(args[1]-1,size+2).hex()

def serial_result(call):
    table=word(context+928)
    n,args,result=call['number'],call['args'],call['result']
    output=b''
    data=bytes(call['uart_bytes'])
    expected=b''.join(bytes.fromhex(payload) for _,payload in serial_patterns)
    if n==1:
        marker=bytes.fromhex(serial_patterns[current['serial_markers']][0])
        assert args[0]==1 and result==args[2]==len(marker)
        assert bytes.fromhex(call['buffer_before'])==marker and not data
        current['serial_markers']+=1
    elif result<0:
        assert result in [-14,-9] and not data
        assert pipe_visible(args[1],len(bytes.fromhex(call['buffer_before']))).hex()==call['buffer_before']
    elif result==0:
        assert args[0]==0 and args[2]==0 and not data and 'extent_call' not in call
    else:
        assert args[0]==0 and 0<result<=min(args[2],256) and len(data)==result
        start=len(current['serial_received'])
        assert data==expected[start:start+result]
        assert memory(args[1],result)==data
        current['serial_received']+=list(data)
        output=data
        call['output_hex']=data.hex()
    window=bytearray.fromhex(call['window_before'])
    if output:
        assert 1+len(output)<=len(window)
        window[1:1+len(output)]=output
    assert pipe_visible(call['window_pointer'],len(window))==bytes(window)
    assert hashlib.sha256(memory(table,8192)).hexdigest()==call['table_before_sha256']
    assert pipe_context()==call['context_before']
    assert call['phase_after']==call['phase_before']
    call['copy_window_verified']=True
    call['complete_table_verified']=True
    call['native_input_verified']=True

def serial_returned(row,after):
    global current
    calls=row['calls']
    assert 16<=len(calls)<=29
    assert sum(c['number']==12 for c in calls)==3 and sum(c['number']==1 for c in calls)==2
    assert all(c['number'] in [0,1,12] for c in calls)
    assert sum(c['result']==-14 for c in calls)==5
    assert sum(c['result']==-9 for c in calls)==2
    assert row['serial_markers']==2
    assert bytes(row['serial_received'])==b'Uabcdefg12345678'
    requests=[c for c in calls if c['number']!=12]
    assert all(c['copy_window_verified'] and c['complete_table_verified'] and c['native_input_verified'] for c in requests)
    large=[c for c in requests if c['number']==0 and c['args'][2]==512]
    assert len(large)==1 and large[0]['extent_call']==dict(candidate=0x639f00,length=256)
    assert hashlib.sha256(memory(word(context+928),8192)).hexdigest()==row['serial_table_before_sha256']
    row['after']=after
    row['serial_input_verified']=True
    observed['executions'].append(row)
    print('SOURCE_SERIAL_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False

def install_serial_observation():
    if component.get('mode')=='serial':
        SerialAdmission('*'+hex(serial_sites['admission']),internal=True)
        SerialByte('*'+hex(serial_sites['byte_received']),internal=True)
'''
