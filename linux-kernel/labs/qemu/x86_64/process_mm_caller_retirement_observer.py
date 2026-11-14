"""Read-only first-return samples for actual process-MM owned-extent callers.

TOPAL-SYSTEMS-USER-EXTENT-001 / TOPAL-SYSTEMS-PROCESS-MM-001. Samples do
not establish every argument/error path, continuous lifetime or public binding.
"""
import hashlib
import json
from pathlib import Path
import sys


DOMAINS = {
    'topal_x86_frame_private_v1':65528,
    'topal_x86_frame_stack_v1':4096,
    'topal_x86_frame_facts_v1':24,
    'topal_x86_frame_output_v1':56,
    'topal_x86_frame_details_v1':32,
    'topal_x86_frame_gateway_recovery_v1':48,
    'topal_frame_allocator_active_v1':8,
    'topal_x86_tail_boundary_active_v1':8,
}


def layout(build):
    sys.path.insert(0,str(Path(__file__).resolve().parents[3]/'tools/user-memory'))
    from inspect_call_inventory import ERASE_RETURN,Elf,verify_calls
    raw=(build/'extent-call-inventory.json').read_bytes();inventory=json.loads(raw)
    kernel=Elf(build/'kernel.elf');provider=Elf(build/'provider.o')
    function=verify_calls(kernel,inventory);verify_calls(provider,inventory)
    _,body=kernel.function(inventory['function'])
    calls=[]
    for row in inventory['calls']:
        at=row['offset'];assert body[at+5:at+5+len(ERASE_RETURN)]==ERASE_RETURN
        calls.append(dict(row,call=function['value']+at,returned=function['value']+at+5+len(ERASE_RETURN),
            instruction=body[at:at+5+len(ERASE_RETURN)].hex(),
            direction=int(row['target']=='runtime-writable-extent')))
    domains={name:dict(address=kernel.symbols[name]['value'],bytes=size) for name,size in DOMAINS.items()}
    return dict(schema='topal.process-mm.caller-retirement-layout/1',
        kernel_sha256=hashlib.sha256(kernel.data).hexdigest(),
        inventory_sha256=hashlib.sha256(raw).hexdigest(),function=inventory['function'],calls=calls,domains=domains,
        gateway=kernel.symbols['topal_x86_frame_gateway_v1']['value'],
        gateway_return=kernel.symbols['topal_x86_frame_gateway_return_reclaimed_v1']['value'])


