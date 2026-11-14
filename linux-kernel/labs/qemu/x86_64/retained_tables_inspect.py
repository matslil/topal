"""Read-only live-owner snapshots at ordinary exec/exit syscall entry."""

GDB_HELPERS=r'''
class RetainedEvent(gdb.Breakpoint):
    def __init__(self):
        entry=address('topal_x86_systems_static_image_process_enter')
        size=profile['process_entry']['size']
        assert entry==profile['process_entry']['address'] and 0<size<=1<<20
        body=memory(entry,size)
        # Read the actual closed provider's RIP-relative LSTAR setup. This
        # locates the existing entry without adding/patching any instructions.
        suffix=bytes.fromhex('b9820000c04889c248c1ea200f30')
        sites=[i for i in range(len(body)-20) if body[i:i+3]==bytes.fromhex('488d05') and body[i+7:i+21]==suffix]
        assert len(sites)==1
        at=sites[0]
        target=entry+at+7+int.from_bytes(body[at+3:at+7],'little',signed=True)
        assert entry<=target<entry+size and memory(target,3)==bytes.fromhex('0f01f8')
        observed['process_syscall_entry']=dict(function=entry,setup_offset=at,setup_bytes=body[at:at+21].hex(),target=target)
        super().__init__('*'+hex(target),type=gdb.BP_HARDWARE_BREAKPOINT,internal=True)
    def stop(self):
        try:
            number=reg('rax')
            if number not in [59,60,231]: return False
            events=observed.setdefault('retention_events',[])
            assert len(events)<64 and len(observed['phases'])==4
            initial=observed['phases'][0]
            ctx=words(address('topal_x86_frame_context_v1'),4)
            assert ctx[:2]==initial['context'][:2] and ctx[3]==initial['context'][3]
            assert ctx[2]>initial['context'][2]
            state=words(address('topal_x86_retained_tables_v1'),32)
            assert state==observed['phases'][3]['retained_tables'] and state[0]==1
            event=dict(ordinal=len(events),syscall=number,context=ctx,retained_tables=state,
                controls={n:reg(n) for n in ['cr0','cr3','cr4']},
                registers={n:reg(n) for n in ['rax','rip','rsp','cs']},banks=[],ordinary_owners=[])
            tokens=address('topal_x86_frame_root_tokens_v1')
            for i,owner in enumerate(extent_owners):
                previous=initial['banks'][i]
                token=words(tokens+owner['slot']*24,3)
                assert token==previous['token']
                record=words(address('topal_x86_frame_registry_v1')+token[2]*64,8)
                assert record==previous['record']
                cells=words(address('topal_x86_frame_table_v1')+128+record[2]*24,record[3]*3)
                assert cells==previous['cells']
                data=memory(previous['base'],previous['bytes'])
                expected=(table_encode(profile['layout'],previous['base'],initial['journal'][10] if i==1 else None)
                          if i<2 else bytes(4096))
                assert data==expected
                filename='retention-%d-bank-%d.bin'%(len(events),i)
                with open(os.path.join(os.path.dirname(observation_path),filename),'wb') as f: f.write(data)
                event['banks'].append(dict(token=token,record=record,cells=cells,
                    base=previous['base'],bytes=len(data),data=filename,data_sha256=hashlib.sha256(data).hexdigest()))
            for owner in [profile['active_owners']['bootstrap'],*profile['active_owners']['roles']]:
                token=words(tokens+owner['slot']*24,3)
                assert token[0]==ctx[1] and token[1]>0
                record=words(address('topal_x86_frame_registry_v1')+token[2]*64,8)
                assert record[:2]==token[:2] and record[3:5]==[owner['count'],owner['alignment']]
                assert record[6:]==[owner['pins'],1] and record[5]>0
                cells=words(address('topal_x86_frame_table_v1')+128+record[2]*24,record[3]*3)
                assert cells[0::3]==[token[1]]*record[3] and cells[1::3]==[record[5]]*record[3]
                assert cells[2::3]==list(range(cells[2],cells[2]+record[3]))
                base,end=cells[2]*4096,(cells[2]+record[3])*4096
                assert all(end<=b['base'] or base>=b['base']+b['bytes'] for b in initial['banks'])
                event['ordinary_owners'].append(dict(owner=owner,token=token,record=record,cells=cells,base=base))
            assert event['controls']['cr3']==event['ordinary_owners'][1]['base']
            if profile.get('startup_storage'):event['startup_storage']=capture_startup('event-'+str(len(events)))
            events.append(event)
            print('RETAINED_TABLES_SYSCALL',number,len(events),flush=True)
            return False
        except Exception: fail()
'''
