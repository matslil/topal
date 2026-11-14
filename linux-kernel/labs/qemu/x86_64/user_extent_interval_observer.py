"""Watch selected identity-address CPU stores from admission through copying."""
from pathlib import Path

MODEL=(Path(__file__).resolve().parents[3]/'tools/user-memory/owner_interval.py').read_text()
assert MODEL.count('from owner_snapshot import check')==1
EMBEDDED=MODEL.replace('from owner_snapshot import check',"check=owner_snapshot_model['check']")
PROGRAM="owner_interval_model=dict(owner_snapshot_model=owner_snapshot_model)\nexec("+repr(EMBEDDED)+",owner_interval_model)\n"+r'''
interval_active=None
interval_records=[]
interval_snapshots={}
interval_controls=[]
interval_control_pending=None
interval_watches={}
interval_created=[]
interval_watchpoints_retired=False
interval_finish_points={}

def interval_remember(snapshot):
    raw=json.dumps(snapshot,sort_keys=True,separators=(',',':')).encode()
    key=hashlib.sha256(raw).hexdigest()
    assert key not in interval_snapshots or interval_snapshots[key]==snapshot
    interval_snapshots[key]=snapshot
    return key

class IntervalWatch(gdb.Breakpoint):
    def __init__(self,row):
        self.row=row
        self.control=None
        super().__init__('*(unsigned char (*)['+str(row['size'])+'])'+hex(row['start']),
            type=gdb.BP_WATCHPOINT,wp_class=gdb.WP_WRITE,internal=True)
        assert self.type==gdb.BP_HARDWARE_WATCHPOINT
        self.enabled=False
        interval_created.append(dict(number=self.number,type=self.type,expression=self.expression,
            start=row['start'],size=row['size']))

    def stop(self):
        try:
            actual=memory(self.row['start'],self.row['size'])
            event=dict(region=self.row,rip=reg('rip'),cs=reg('cs'),
                thread=list(gdb.selected_thread().ptid),epoch=owner_read('mm_epoch',2),after=actual.hex())
            if self.control is not None:
                assert interval_active is None and pending is not None and pending['number']==12
                assert actual!=bytes.fromhex(self.control['before']) and reg('cs')&3==0
                event.update(before=self.control['before'],kind='ordinary-brk-positive-control')
                interval_controls.append(event)
                self.enabled=False
                self.control=None
                return False
            observed['owner_interval_violation']=event
            raise AssertionError('CPU store changed a protected owner/translation range during admission and copy')
        except Exception:
            traceback.print_exc()
            with open(observation_path,'w') as handle:json.dump(observed,handle,indent=2)
            return True

def interval_point(row):
    key=(row['start'],row['size'])
    if key not in interval_watches:interval_watches[key]=IntervalWatch(row)
    point=interval_watches[key]
    assert point.type==gdb.BP_HARDWARE_WATCHPOINT and not point.enabled and point.control is None
    return point

def interval_syscall_entered(call):
    global interval_control_pending
    if not interval_profile or current is None or call['number']!=12 or call['args'][0]!=0x63a000:return
    assert interval_active is None and interval_control_pending is None
    snapshot=owner_capture()
    rows=[dict(name='control-translation-page',start=snapshot['context']['root']+16384,size=4096),
        dict(name='control-mm-state',start=owner_addresses['mm_state'],size=32),
        dict(name='control-heap-registry',start=owner_addresses['registry']+snapshot['heap_tokens'][2]*64,size=64)]
    interval_control_pending=dict(before=len(interval_controls),points=[])
    for row in rows:
        point=interval_point(row)
        point.control=dict(before=memory(row['start'],row['size']).hex())
        point.enabled=True
        interval_control_pending['points'].append(point)

def interval_syscall_returned(call):
    global interval_control_pending
    if not interval_profile:return
    assert interval_active is None,'copy completion was not observed before source syscall return'
    if interval_control_pending is not None:
        assert call['number']==12 and call['args'][0]==call['result']==0x63a000
        assert len(interval_controls)-interval_control_pending['before']==3
        assert all(not p.enabled and p.control is None for p in interval_control_pending['points'])
        interval_control_pending=None

def interval_begin(before):
    global interval_active
    if not interval_profile:return
    assert interval_active is None and interval_control_pending is None
    call_offset=before['site']-extent_inventory['address']-5
    audit=extent_inventory['stat_output_audit'] if before['number']==5 else extent_inventory['file_input_audit']
    row,=[row for row in audit['cases'] if row['call_offset']==call_offset]
    assert row['name']==('descriptor-stat-output' if before['number']==5 else 'volatile-pwrite-input')
    ranges=owner_interval_model['regions'](before['snapshot'],owner_profile['bindings'],owner_addresses)
    points=[]
    for region in ranges:
        point=interval_point(region)
        point.enabled=True
        points.append(point)
    interval_active=dict(number=before['number'],args=list(before['args']),start=before['start'],
        length=before['length'],direction=before['direction'],before=before['snapshot'],
        ranges=ranges,points=points,call_offset=call_offset,copy_offset=row['copy_offset'],copy_size=row['copy_size'])
    interval_finish_points[row['copy_offset']].enabled=True

class IntervalFinished(Checked):
    def __init__(self,row):
        self.row=row
        super().__init__('*'+hex(extent_inventory['address']+row['copy_offset']+row['copy_size']),internal=True)
        self.enabled=False

    def observe(self):
        global interval_active
        assert interval_active is not None and owner_pending is None
        before=interval_active
        assert before['copy_offset']==self.row['copy_offset']
        assert all(point.enabled and point.type==gdb.BP_HARDWARE_WATCHPOINT for point in before['points'])
        assert reg('rcx')==0
        assert reg('rdi' if before['direction'] else 'rsi')==before['start']+before['length']
        after=owner_capture()
        comparison=owner_interval_model['compare'](before['before'],after,owner_profile['bindings'],
            owner_profile['kernel_spans'],before['start'],before['length'],before['direction'])
        for point in before['points']:point.enabled=False
        self.enabled=False
        interval_records.append(dict(number=before['number'],args=before['args'],start=before['start'],
            length=before['length'],direction=before['direction'],call_offset=before['call_offset'],
            copy_offset=before['copy_offset'],copy_size=before['copy_size'],
            before=interval_remember(before['before']),after=interval_remember(after),
            ranges=before['ranges'],hardware_watchpoint_count=len(before['points']),comparison=comparison,
            identity_address_cpu_store_changes=0,copy_completed=True))
        interval_active=None
        return False

def install_interval_observation():
    if not interval_profile:return
    assert owner_profile is not None and component['mode']=='metadata'
    owner_addresses['mm_state']=address('topal_x86_frame_mm_state_v1')
    for audit,name in [(extent_inventory['stat_output_audit'],'descriptor-stat-output'),
            (extent_inventory['file_input_audit'],'volatile-pwrite-input')]:
        row,=[row for row in audit['cases'] if row['name']==name]
        interval_finish_points[row['copy_offset']]=IntervalFinished(row)

def interval_case_completed():
    global interval_watchpoints_retired
    if not interval_profile or len(observed['executions'])!=3:return
    assert interval_active is interval_control_pending is None
    assert len(interval_records)==52 and len(interval_controls)==12
    assert all(not point.enabled and point.control is None for point in interval_watches.values())
    # The selected intervals are all finished. Release inactive debugger value
    # histories before the enclosing application's independent retirement run.
    for point in interval_watches.values():point.delete()
    interval_watches.clear()
    interval_watchpoints_retired=True
    with open(observation_path+'.interval-progress.json','w') as handle:
        json.dump(dict(passed=False,selected_intervals_complete=True,records=interval_records,
            controls=interval_controls,snapshots=interval_snapshots,created_watchpoints=interval_created,
            inactive_watchpoints_deleted=True,full_guest_retirement_complete=False),handle,indent=2)

def interval_observation_complete():
    if not interval_profile:return
    assert interval_active is interval_control_pending is None
    assert len(interval_records)==52 and len(interval_controls)==12
    assert interval_watchpoints_retired and not interval_watches
    assert all(not point.enabled and point.control is None for point in interval_watches.values())
    observed['owner_interval_observation']=dict(passed=True,records=interval_records,controls=interval_controls,
        snapshots=interval_snapshots,model_sha256=interval_model_sha256,
        addresses=owner_addresses,watchpoints_created=len(interval_created),created_watchpoints=interval_created,
        inactive_watchpoints_deleted_after_cases=interval_watchpoints_retired,
        all_watchpoints_hardware=True,all_watchpoints_disabled=True,
        scope='Selected CPU stores to watched identity virtual addresses from helper entry through actual copy completion, with ordinary brk positive controls. Not physical-alias, DMA, continuous control-register, all asynchronous-entry or general stable-lifetime qualification')
'''