PROGRAM=r'''
caller_pending=None
caller_gateways={}
caller_return_points={}
caller_registers=callee+['rdx','rsi','rdi','r8','r9']
caller_data=dict(schema='topal.process-mm.caller-retirement/1',complete=False,
    profile=caller_retirement_profile,records=[],blobs={},pending=None,
    scope='First completed normal return per observed native helper site; not every argument/path, continuous loan lifetime, fatal/exception or production qualification')
observed['caller_retirement']=caller_data
caller_original_save=save

def save():
    caller_data['pending']=caller_pending
    caller_original_save()

def caller_snapshot():
    value=capture();model['snapshot'](value)
    # Retain every observed heap byte, sharing identical contents by digest.
    # Each reference is checked against its actual blob by the saved reader.
    for bank in ['active','candidate','inactive','prepared']:
        raw=bytes.fromhex(value[bank]['bytes']);key=hashlib.sha256(raw).hexdigest()
        encoded=raw.hex()
        assert key not in caller_data['blobs'] or caller_data['blobs'][key]==encoded
        caller_data['blobs'].setdefault(key,encoded)
        value[bank]['bytes']=dict(sha256=key,bytes=len(raw))
    return value

def caller_stable(first,last):
    assert set(first)==set(last)
    for key in first:
        if key not in ['flags','details','tables']:assert first[key]==last[key],('retained MM',key)
    assert first['cs']&3==last['cs']&3==0
    assert first['flags']&0x600==last['flags']&0x600==0
    assert first['cr3']==first['context']['root'] and first['cr0']&(1<<16)
    assert set(first['tables'])==set(last['tables'])
    for at,old in first['tables'].items():
        new=last['tables'][at];assert len(old)==len(new)==512
        assert all((a&~0x60)==(b&~0x60) and not (a&0x60)&~(b&0x60) for a,b in zip(old,new))

def caller_zero(at,size):
    raw=memory(at,size);assert raw==bytes(size)
    return dict(address=at,bytes=size,sha256=hashlib.sha256(raw).hexdigest())

def caller_domains():
    return {name:caller_zero(row['address'],row['bytes'])
        for name,row in caller_retirement_profile['domains'].items()}

class CallerSampleGateway(Checked):
    def __init__(self,returned):
        self.returned=returned
        symbol=caller_retirement_profile['gateway_return' if returned else 'gateway']
        super().__init__('*'+hex(symbol),internal=True)
        self.enabled=False
    def observe(self):
        assert caller_pending is not None and pending is None and reg('cs')&3==0
        row=caller_pending
        if not self.returned:
            assert 'gateway_entry' not in row
            expected=dict(rdi=8,rsi=row['arguments']['r8'],rdx=row['arguments']['r9'],rcx=row['site']['direction'])
            assert {r:reg(r) for r in expected}==expected
            row['gateway_entry']=dict(rip=reg('rip'),sp=reg('rsp'),return_pc=word(reg('rsp')),
                arguments=expected,callee={r:reg(r) for r in callee})
            self.enabled=False
        else:
            assert 'gateway_entry' in row and 'gateway_return' not in row
            entry=row['gateway_entry']
            assert reg('rsp')==entry['sp'] and {r:reg(r) for r in callee}==entry['callee']
            row['gateway_return']=dict(rip=reg('rip'),sp=reg('rsp'),result=reg('rax'),
                callee={r:reg(r) for r in callee},domains=caller_domains(),
                inner_caller=caller_zero(reg('rsp')-4000,4000))
            self.enabled=False
        return False

class CallerSampleReturned(Checked):
    def __init__(self,site):
        self.site=site
        super().__init__('*'+hex(site['returned']),internal=True)
        self.enabled=False
    def observe(self):
        global caller_pending
        self.enabled=False
        assert caller_pending is not None and caller_pending['site']==self.site and pending is None
        row=caller_pending;entry=row['gateway_entry'];returned=row['gateway_return']
        assert reg('rsp')==row['sp'] and {r:reg(r) for r in caller_registers}==row['registers']
        assert reg('rax')==returned['result'] and bool(reg('eflags')&1)==bool(reg('rax'))
        assert all(not point.enabled for point in caller_gateways.values())
        outer=row['sp']-entry['sp'];assert outer in [80,88]
        after=caller_snapshot();caller_stable(row['before'],after)
        row['return']=dict(rip=reg('rip'),sp=reg('rsp'),registers={r:reg(r) for r in caller_registers},
            result=reg('rax'),flags=reg('eflags'),after=after,domains=caller_domains(),
            caller=caller_zero(entry['sp']-4000,4000+outer))
        caller_data['records'].append(row);caller_pending=None
        return False

class CallerSampleEntered(Checked):
    def __init__(self,site):
        self.site=site
        super().__init__('*'+hex(site['call']),internal=True)
    def observe(self):
        global caller_pending
        self.enabled=False
        assert caller_pending is None and pending is None and reg('cs')&3==0
        assert reg('eflags')&0x600==0
        assert memory(self.site['call'],len(bytes.fromhex(self.site['instruction'])))==bytes.fromhex(self.site['instruction'])
        caller_pending=dict(site=self.site,rip=reg('rip'),sp=reg('rsp'),flags=reg('eflags'),
            arguments={r:reg(r) for r in ['r8','r9']},registers={r:reg(r) for r in caller_registers},
            before=caller_snapshot())
        caller_return_points[self.site['offset']].enabled=True
        for point in caller_gateways.values():point.enabled=True
        return False

def complete_caller_retirement():
    assert caller_pending is None and caller_data['records']
    assert all(not point.enabled for point in caller_gateways.values())
    assert all(not point.enabled for point in caller_return_points.values())
    caller_data['complete']=True

class Complete(Complete):
    def observe(self):
        complete_caller_retirement()
        result=super().observe()
        assert result is True and observed['passed']
        print('CALLER_RETIREMENT_PASS',len(caller_data['records']),flush=True)
        return result

def install_caller_retirement():
    caller_gateways.update(entry=CallerSampleGateway(False),returned=CallerSampleGateway(True))
    for site in caller_retirement_profile['calls']:
        caller_return_points[site['offset']]=CallerSampleReturned(site)
        CallerSampleEntered(site)
'''


def augment(original):
    marker="try:\n    Gateway('topal_x86_frame_gateway_v1',True)"
    assert original.count(marker)==1
    return original.replace(marker,PROGRAM+"\ntry:\n    install_caller_retirement()\n    Gateway('topal_x86_frame_gateway_v1',True)")


def boundary_program():
    """Compose with the existing MM namespace and its own terminal checker.

    The enclosing interface observer calls complete_caller_retirement before
    publishing its original successful completion and saving the whole record.
    """
    start='class Complete(Complete):';end='def install_caller_retirement():'
    assert PROGRAM.count(start)==PROGRAM.count(end)==1
    return PROGRAM.split(start)[0]+end+PROGRAM.split(end)[1]
