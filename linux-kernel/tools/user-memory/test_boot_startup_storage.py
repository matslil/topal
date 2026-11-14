"""TOPAL-SYSTEMS-BOOT-STARTUP-STORAGE-001: replay controls, not native execution."""
import copy
import hashlib
import struct
import unittest
from boot_startup_storage import inspect,page,SEED,STRIDE,MASK


def fixture():
    base=0x10000
    params=bytearray(4096)
    params[497]=15;params[514:518]=b'HdrS'
    struct.pack_into('<H',params,518,527)
    struct.pack_into('<I',params,532,0x100000)
    struct.pack_into('<I',params,552,0x1f000)
    params[488]=2
    struct.pack_into('<QQI',params,720,0,0x9f000,1)
    struct.pack_into('<QQI',params,740,0x9f000,0x61000,2)
    image=bytes(params)+page()+bytes(128)
    profile=dict(schema='topal.boot-startup-storage.x86_64/1',setup_bytes=8192,setup_sectors=15,
        page_offset=4096,page_bytes=4096,seed=SEED,stride=STRIDE,probe_mask=MASK,
        page_sha256=hashlib.sha256(page()).hexdigest(),owner='boot-adapter-setup-instance',
        lifetime='kernel',reclamation=False,processor_startup=False,participant_arrival=False)
    contract=dict(adapter_revision='topal.boot-adapter.linux-x86-protocol-2.15-q35-startup-storage/1',
        boot_startup_storage=profile,boot_image_sha256=hashlib.sha256(image).hexdigest())
    boot=[0]*128;boot[2]=0x101000;boot[16]=base;boot[19]=0x300000;boot[17:19]=[0x400000,0x500000]
    controls=dict(cr0=0x80000011,cr3=0x101000,cr4=32)
    binding=[base,base+4096,4096,8192,1,1,0,2,0x9f000,0x1f000,boot[2],boot[19],boot[17],boot[18],SEED,STRIDE,MASK,1,1]+[0]*12
    files={}
    banks=[]
    for i in range(2):
        physical=0x2000000+i*0x10000
        words=[0]*2048;words[0]=(physical+4096)|3;words[512]=(physical+8192)|3
        words[1024]=(1<<63)|0x83
        data=struct.pack('<2048Q',*words);path=f'bank-{i}.bin';files[path]=data
        banks.append(dict(base=physical,bytes=len(data),data=path,data_sha256=hashlib.sha256(data).hexdigest()))
    banks.append(dict(base=0x3000000,bytes=4096))
    def row(label,phase,probe=False):
        regions={}
        for name,offset,data in [('parameters',0,bytes(params)),('page',4096,page(probe))]:
            path=f'startup-{label}-{name}.bin';files[path]=data
            regions[name]=dict(start=base+offset,bytes=4096,file=path,sha256=hashlib.sha256(data).hexdigest())
        return dict(boot_params=base,ebda_segment=0x9f00,journal=[phase]+binding if phase else [0]*32,regions=regions,controls=controls.copy())
    phases=[dict(phase=i,boot_registers=boot,banks=copy.deepcopy(banks),
        controls=controls.copy(),reservations=[[0,boot[19]]],firmware_claims=[[0,0x9f000,1],[0x9f000,0x100000,2]],
        startup_storage=row('table-'+str(i),0 if i<3 else 3 if i==3 else 4)) for i in range(7)]
    native=dict(boot_params=base,phases=phases,startup_storage_events=[row('claim-'+str(i),i+1,i==1) for i in range(3)],
        retention_events=[dict(startup_storage=row('event-'+str(i),3),ordinary_owners=[dict(base=0x600000,record=[0,0,0,5])]) for i in range(4)],
        retention_event_count=4,final_bootstrap=dict(startup_storage=row('terminal',4)))
    return native,contract,image,files


class StartupStorage(unittest.TestCase):
    def test_complete_native_shape_replays_loaded_probe_restore_and_retained_owner(self):
        native,contract,image,files=fixture()
        answer=inspect(native,contract,image,files.__getitem__)
        self.assertEqual(answer['checked_snapshots'],15)
        self.assertTrue(answer['final_owner_retained'])
        self.assertFalse(answer['processor_startup'])

    def test_rehashed_wrong_page_at_every_boundary_is_rejected(self):
        for label in ['table-0','claim-0','claim-1','claim-2','table-3','event-0','table-4','terminal']:
            native,contract,image,files=fixture()
            rows=[p['startup_storage'] for p in native['phases']]+native['startup_storage_events']+[
                e['startup_storage'] for e in native['retention_events']]+[native['final_bootstrap']['startup_storage']]
            path=f'startup-{label}-page.bin'
            files[path]=bytes([files[path][0]^1])+files[path][1:]
            for row in rows:
                if row['regions']['page']['file']==path:row['regions']['page']['sha256']=hashlib.sha256(files[path]).hexdigest()
            with self.subTest(label=label),self.assertRaises(ValueError):inspect(native,contract,image,files.__getitem__)

    def test_forged_owner_fields_cannot_be_fixed_by_consistent_later_journals(self):
        for index in [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,31]:
            native,contract,image,files=fixture()
            rows=[p['startup_storage'] for p in native['phases']]+native['startup_storage_events']+[
                e['startup_storage'] for e in native['retention_events']]+[native['final_bootstrap']['startup_storage']]
            for row in rows:
                if row['journal'][0]:row['journal'][index]^=1
            with self.subTest(index=index),self.assertRaises(ValueError):inspect(native,contract,image,files.__getitem__)

    def test_changed_boot_parameters_and_lost_probe_are_not_lifetime_proof(self):
        for case in ('parameters','probe','event','address','typed'):
            native,contract,image,files=fixture()
            if case=='parameters':
                row=native['phases'][4]['startup_storage'];fact=row['regions']['parameters'];path=fact['file']
                files[path]=files[path][:10]+bytes([1])+files[path][11:];fact['sha256']=hashlib.sha256(files[path]).hexdigest()
            elif case=='probe':native['startup_storage_events'].pop(1)
            elif case=='event':native['retention_events'][0].pop('startup_storage')
            elif case=='address':native['retention_events'][0]['startup_storage']['regions']['page']['start']+=4096
            else:contract['boot_startup_storage']['reclamation']=0
            with self.subTest(case=case),self.assertRaises(ValueError):inspect(native,contract,image,files.__getitem__)

    def test_conflicting_ram_claim_and_incompatible_inactive_view_are_rejected(self):
        for case in ('firmware','view'):
            native,contract,image,files=fixture()
            if case=='firmware':
                for path in list(files):
                    if path.endswith('-parameters.bin'):
                        data=bytearray(files[path]);struct.pack_into('<I',data,736,2);files[path]=bytes(data)
                native['phases'][0]['firmware_claims'][0][2]=2
                rows=[p['startup_storage'] for p in native['phases']]+native['startup_storage_events']+[
                    e['startup_storage'] for e in native['retention_events']]+[native['final_bootstrap']['startup_storage']]
                for row in rows:
                    fact=row['regions']['parameters'];fact['sha256']=hashlib.sha256(files[fact['file']]).hexdigest()
            else:
                bank=native['phases'][4]['banks'][0];data=bytearray(files[bank['data']])
                struct.pack_into('<Q',data,1024*8,0x83);files[bank['data']]=bytes(data)
                for row in native['phases']:row['banks'][0]['data_sha256']=hashlib.sha256(data).hexdigest()
            with self.subTest(case=case),self.assertRaises(ValueError):inspect(native,contract,image,files.__getitem__)
