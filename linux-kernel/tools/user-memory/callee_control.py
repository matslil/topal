"""Saved x86 transfer/guard checks for conditional callee graph inspection.

TOPAL-SYSTEMS-USER-EXTENT-001. Linked table bytes and decoded guards do not
establish hardware permissions, runtime stability, stack ownership or entries.
"""
import re
import struct

from inventory_paging_controls import PREFIXES,prefix_fragment


class Unresolved(Exception):
    """A valid input lies outside this selected reader's supported patterns."""


def require(condition,message):
    if not condition:raise ValueError(message)


TEXT_PREFIXES={'rep','repne','repe','repnz','repz','lock','bnd','notrack',
    'data16','addr32','cs','ds','es','ss','fs','gs'}
TERMINALS={'ret','retf','iret','iretq','sysret','sysretq','hlt','ud2','int','int3',
    'syscall','sysenter','sysexit'}
UNSUPPORTED_CONTROL={'xbegin','xabort','xend','rsm','vmrun','vmlaunch','vmresume',
    'callf','jmpf','retfq','lretq','sysretl','iretd'}


def mnemonic(row):
    words=row['instruction'].lower().split()
    while words and words[0] in TEXT_PREFIXES:words.pop(0)
    if not words or prefix_fragment(bytes.fromhex(row['bytes'])):raise Unresolved('isolated prefix')
    name=words[0]
    return {'callq':'call','jmpq':'jmp','retq':'ret','movabs':'mov','sal':'shl'}.get(name,name)


def operand(row):
    words=row['instruction'].lower().split()
    while words and words[0] in TEXT_PREFIXES:words.pop(0)
    return ' '.join(words[1:])


def category(row):
    name=mnemonic(row)
    if name in UNSUPPORTED_CONTROL:raise Unresolved('unsupported control family '+name)
    if name in TERMINALS:return name
    if name in ['call','jmp']:return name
    if name.startswith(('j','loop')):return 'conditional'
    return 'ordinary'


