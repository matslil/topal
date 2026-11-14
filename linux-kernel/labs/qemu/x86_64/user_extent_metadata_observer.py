"""Read-only complete selected metadata deltas and actual clock/caller joins."""
from pathlib import Path

MODEL=(Path(__file__).resolve().parents[3]/'tools/user-memory/metadata_model.py').read_text()
PROGRAM='metadata_model = {}\nexec('+repr(MODEL)+', metadata_model)\n'+r'''
METADATA_BASE=0x43000
METADATA_BYTES=0x9000

def metadata_deltas(before,after):
    assert len(before)==len(after)
    changes=[]
    at=0
    while at<len(before):
        if before[at]==after[at]:
            at+=1
            continue
        start=at
        while at<len(before) and before[at]!=after[at]:at+=1
        changes.append(dict(offset=start,before=before[start:at].hex(),after=after[start:at].hex()))
    return changes

def metadata_begin(row):
    table=word(context+928)
    assert word(table)==1
    row['metadata_initial_table']=memory(table,8192).hex()
    row['metadata_initial_filesystem']=memory(table+METADATA_BASE,METADATA_BYTES).hex()

def metadata_path(pointer):
    raw=pipe_visible(pointer,256)
    return raw[:raw.index(0)].hex() if b'\0' in raw else None

def metadata_request(call):
    table=word(context+928)
    call['_metadata_table']=memory(table,8192).hex()
    call['_metadata_fs']=memory(table+METADATA_BASE,METADATA_BYTES).hex()
    call['metadata_context_before']=pipe_context()
    call['metadata_clocks']=[]
    n,args=call['number'],call['args']
    pointers=([args[1]] if n==257 else [args[0]] if n in [90,87] else args[:2] if n in [82,86] else [])
    call['metadata_paths']=[metadata_path(pointer) for pointer in pointers]
    call['metadata_payload']=pipe_visible(args[1],args[2]).hex() if n==18 else ''
    if n==5:stat_request(call)

class MetadataClock(Checked):
    def __init__(self,row):
        super().__init__('*'+hex(extent_inventory['address']+row['end']),internal=True)
        self.row=row

    def observe(self):
        if current is not None and component.get('mode')=='metadata':
            assert pending is not None and 'metadata_clocks' in pending
            assert reg('cs')&3==0 and not reg('eflags')&(1<<9)
            pending['metadata_clocks'].append(dict(block=self.row['name'],
                call_offset=self.row['previous_offset'],seconds=reg('rax'),carry=reg('eflags')&1))
        return False

def install_metadata_observation():
    if component.get('mode')!='metadata':return
    audit=extent_inventory['stack_retirement_audit']
    assert audit['passed'] and audit['kernel_sha256']==extent_inventory['kernel_sha256']
    callers=[row for row in audit['blocks'] if row['name'].startswith('clock-caller-')]
    assert len(callers)==9
    for row in callers:MetadataClock(row)

def metadata_result(call):
    table=word(context+928)
    before_table=bytes.fromhex(call.pop('_metadata_table'))
    before_fs=bytes.fromhex(call.pop('_metadata_fs'))
    clocks=call['metadata_clocks']
    mutation=call['result']>=0 and call['number'] in [257,18,77,91,82,86,87]
    assert len(clocks)==int(mutation), ('metadata clock count',call)
    if clocks:assert clocks[0]['carry']==0
    seconds=clocks[0]['seconds'] if clocks else None
    paths=[bytes.fromhex(path) if path is not None else None for path in call['metadata_paths']]
    expected_fs,expected_table=metadata_model['transition'](before_fs,before_table,
        call['number'],call['args'],call['result'],paths,seconds,bytes.fromhex(call['metadata_payload']))
    actual_fs,actual_table=memory(table+METADATA_BASE,METADATA_BYTES),memory(table,8192)
    assert actual_fs==expected_fs, ('complete metadata filesystem delta',call['number'],
        metadata_deltas(expected_fs,actual_fs)[:8])
    assert actual_table==expected_table, ('complete metadata descriptor/table delta',call['number'],
        metadata_deltas(expected_table,actual_table)[:8])
    assert pipe_context()==call['metadata_context_before']
    call['metadata_filesystem_delta']=metadata_deltas(before_fs,actual_fs)
    call['metadata_table_delta']=metadata_deltas(before_table,actual_table)
    call['metadata_filesystem_sha256']=hashlib.sha256(actual_fs).hexdigest()
    call['metadata_table_sha256']=hashlib.sha256(actual_table).hexdigest()
    call['complete_metadata_delta_verified']=True
    if call['number']==5:stat_result(call)

def metadata_returned(row,after):
    global current
    calls=row['calls']
    expected=[12,12,90,257,5,5,18,8,5,77,5,91,91,5,86,5,82,87,5,257,5,18,8,5,91,86,82,87,5,3,5,3,87,12]
    assert [call['number'] for call in calls]==expected
    actions=[call for call in calls if call['number']!=12]
    assert all(call['complete_metadata_delta_verified'] for call in actions)
    assert [call['result'] for call in actions if call['result']<0]==[-14,-9,-17,-14,-2]
    assert sum(len(call['metadata_clocks']) for call in actions)==11
    table=word(context+928)
    assert memory(table,8192)==bytes.fromhex(row['metadata_initial_table'])
    before=bytes.fromhex(row['metadata_initial_filesystem'])
    expected=bytearray(before)
    struct.pack_into('<Q',expected,0,struct.unpack_from('<Q',before)[0]+1)
    last=next(call['metadata_clocks'][0]['seconds'] for call in reversed(actions) if call['number']==87 and call['result']==0)
    struct.pack_into('<QQ',expected,48,last,last)
    assert memory(table+METADATA_BASE,METADATA_BYTES)==bytes(expected), 'complete named-inode/page/name reclamation'
    row['metadata_retirement_verified']=True
    row['after']=after
    observed['executions'].append(row)
    print('SOURCE_METADATA_CASES_PASS',len(observed['executions']),len(calls),flush=True)
    current=None
    return False
'''
