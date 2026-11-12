"""Reject altered saved paging graphs and coherently rebound analysis reports.

All changes affect offline JSON copies. No ELF, VM, debugger state or guest
memory is modified or executed. TOPAL-SYSTEMS-USER-EXTENT-001.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import tempfile

from audit_mapping_aliases import audit_build
from readback_mapping_aliases import readback


def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':')).encode()
def digest(raw):return hashlib.sha256(raw).hexdigest()


def controls(build,report,output):
    build=Path(build).resolve()
    output=Path(output)
    output.mkdir(parents=True,exist_ok=False)
    original=(build/'source-user-extents/observations.json').read_bytes()
    observation=json.loads(original)
    baseline=audit_build(build)
    assert json.loads(canonical(baseline))==report
    assert readback(build,report)['passed']
    results=[]
    def reject(name,expected,operation):
        try:operation()
        except AssertionError as error:
            message=str(error)
            assert expected in message,(name,expected,message)
            results.append(dict(name=name,rejected=True,expected=expected,assertion=message))
        else:raise AssertionError('altered saved record accepted: '+name)
    with tempfile.TemporaryDirectory(prefix='mapping-alias-json-',dir=output) as temp:
        changed=Path(temp)
        for name in ['kernel.elf','target.ll','user-owners.json']:(changed/name).symlink_to(build/name)
        (changed/'source-user-extents').mkdir()
        key=observation['owner_boundary_observation']['records'][0]['before']
        snapshot=observation['owner_boundary_observation']['snapshots'][key]
        root,low=snapshot['context']['root'],snapshot['context']['low']
        def altered(name,table,index,word,expected):
            value=copy.deepcopy(observation)
            replacement=copy.deepcopy(snapshot)
            replacement['tables'][str(table)][index]=word
            identity=digest(canonical(replacement))
            assert identity!=key
            # Rebind both record namespaces; rejection must not rely on a stale hash.
            for group in ['owner_boundary_observation','owner_interval_observation']:
                saved=value[group]
                if key in saved['snapshots']:
                    del saved['snapshots'][key]
                    saved['snapshots'][identity]=replacement
                for row in saved['records']:
                    for field in ['before','after']:
                        if row[field]==key:row[field]=identity
            (changed/'source-user-extents/observations.json').write_bytes(canonical(value))
            reject(name,expected,lambda:audit_build(changed))
        altered('upper-supervisor-alias',root,256,snapshot['tables'][str(root)][0]&~4,'nonidentity route')
        altered('upper-user-alias',root,256,snapshot['tables'][str(root)][0],'outside the selected live user owners')
        altered('foreign-next-table',root,1,0x17000000|3,'foreign or wrong-level table')
        altered('wrong-level-next-table',root,1,low|3,'foreign or wrong-level table')
        altered('unselected-user-kernel-backing',low,503,0x200000|7,'actual backing owner')
        altered('unselected-user-table-backing',low,503,root|7,'actual backing owner')
        altered('missing-protected-identity',root+8192,4,0,'unmapped protected physical range')
        altered('reserved-large-address',root+8192,32,snapshot['tables'][str(root+8192)][32]|1<<13,'large leaf format')
        altered('unqualified-user-large-leaf',root+8192,30,0x13000000|0x87,'large user mapping')
        altered('extra-supervisor-protected-alias',root+8192,30,0x800000|0x83,'nonidentity route')
        altered('missing-unselected-user-page',low,503,0,'missing live user mapping')
    def report_control(name,change,expected):
        value=copy.deepcopy(report)
        old=value['events'][0]['inventory_sha256']
        result=value['inventories'].pop(old)
        change(result)
        new=digest(canonical(result))
        assert new!=old
        value['inventories'][new]=result
        for event in value['events']:
            if event['inventory_sha256']==old:event['inventory_sha256']=new
        reject(name,expected,lambda:readback(build,value))
    report_control('report-missing-leaf',lambda r:r['inventory']['leaves'].pop(),'flat descriptor inventory disagrees')
    for field,value in [('physical',4096),('user',True),('writable',False),('executable',False)]:
        report_control('report-wrong-'+field,lambda r,f=field,v=value:r['inventory']['leaves'][0].__setitem__(f,v),
            'flat descriptor inventory disagrees')
    report_control('report-wrong-present-entry',lambda r:r['inventory']['entries'][0].__setitem__('word',7),
        'flat descriptor inventory disagrees')
    report_control('report-wrong-table-level',lambda r:r['inventory']['visits'][0].__setitem__('level',3),
        'flat descriptor inventory disagrees')
    report_control('report-wrong-protected-route',lambda r:r['protected_routes'][0]['identity_segments'][0].__setitem__(0,0),
        'protected inverse routes disagree')
    report_control('report-missing-protected-route',lambda r:r['protected_routes'].pop(),'protected inverse routes disagree')
    assert len(results)==20 and all(row['rejected'] for row in results)
    result=dict(passed=True,original_observation_sha256=digest(original),alias_report_sha256=digest(canonical(report)),
        runner_sha256=digest(Path(__file__).read_bytes()),controls=results,
        scope='Eleven coherently rebound offline observation changes and nine coherently rebound report changes; data-only rejection evidence, no changed guest program/state execution or native refusal coverage')
    (output/'controls.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('build',type=Path)
    parser.add_argument('report',type=Path)
    parser.add_argument('output',type=Path)
    args=parser.parse_args()
    result=controls(args.build,json.loads(args.report.read_bytes()),args.output)
    print(json.dumps(dict(passed=result['passed'],rejected=len(result['controls']),scope=result['scope'])))
