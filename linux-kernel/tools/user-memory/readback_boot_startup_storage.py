"""Independent startup-page replay and altered-saved-record controls; no guest writes."""
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys
from boot_startup_storage import inspect,page


def main():
    root=Path(__file__).resolve().parents[3]
    build=Path(sys.argv[1]).resolve()
    assert build.is_relative_to(root/'linux-kernel/.cache/x86_64')
    native=json.loads((build/'image-tables-native/observations.json').read_text())
    contract=json.loads((build/'kernel-image-boot-binding.json').read_text())
    image=(build/'bzImage').read_bytes()
    outcome=json.loads((build/'image-tables-observations.json').read_text())
    read=lambda path:(build/'image-tables-native'/path).read_bytes()
    assert native['passed'] and outcome['passed'] and outcome['native_execution']
    answer=inspect(native,contract,image,read)
    assert answer==outcome['boot_startup_storage']
    controls=['lost-claim','wrong-probe','changed-live-page','changed-terminal-parameters',
        'stale-generation','coherent-alias','missing-event-view','missing-table-view',
        'wrong-ebda','startup-authority','incompatible-view','reserved-journal','reclaimed-owner']
    rejected=[]
    for case in controls:
        changed=copy.deepcopy(native);selected=copy.deepcopy(contract);overrides={}
        rows=[p['startup_storage'] for p in changed['phases']]+changed['startup_storage_events']+[
            e['startup_storage'] for e in changed['retention_events']]+[changed['final_bootstrap']['startup_storage']]
        def replace(fact,data):
            overrides[fact['file']]=data;fact['sha256']=hashlib.sha256(data).hexdigest()
        if case=='lost-claim':changed['startup_storage_events'].pop(1)
        elif case=='wrong-probe':replace(changed['startup_storage_events'][1]['regions']['page'],page())
        elif case=='changed-live-page':
            fact=changed['retention_events'][0]['startup_storage']['regions']['page'];data=read(fact['file'])
            replace(fact,bytes([data[0]^1])+data[1:])
        elif case=='changed-terminal-parameters':
            fact=changed['final_bootstrap']['startup_storage']['regions']['parameters'];data=read(fact['file'])
            replace(fact,data[:10]+bytes([data[10]^1])+data[11:])
        elif case in ['stale-generation','reserved-journal','reclaimed-owner']:
            index={'stale-generation':5,'reserved-journal':31,'reclaimed-owner':7}[case]
            for row in rows:
                if row['journal'][0]:row['journal'][index]^=1
        elif case=='coherent-alias':
            for row in rows:
                row['regions']['page']['start']=changed['boot_params']
                if row['journal'][0]:row['journal'][2]=changed['boot_params']
        elif case=='missing-event-view':changed['retention_events'][0].pop('startup_storage')
        elif case=='missing-table-view':changed['phases'][3].pop('startup_storage')
        elif case=='wrong-ebda':
            for row in rows:row['ebda_segment']^=1
        elif case=='startup-authority':selected['boot_startup_storage']['processor_startup']=True
        elif case=='incompatible-view':
            bank=changed['phases'][4]['banks'][0];data=bytearray(read(bank['data']))
            word=struct.unpack_from('<Q',data,1024*8)[0];struct.pack_into('<Q',data,1024*8,word&~(1<<63))
            overrides[bank['data']]=bytes(data);bank['data_sha256']=hashlib.sha256(data).hexdigest()
        try:inspect(changed,selected,image,lambda path:overrides[path] if path in overrides else read(path))
        except ValueError:rejected.append(case)
        else:raise AssertionError('altered saved record accepted: '+case)
    answer['rejected_offline_controls']=rejected
    (build/'boot-startup-storage-readback.json').write_text(json.dumps(answer,indent=2)+'\n')
    print(json.dumps(answer))


if __name__=='__main__':main()
