"""Read-only native observations of source-issued exec vector failures."""

PROGRAM = r'''
def exec_protected():
    table = word(context+928)
    assert word(table) == 1
    return dict(table_sha256=hashlib.sha256(memory(table,8192)).hexdigest(),
        registrations=memory(context+152,104).hex(),
        shell=memory(context+416,8).hex(),files=memory(context+736,160).hex(),
        command=memory(word(context+904)+0x9e00,512).hex(),
        snapshot=word(context+904),auxiliary=word(context+920))

def observe_exec_request(call):
    call['protected_before'] = exec_protected()
    assert memory(word(context+904)+0x9800,0x600) == bytes(0x600)
    name,argv,envp = call['args']
    def candidate(address,size):
        # Read only the candidate's actual mapped portion, including code data.
        if component['base'] <= address < component['base']+component['text_bytes']:
            return memory(address,min(size,component['base']+component['text_bytes']-address))
        if 0x5f7000 <= address < 0x63a000:
            return memory(address,min(size,0x63a000-address))
        return b''
    call['pathname_prefix_hex'] = candidate(name,256).hex()
    call['argv_prefix_hex'] = candidate(argv,72).hex()
    call['environment_prefix_hex'] = candidate(envp,136).hex()

def observe_exec_result(call):
    assert call['result'] in [-14,-2,-36,-22,-7],call
    assert exec_protected() == call['protected_before'],call
    assert memory(word(context+904)+0x9800,0x600) == bytes(0x600)
    call['private_staging_zero'] = True
    call['protected_state_unchanged'] = True

def exec_returned(row,after):
    global current
    calls = row['calls']
    assert [c['number'] for c in calls] == [12,12]+[59]*29+[12]
    assert [calls[i]['args'][0] for i in [0,1,-1]] == [0,0x63a000,0x619000]
    assert [calls[i]['result'] for i in [0,1,-1]] == [0x619000,0x63a000,0x619000]
    actions=calls[2:-1]
    expected = [-14]*5+[-2,-36]+[-14]*10+[-22,-7,-7]+[-14]*6+[-7]*3
    assert len(expected)==29
    assert [c['result'] for c in actions] == expected
    assert all(c['private_staging_zero'] and c['protected_state_unchanged'] for c in actions)
    assert [c['args'][0] for c in actions[:5]] == [0,0x63a000,(1<<64)-1,0x800000,0x639fff]
    assert [c['args'][1] for c in actions[7:12]] == [0,0x63a000,(1<<64)-4,0x800000,0x639ffc]
    assert actions[12]['args'][1] == 0x5ffffc
    assert actions[13]['args'][:2] == [0x638ff8,0x637ffc]
    assert component['base'] <= actions[14]['args'][1] < component['base']+component['text_bytes']
    assert actions[16]['args'][1] == 0x639ff0
    assert actions[23]['args'][2] == 0x639ffc
    assert actions[25]['args'][2] == 0x639ff0
    row['after'] = after
    observed['executions'].append(row)
    print('SOURCE_EXEC_CASES_PASS',len(observed['executions']),len(actions),flush=True)
    current = None
    return False
'''
