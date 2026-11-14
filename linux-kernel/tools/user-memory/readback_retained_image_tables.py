"""Read-only replay of the actual retained-table VM protocol."""
import hashlib,json,sys,struct
from pathlib import Path
from retained_image_tables import inspect,altered_record_controls


def main():
    root=Path(__file__).resolve().parents[3]
    build=Path(sys.argv[1]).resolve()
    assert build.is_relative_to(root/'linux-kernel')
    plan=json.loads((build/'image-tables-plan-binding.json').read_text())
    component=json.loads((build/'image-tables-component.json').read_text())
    result=json.loads((build/'image-tables-observations.json').read_text())
    native=json.loads((build/'image-tables-native/observations.json').read_text())
    contract=json.loads((build/'kernel-image-boot-binding.json').read_text())
    ordinary=json.loads((build/'root-backing-owners.json').read_text())
    digest=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
    assert plan['retained'] and result['passed'] and result['native_execution'] and result['retained']
    assert result['kernel_sha256']==plan['kernel_sha256']==digest(build/'kernel.elf')
    assert component['object_sha256']==digest(build/'image-tables.o') and component['combined_boot_stack_bytes']<=8192
    assert plan['bindings']['root_object_sha256']==ordinary['root_object_sha256']==digest(build/'root.o')
    read=lambda path:(build/'image-tables-native'/path).read_bytes()
    from boot_image_binding import symbols
    kernel=(build/'kernel.elf').read_bytes()
    name='topal_x86_systems_static_image_process_enter'
    actual=symbols(kernel,[name])[name]
    entry=native['process_syscall_entry']
    assert actual['type']==2 and actual['address']==entry['function']
    offset=entry['setup_offset']
    assert 0<=offset and offset+21<=actual['size']
    phoff=struct.unpack_from('<Q',kernel,32)[0]
    size,count=struct.unpack_from('<HH',kernel,54)
    assert size==56
    def instruction_bytes(address,length):
        matches=[]
        for i in range(count):
            p=struct.unpack_from('<II6Q',kernel,phoff+i*size)
            if p[0]==1 and p[1]&1 and p[3]<=address and address+length<=p[3]+p[5]:
                start=p[2]+address-p[3];matches.append(kernel[start:start+length])
        assert len(matches)==1
        return matches[0]
    setup=bytes.fromhex(entry['setup_bytes'])
    assert setup==instruction_bytes(actual['address']+offset,21)
    assert setup[:3]==bytes.fromhex('488d05') and setup[7:]==bytes.fromhex('b9820000c04889c248c1ea200f30')
    assert entry['target']==actual['address']+offset+7+int.from_bytes(setup[3:7],'little',signed=True)
    assert actual['address']<=entry['target']<actual['address']+actual['size']
    assert instruction_bytes(entry['target'],3)==bytes.fromhex('0f01f8')
    answer=inspect(native,plan['layout'],contract,ordinary,read)
    answer['actual_process_syscall_entry']=entry
    answer['rejected_offline_controls']=altered_record_controls(native,plan['layout'],contract,ordinary,read)
    # Only the actual initial comparison window is passed to the old binding
    # readers. Later controls and descriptors are explicitly not asserted equal.
    prefix=dict(native,phases=native['phases'][:4])
    from boot_image_binding import inspect as boot_inspect
    from boot_processors import inspect as processor_inspect
    answer['initial_boot_binding']=boot_inspect(prefix,contract,(build/'kernel.elf').read_bytes(),(build/'bzImage').read_bytes(),read)
    answer['initial_boot_binding']['comparison_boundary']='retention-before-ordinary-work'
    answer['initial_processor_inventory']=processor_inspect(prefix,read)
    (build/'retained-image-tables-readback.json').write_text(json.dumps(answer,indent=2)+'\n')
    print(json.dumps(answer))


if __name__=='__main__': main()
