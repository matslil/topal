"""Parse saved QEMU TCG CPU states immediately following the publisher INVLPG.

These are execution-log observations, not a TLB dump or general CPU exclusion.
"""
import re


def parse(raw,site):
    # With default chaining, QEMU's exec logger also emits link diagnostics,
    # including on a CPU with no states inside the address filter.
    text=''.join(line for line in raw.decode('ascii').splitlines(keepends=True)
        if not re.fullmatch(r'Linking TBs 0x[0-9a-f]+ index [01] -> 0x[0-9a-f]+\n?',line))
    starts=list(re.finditer(r'^Trace ([0-9]+): (0x[0-9a-f]+) \[([0-9a-f]+)/([0-9a-f]+)/[^\]]+\][^\n]*\n',text,re.M))
    assert not starts or not text[:starts[0].start()].strip(),'unexpected trace prefix'
    result=[]
    for i,header in enumerate(starts):
        cpu,tb,_,pc=header.groups()
        pc=int(pc,16)
        assert pc==site,'unexpected CPU trace PC'
        block=text[header.end():starts[i+1].start() if i+1<len(starts) else len(text)]
        def value(name):
            values=re.findall(r'\b'+name+r'\s*=([0-9a-fA-F]+)',block)
            assert len(values)==1,('missing or duplicate CPU state',name)
            return int(values[0],16)
        assert value('RIP')==site and value('CS')&3==0
        stopped=re.findall(r'^Stopped execution of TB chain before (0x[0-9a-f]+) \[([0-9a-f]+)\][^\n]*$',block,re.M)
        assert len(stopped)==sum(line.startswith('Stopped execution') for line in block.splitlines())
        assert len(stopped)<=1 and all(pointer==tb and int(address,16)==site for pointer,address in stopped)
        result.append(dict(cpu=int(cpu),index=value('RCX'),address=value('RAX'),
            cr0=value('CR0'),cr3=value('CR3'),cr4=value('CR4'),efer=value('EFER'),flags=value('RFL'),cancelled=bool(stopped)))
    assert not text or starts,'unparsed nonempty CPU trace'
    return result


def completed(events):
    """Keep explicit cancelled attempts separate and require their exact retry.

    A CPU dump precedes its execution block. QEMU explicitly reports a block
    stopped before execution; it can then log the same post-INVLPG PC again.
    No unmarked duplicate, different retry or terminal cancellation is removed.
    """
    pending=None
    states=[]
    cancelled=[]
    for event in events:
        key={k:v for k,v in event.items() if k!='cancelled'}
        if pending is not None:assert key==pending,'cancelled block lacks matching retry'
        if event['cancelled']:
            pending=key
            cancelled.append(event)
        else:
            states.append(event)
            pending=None
    assert pending is None,'terminal cancelled block has no retry'
    return states,cancelled
