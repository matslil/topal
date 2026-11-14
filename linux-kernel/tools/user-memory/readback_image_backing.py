"""Replay complete native preparation snapshots without executing a guest."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys


def validate(record, pages, read, layout=None):
    tables = layout is not None
    released = 4 if tables else 3
    assert record['passed'] and record['target_state_injected'] is False
    assert record['protected_context'] is False
    phases = record['phases']
    assert [p['phase'] for p in phases] == list(range(released+1))
    first = phases[0]
    extents = 3 if tables else 2
    assert len(first['banks']) == extents and len(first['journal']) == 32
    ctx = first['context']
    assert ctx[0] == 1 and ctx[2] == first['journal'][9]
    assert 0x109040 <= first['stack'] <= record['initial_stack'] <= 0x180000
    for phase in phases:
        assert phase['context'] == ctx and phase['controls'] == first['controls']
        assert phase['journal'][0] == phase['phase']
        assert phase['journal'][1:] == first['journal'][1:]
        assert phase['journal'][1] == pages and phase['controls']['cr3'] == phase['journal'][4]
        assert len(phase['banks']) == extents
        for i, bank in enumerate(phase['banks']):
            count = pages if i<2 else 1
            original = first['banks'][i]
            token, allocation, cells = bank['token'], bank['record'], bank['cells']
            assert bank['base'] == original['base'] == first['journal'][[2,3,10][i]]
            assert bank['bytes'] == count * 4096
            assert bank['base'] % 4096 == 0 and bank['base'] + bank['bytes'] <= 1 << 30
            assert not bank['base'] <= (phase['controls']['cr3'] & ~4095) < bank['base'] + bank['bytes']
            if phase['phase'] < released:
                assert token == original['token'] and allocation == original['record'] and cells == original['cells']
                assert token[0] == 1 and 0 < token[1] < ctx[2] and token[2] < ctx[3]
                assert allocation[:2] == token[:2] and allocation[3:5] == [count, 1]
                assert allocation[6:] == [1, 1] and allocation[5] > 0
                assert 0 <= allocation[2] and allocation[2] + count <= ctx[3]
                assert len(cells) == count * 3 and cells[0::3] == [token[1]] * count
                assert cells[1::3] == [allocation[5]] * count
                assert cells[2::3] == list(range(bank['base']//4096, bank['base']//4096 + count))
            else:
                assert token == [0, 0, 0] and allocation == [0] * 8
                assert cells[0::3] == [0] * count
                assert cells[1::3] == original['cells'][1::3] and cells[2::3] == original['cells'][2::3]
            assert bank['data'] == f"bank-{phase['phase']}-{i}.bin"
            data = read(bank['data'])
            assert len(data) == bank['bytes'] and hashlib.sha256(data).hexdigest() == bank['data_sha256']
            expected = (b''.join((6510615555426900570 ^ bank['base'] ^ j).to_bytes(8, 'little')
                                 for j in range(count*512)) if phase['phase'] == 0 else bytes(count*4096))
            if tables and phase['phase']==2 and i<2:
                from inactive_image_tables import encode
                expected = encode(layout,bank['base'],first['journal'][10] if i==1 else None)
            assert data == expected
    for i,a in enumerate(first['banks']):
        for b in first['banks'][:i]:
            assert a['base'] + a['bytes'] <= b['base'] or b['base'] + b['bytes'] <= a['base']
            assert a['token'][1] != b['token'][1] and a['token'][2] != b['token'][2]
    for bank in first['banks']:
        lo, hi = bank['base'], bank['base'] + bank['bytes']
        assert any(kind == 1 and start <= lo and hi <= end for start, end, kind in first['firmware_claims'])
        assert all(kind == 1 or hi <= start or lo >= end for start, end, kind in first['firmware_claims'])
        assert all(hi <= start or lo >= end for start, end in first['reservations'])
    final = record['final_bootstrap']
    assert final['controls']['cr3'] == final['base'] and final['base'] % 4096 == 0
    assert final['record'][:2] == final['token'][:2] and final['token'][0] == 1
    assert final['record'][3:5] == [5, 1] and final['record'][6:] == [1, 1]
    data = read('final-bootstrap-bank.bin')
    assert len(data) == 5 * 4096 and hashlib.sha256(data).hexdigest() == final['bank_sha256']


def inspect(build, tables=False):
    role = 'image-tables' if tables else 'image-backing'
    folder = build / (role+'-native')
    native = json.loads((folder / 'observations.json').read_text())
    result = json.loads((build / (role+'-observations.json')).read_text())
    plan = json.loads((build / (role+'-plan-binding.json')).read_text())
    component = json.loads((build / (role+'-component.json')).read_text())
    assert result['passed'] and result['native_execution'] and result['native_execution_attempted']
    assert result['protected_context'] is result['production_qualified'] is False
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    assert result['kernel_sha256'] == plan['kernel_sha256'] == digest(build / 'kernel.elf')
    assert plan['bindings']['root_object_sha256'] == digest(build / 'root.o')
    assert component['object_sha256'] == digest(build / (role+'.o'))
    assert component['acyclic'] and component['combined_boot_stack_bytes'] <= 8192
    pages = result['pages_per_bank']
    if tables:
        from inactive_image_tables import page_count
        assert pages == page_count(plan['layout']) and result['extents']==3
        assert result['low_user_leaf_bytes']==4096 and result['constructed_table_bytes']==pages*4096*2
    else:
        assert pages == 5 + plan['layout']['image_leaf_storage_bytes']//4096
    assert record_size(component) == component['combined_boot_stack_bytes']
    read = lambda name: (folder / name).read_bytes()
    selected_layout = plan['layout'] if tables else None
    validate(native, pages, read, selected_layout)
    controls = []
    for case in range(12):
        changed = deepcopy(native)
        if case == 0: changed['phases'].pop()
        elif case == 1: changed['phases'][1]['context'][2] += 1
        elif case == 2: changed['phases'][1]['controls']['cr3'] += 4096
        elif case == 3: changed['phases'][0]['banks'][0]['record'][3] -= 1
        elif case == 4: changed['phases'][0]['banks'][0]['record'][6] = 0
        elif case == 5: changed['phases'][2]['banks'][0]['cells'][0] = 0
        elif case == 6: changed['phases'][-1]['banks'][0]['token'][0] = 1
        elif case == 7: changed['phases'][-1]['banks'][0]['cells'][1] += 1
        elif case == 8: changed['phases'][0]['reservations'].append([0, 1 << 30])
        elif case == 9: changed['phases'][0]['firmware_claims'] = []
        elif case == 10: changed['phases'][0]['banks'][0]['data_sha256'] = '0' * 64
        elif case == 11: changed['final_bootstrap']['controls']['cr3'] += 4096
        try:
            validate(changed, pages, read, selected_layout)
        except (AssertionError,ValueError):
            controls.append(case)
        else:
            raise AssertionError(f'altered-record control accepted: {case}')
    if tables:
        for case in range(4):
            changed, altered_layout = deepcopy(native), deepcopy(selected_layout)
            if case==0: changed['phases'][0]['banks'][2]['record'][3]=2
            elif case==1: changed['phases'][2]['banks'].pop()
            elif case==2: altered_layout['spans'][0]['role']='code' if altered_layout['spans'][0]['role']!='code' else 'immutable'
            elif case==3: changed['phases'][0]['banks'][2]['base']=changed['phases'][0]['banks'][0]['base']
            try:
                validate(changed,pages,read,altered_layout)
            except (AssertionError,ValueError):
                controls.append(12+case)
            else:
                raise AssertionError(f'altered table/low-leaf record accepted: {case}')
        # Offline bytes are altered in memory and their claimed digest is
        # updated too. Exact independent table validation must still reject
        # pointer, permission and unused-table changes. Guest files stay intact.
        for case,(bank_index,index,bit) in enumerate([(0,0,12),(0,5*512,1),(0,5*512,63),
                                                     (0,4*512,0),(1,1026,12),(1,1027,2)]):
            changed = deepcopy(native)
            bank = changed['phases'][2]['banks'][bank_index]
            data = bytearray(read(bank['data']))
            data[index*8+bit//8] ^= 1 << (bit%8)
            bank['data_sha256'] = hashlib.sha256(data).hexdigest()
            altered_read = lambda name: bytes(data) if name==bank['data'] else read(name)
            try:
                validate(changed,pages,altered_read,selected_layout)
            except (AssertionError,ValueError):
                controls.append(16+case)
            else:
                raise AssertionError(f'altered table bytes accepted after rehash: {case}')
    return dict(passed=True, native_execution=False, kernel_sha256=result['kernel_sha256'],
                phases=len(native['phases']), banks=2, extents=3 if tables else 2,
                pages_per_bank=pages, rejected_controls=controls,
                protected_context=False, production_qualified=False,
                scope='Replay of current native bytes/owners/control registers and final actual bootstrap bank; altered records are offline controls, not injected guest execution')


def record_size(component):
    return sum(n+8 for n in component['frames'].values()) + component['gateway_window_bytes']


if __name__ == '__main__':
    build = Path(sys.argv[1]).resolve()
    assert build.is_relative_to(Path(__file__).resolve().parents[2])
    result = inspect(build)
    (build / 'image-backing-readback.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))