def window_rows(text,start,body,lo,hi):
    """Bind a targeted decoder window, including zero/prefix/overlap rows."""
    require(start<=lo<hi<=start+len(body),'decoder recovery extent')
    rows={}
    for line in text.splitlines():
        match=re.match(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(\S.*)$',line)
        if match is None:continue
        address=int(match[1],16);raw=bytes.fromhex(match[2])
        require(lo<=address<hi and 0<len(raw)<=15 and address+len(raw)<=start+len(body),
                'recovery row outside selected window')
        require(body[address-start:address-start+len(raw)]==raw,'recovery row linked bytes differ')
        require(address not in rows,'duplicate recovery start')
        rows[address]=dict(address=address,bytes=raw.hex(),size=len(raw),
            instruction=match[3].split('#',1)[0].strip(),kind=None)
    cursor=lo
    for address,row in sorted(rows.items()):
        require(address<=cursor,'recovery window byte gap');cursor=max(cursor,address+row['size'])
    require(cursor>=hi and lo in rows,'incomplete recovery window')
    return rows


def merge_window(rows,recovered,lo,hi,code_end):
    """Keep stop-truncated prefix rows from replacing complete full listings."""
    # Both decoders may treat a final instruction differently at --stop-address.
    # A complete x86 instruction is at most15bytes. Leave that final margin
    # unapplied unless the window ends at the actual executable-section end.
    applied_end=hi if hi==code_end else hi-15
    require(lo<applied_end<=hi,'decoder recovery applied range')
    rows.update({at:row for at,row in recovered.items() if lo<=at<applied_end})
    return applied_end


def matched(address,llvm,gnu):
    """Require equal bytes/control class, allowing LLVM prefix fragments."""
    if address not in llvm or address not in gnu:raise Unresolved('missing decoder start')
    first,last=llvm[address],gnu[address];raw=bytes.fromhex(last['bytes'])
    require(last['address']==address and last['size']==len(raw),'GNU row position/width')
    parts=[];cursor=address
    while cursor<address+len(raw):
        row=llvm.get(cursor)
        if row is None:raise Unresolved('LLVM fragment gap')
        part=bytes.fromhex(row['bytes']);require(row['size']==len(part)>0,'LLVM fragment width')
        parts.append(row);cursor+=len(part)
        if not prefix_fragment(part):break
    require(cursor==address+len(raw) and b''.join(bytes.fromhex(r['bytes']) for r in parts)==raw,
            ('decoder instruction boundary/bytes differ',address,[r['bytes'] for r in parts],last['bytes']))
    if any('<unknown>' in r['instruction'] or '(bad)' in r['instruction'] for r in [*parts,last]):
        raise Unresolved('unknown decoder operation')
    require(category(parts[-1])==category(last),'decoder control classification differs')
    if category(last) in ['call','jmp','conditional']:
        combined=dict(parts[-1],address=address,size=len(raw),bytes=raw.hex())
        require(transfer(combined)==transfer(last),'decoder transfer target differs')
    return dict(last,mnemonic=mnemonic(last),llvm_mnemonic=mnemonic(parts[-1]),
        llvm_fragments=[r['address'] for r in parts])


def transfer(row):
    """Bind a supported direct displacement or indirect group to both listings."""
    name=mnemonic(row);raw=bytes.fromhex(row['bytes']);index=0
    while index<len(raw) and (raw[index] in PREFIXES or 0x40<=raw[index]<=0x4f):index+=1
    op=raw[index:];require(bool(op),'empty transfer opcode')
    direct=None
    if op[0] in [0xe8,0xe9]:
        require(len(op)==5 and name==('call' if op[0]==0xe8 else 'jmp'),'near transfer encoding')
        direct=struct.unpack_from('<i',op,1)[0]
    elif len(op)>=2 and op[0]==0x0f and 0x80<=op[1]<=0x8f:
        require(len(op)==6 and name.startswith('j') and name!='jmp','conditional transfer encoding')
        direct=struct.unpack_from('<i',op,2)[0]
    elif op[0] in [0xeb,0xe0,0xe1,0xe2,0xe3] or 0x70<=op[0]<=0x7f:
        require(len(op)==2,'short transfer width')
        if op[0]==0xeb:require(name=='jmp','short unconditional transfer encoding')
        elif op[0] in [0xe0,0xe1,0xe2]:require(name.startswith('loop'),'short loop transfer encoding')
        elif op[0]==0xe3:require(name in ['jcxz','jecxz','jrcxz'],'short count-register transfer encoding')
        else:require(name.startswith('j') and name!='jmp','short conditional transfer encoding')
        direct=struct.unpack_from('<b',op,1)[0]
    elif op[0]==0xff:
        require(len(op)>=2,'indirect transfer width')
        group=(op[1]>>3)&7
        if group not in [2,4]:raise Unresolved('far/unsupported indirect group')
        require(name=={2:'call',4:'jmp'}[group],'indirect transfer direction')
        require(re.match(r'^(?:0x)?[0-9a-f]+(?:\s|$)',operand(row)) is None,'indirect relabelled direct')
        return None
    else:raise Unresolved('unsupported transfer opcode')
    expected=row['address']+len(raw)+direct
    match=re.match(r'^(?:0x)?([0-9a-f]+)(?:\s|$)',operand(row))
    require(match is not None and int(match[1],16)==expected,'direct transfer operand/displacement')
    return expected


def bytes_at(rows,address,size):
    result=b'';cursor=address
    while len(result)<size:
        if cursor not in rows:raise Unresolved('guard instruction gap')
        row=rows[cursor];raw=bytes.fromhex(row['bytes'])
        require(row['size']==len(raw)>0,'guard row width')
        result+=raw;cursor+=len(raw)
    if len(result)!=size:raise Unresolved('guard cuts an instruction')
    return result


def previous(rows,address,count):
    """Read an exact contiguous predecessor chain, not nearby sorted rows."""
    ends={}
    for row in rows.values():ends.setdefault(row['address']+row['size'],[]).append(row)
    result=[]
    for _ in range(count):
        candidates=ends.get(address,[])
        if len(candidates)!=1:raise Unresolved('missing/ambiguous contiguous guard predecessor')
        row=candidates[0]
        result.append(row);address=row['address']
    return list(reversed(result))


def guarded_table(rows,address,read):
    """Recognize the six selected compiler-generated table-jump shapes.

    The caller checks all table bytes, code targets, and bypass entries. The
    resulting edge set remains conditional on code/table/stack stability.
    """
    jump=rows[address];raw=bytes.fromhex(jump['bytes'])
    require(transfer(jump) is None and mnemonic(jump)=='jmp','table jump kind')
    register=None;stack_slot=None
    if len(raw)==7 and raw[:3]==bytes.fromhex('ff24fd'):
        register='rdi';base=struct.unpack_from('<i',raw,3)[0]
        compare,guard=previous(rows,address,2)
        code=bytes.fromhex(compare['bytes'])
        if len(code)!=4 or code[:3]!=bytes.fromhex('4883ff'):raise Unresolved('unsupported RDI guard')
        maximum=code[3];begin=compare['address']
    elif len(raw)==8 and raw[:4]==bytes.fromhex('42ff24f5'):
        register='r14';base=struct.unpack_from('<i',raw,4)[0]
        compare,guard=previous(rows,address,2)
        code=bytes.fromhex(compare['bytes'])
        if len(code)!=4 or code[:3]!=bytes.fromhex('4983fe'):raise Unresolved('unsupported R14 guard')
        maximum=code[3];begin=compare['address']
    elif raw==bytes.fromhex('ffe0'):
        saved,store,compare,guard,reload,lookup=previous(rows,address,6)
        a,b,c,d,e=[bytes.fromhex(row['bytes']) for row in [saved,store,compare,reload,lookup]]
        if not (a==bytes.fromhex('89c1') and len(b)==4 and b[:3]==bytes.fromhex('48894d')
                and len(c)==3 and c[:2]==bytes.fromhex('83e8') and len(d)==4
                and d[:3]==bytes.fromhex('488b45') and b[3]==d[3]
                and len(e)==8 and e[:4]==bytes.fromhex('488b04c5')):
            raise Unresolved('unsupported saved-EAX table guard')
        register='zero-extended saved eax';maximum=c[2];begin=saved['address']
        stack_slot=struct.unpack('<b',b[3:])[0];base=struct.unpack_from('<i',e,4)[0]
        require(stack_slot<0 and stack_slot%8==0,'unqualified saved-index stack slot')
    else:raise Unresolved('unsupported table addressing')
    require(0<=maximum<128,'sign-extended guard immediate outside selected positive range')
    guard_raw=bytes.fromhex(guard['bytes'])
    require(mnemonic(guard)==guard.get('llvm_mnemonic','ja')=='ja' and ((len(guard_raw)==2 and guard_raw[0]==0x77)
            or (len(guard_raw)==6 and guard_raw[:2]==b'\x0f\x87')),
            'table guard is not unsigned above')
    default=transfer(guard);require(default is not None and not begin<=default<=address,'table default enters guard')
    payload=read(base,(maximum+1)*8)
    require(len(payload)==(maximum+1)*8,'table payload width')
    targets=list(struct.unpack('<'+'Q'*(maximum+1),payload))
    return dict(address=address,guard_begin=begin,guard=guard['address'],default=default,
        index=register,maximum=maximum,stack_slot=stack_slot,table=base,table_bytes=payload.hex(),targets=targets,
        scope='Decoded positive-range guard and linked table bytes; conditional on unchanged code/table/local stack and the separately checked absence of declared-entry/direct-edge bypasses.')


def no_bypass(table,entries,edges):
    begin,end=table['guard_begin'],table['address']
    require(not any(begin<at<=end for at in entries),'entry bypasses table guard')
    for edge in edges:
        source,target=edge['source'],edge['target']
        if begin<target<=end:
            require(begin<=source<target and edge['kind']=='fallthrough','transfer bypasses table guard')
