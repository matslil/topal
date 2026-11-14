"""Independent selected public metadata effects over complete saved byte images.

TOPAL-SYSTEMS-USER-EXTENT-001; TOPAL-SYSTEMS-INODE-MODE-STATE-001;
TOPAL-SYSTEMS-NAMESPACE-TRANSACTION-001. Numeric images are observation/model
inputs, never kernel ownership or source mutation authority.
"""
import struct

FS_BYTES=0x9000
TABLE_BYTES=8192


def word(data,at): return struct.unpack_from('<Q',data,at)[0]
def put(data,at,value): struct.pack_into('<Q',data,at,value)


def entry(fs,path):
    assert path.startswith(b'/tmp/') and 0<len(path[5:])<=255
    name=path[5:]
    found=[i for i in range(4) if word(fs,64+i*16) and
        word(fs,72+i*16)==len(name) and fs[1536+i*256:1536+i*256+len(name)]==name]
    assert len(found)<=1
    return found[0] if found else None


def inode(fs,identity):
    slots=[256+i*128 for i in range(4) if word(fs,256+i*128)==1 and word(fs,264+i*128)==identity]
    assert len(slots)==1
    return slots[0]


def description(table,fd):
    assert 0<=fd<16
    index=word(table,64+fd*16)
    assert 1<=index<=16
    at=320+(index-1)*64
    assert word(table,at)==7 and word(table,at+8)==1
    return at


def erase_pages(fs,identity):
    for slot in range(8):
        at=1024+slot*32
        if word(fs,at)!=identity:continue
        assert fs[24+slot]==1
        fs[24+slot]=0
        fs[at:at+32]=bytes(32)
        fs[4096+slot*4096:8192+slot*4096]=bytes(4096)


def transition(before_fs,before_table,number,args,result,paths=(),seconds=None,payload=b''):
    assert len(before_fs)==FS_BYTES and len(before_table)==TABLE_BYTES
    fs,table=bytearray(before_fs),bytearray(before_table)
    if result<0:
        assert result in [-14,-9,-17,-2] and seconds is None
        return bytes(fs),bytes(table)
    if number in [3,5,8]:assert seconds is None
    else:assert seconds is not None and 0<seconds<1<<63
    def stamp_directory():
        put(fs,48,seconds)
        put(fs,56,seconds)
    def publish_name(slot,path,identity):
        assert path.startswith(b'/tmp/') and 0<len(path[5:])<=255
        name=path[5:]
        struct.pack_into('<QQ',fs,64+slot*16,identity,len(name))
        fs[1536+slot*256:1792+slot*256]=name+bytes(256-len(name))
    if number==257:
        path,=paths
        old_entry=entry(fs,path)
        if old_entry is None:
            assert args[2]==0x42
            identity=word(fs,0)
            at=next(256+i*128 for i in range(4) if fs[256+i*128:384+i*128]==bytes(128))
            mode=0o100000|(args[3]&0o7777&~word(table,1664))
            struct.pack_into('<16Q',fs,at,1,identity,0,mode,seconds,seconds,seconds,1,*([0]*8))
            put(fs,0,identity+1)
            slot=next(i for i in range(4) if fs[64+i*16:80+i*16]==bytes(16))
            publish_name(slot,path,identity)
            stamp_directory()
        else:
            assert args[2]==0x202
            identity=word(fs,64+old_entry*16)
            at=inode(fs,identity)
            erase_pages(fs,identity)
            put(fs,at+16,0)
            put(fs,at+40,seconds)
            put(fs,at+48,seconds)
        free=next(i for i in range(16) if word(table,64+i*16)==0)
        index=next(i for i in range(16) if table[320+i*64:384+i*64]==bytes(64))
        assert result==free
        struct.pack_into('<QQ',table,64+free*16,index+1,0)
        status=(args[2]&~(0x80000|0x100|0x40|0x80|0x200))|0x8000
        struct.pack_into('<8Q',table,320+index*64,7,1,status,0,identity,0,0,0)
    elif number==18:
        desc=description(table,args[0])
        identity=word(table,desc+32)
        at=inode(fs,identity)
        assert args[2]==result==len(payload)==4 and args[3]==0
        pages=[i for i in range(8) if word(fs,1024+i*32)==identity and word(fs,1032+i*32)==0]
        assert len(pages)<=1
        slot=pages[0] if pages else next(i for i in range(8) if fs[24+i]==0)
        if not pages:
            assert fs[1024+slot*32:1056+slot*32]==bytes(32)
            assert fs[4096+slot*4096:8192+slot*4096]==bytes(4096)
            fs[24+slot]=1
            struct.pack_into('<4Q',fs,1024+slot*32,identity,0,0,0)
        fs[4096+slot*4096:4100+slot*4096]=payload
        put(fs,at+16,max(word(fs,at+16),4))
        put(fs,at+40,seconds)
        put(fs,at+48,seconds)
    elif number==77:
        assert result==0 and args[1]==2
        desc=description(table,args[0])
        identity=word(table,desc+32)
        at=inode(fs,identity)
        assert word(fs,at+16)==4
        for slot in range(8):
            page=1024+slot*32
            if word(fs,page)!=identity:continue
            assert word(fs,page+8)==0
            fs[4098+slot*4096:8192+slot*4096]=bytes(4094)
        put(fs,at+16,2)
        put(fs,at+40,seconds)
        put(fs,at+48,seconds)
    elif number==91:
        assert result==0
        desc=description(table,args[0])
        at=inode(fs,word(table,desc+32))
        put(fs,at+24,(word(fs,at+24)&0o170000)|(args[1]&0o7777))
        put(fs,at+48,seconds)
    elif number in [82,86]:
        assert result==0
        old,new=paths
        source=entry(fs,old)
        assert source is not None and entry(fs,new) is None
        identity=word(fs,64+source*16)
        at=inode(fs,identity)
        if number==86:
            destination=next(i for i in range(4) if fs[64+i*16:80+i*16]==bytes(16))
            put(fs,at+56,word(fs,at+56)+1)
        else:destination=source
        publish_name(destination,new,identity)
        put(fs,at+48,seconds)
        stamp_directory()
    elif number==87:
        assert result==0
        path,=paths
        slot=entry(fs,path)
        assert slot is not None
        at=inode(fs,word(fs,64+slot*16))
        assert word(fs,at+56)>0
        put(fs,at+56,word(fs,at+56)-1)
        put(fs,at+48,seconds)
        fs[64+slot*16:80+slot*16]=bytes(16)
        fs[1536+slot*256:1792+slot*256]=bytes(256)
        stamp_directory()
    elif number==3:
        assert result==0
        desc=description(table,args[0])
        identity=word(table,desc+32)
        table[64+args[0]*16:80+args[0]*16]=bytes(16)
        table[desc:desc+64]=bytes(64)
        at=inode(fs,identity)
        if not word(fs,at+56) and not any(word(table,320+i*64)==7 and word(table,352+i*64)==identity for i in range(16)):
            erase_pages(fs,identity)
            fs[at:at+128]=bytes(128)
    elif number==8:
        desc=description(table,args[0])
        assert args[1:3]==[0,1] and result==word(table,desc+24)==0
    else:assert number==5 and result==0
    return bytes(fs),bytes(table)
