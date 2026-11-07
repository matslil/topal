"""Independent complete V6 filesystem checks for existing file-service observers."""
import hashlib
from pathlib import Path


def validate_profile(profile, expected_sha256):
    assert hashlib.sha256(Path(__file__).read_bytes()).hexdigest()==expected_sha256
    assert profile['schema']=='topal.namespace-entry-table.x86_64/1'
    assert tuple(profile[key] for key in ('entry_offset','entry_count','entry_bytes',
        'inode_link_count_offset','inode_initial_link_intent_offset'))==(64,4,16,56,64)


def inspect(store, descriptions, page_map_offset):
    # Entry and inode slots are independent. Never associate a name with an
    # inode by its slot number, nor accept the native validator as our oracle.
    assert len(store)==36864 and page_map_offset==24
    def word(at): return int.from_bytes(store[at:at+8],'little')
    assert word(0)>=7 and not any(store[8:24])
    assert all(value in (0,1) for value in store[24:32])
    assert word(32)==0o41777 and word(40)==0 and word(48)==word(56)
    for lo,hi in ((128,256),(768,1024),(1280,1536),(2560,4096)):
        assert not any(store[lo:hi]),('retained metadata/staging',lo,hi)
    objects={}
    links={}
    for slot in range(4):
        at=256+slot*128
        value=store[at:at+128]
        if word(at)==0:
            assert not any(value)
            continue
        assert word(at)==1
        identity,size=word(at+8),word(at+16)
        assert 7<=identity<word(0) and identity not in objects
        assert size<1<<63 and word(at+24)&~0o7777==0o100000
        assert all(word(at+offset)>0 for offset in (32,40,48))
        count,intent=word(at+56),word(at+64)
        assert count<=4 and intent in (0,1) and not any(value[72:])
        assert count==0 or intent==0
        assert count>0 or descriptions.get(identity,0)>0,'unreferenced inode retained'
        objects[identity]=size
        links[identity]=count
    assert set(descriptions)<=set(objects)
    assert all(count>0 for count in descriptions.values())
    observed={identity:0 for identity in objects}
    names=set()
    for slot in range(4):
        identity,length=word(64+slot*16),word(72+slot*16)
        name=store[1536+slot*256:1536+(slot+1)*256]
        if identity==0:
            assert length==0 and not any(name)
            continue
        assert identity in objects and 1<=length<=255
        component=bytes(name[:length])
        assert 0 not in component and 47 not in component and not any(name[length:])
        assert component not in names
        names.add(component)
        observed[identity]+=1
    assert links==observed,'inode counts disagree with independent entry associations'
    associations=set()
    for slot in range(8):
        at=1024+slot*32
        owner,page=word(at),word(at+8)
        payload=store[4096+slot*4096:4096+(slot+1)*4096]
        assert store[24+slot]==int(owner!=0),'allocation/association mismatch'
        if owner==0:
            assert not any(store[at:at+32]) and not any(payload)
            continue
        assert owner in objects and not any(store[at+16:at+32])
        assert (owner,page) not in associations
        associations.add((owner,page))
        remaining=objects[owner]-page*4096
        assert remaining>0
        if remaining<4096: assert not any(payload[remaining:])
    return objects,associations
