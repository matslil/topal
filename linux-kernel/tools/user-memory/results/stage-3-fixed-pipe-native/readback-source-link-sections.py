"""Compare all section identities and allocated bytes of two source-built ELFs."""
import hashlib
import json
from pathlib import Path
import struct
import sys

digest = lambda data:hashlib.sha256(data).hexdigest()
def sections(path):
    data = path.read_bytes()
    assert data[:7] == b'\x7fELF\x02\x01\x01'
    at = struct.unpack_from('<Q',data,40)[0]
    size,count,names = struct.unpack_from('<HHH',data,58)
    assert size == 64 and count
    headers = [struct.unpack_from('<II4QII2Q',data,at+i*size) for i in range(count)]
    strings = headers[names]
    strings = data[strings[4]:strings[4]+strings[5]]
    result = {}
    for section in headers:
        start = section[0]
        name = strings[start:strings.index(0,start)].decode()
        assert name not in result
        result[name] = dict(type=section[1],flags=section[2],address=section[3],bytes=section[5],
            sha256=digest(data[section[4]:section[4]+section[5]]) if section[1] != 8 else None)
    return dict(bytes=len(data),sha256=digest(data)),result

previous,current,output = map(Path,sys.argv[1:4])
old,old_sections = sections(previous)
new,new_sections = sections(current)
assert old_sections.keys() == new_sections.keys()
allocated = {name:row for name,row in old_sections.items() if row['flags']&2}
assert all(new_sections[name] == row for name,row in allocated.items())
different = [name for name in old_sections if old_sections[name] != new_sections[name]]
assert set(different) <= {'.debug_info','.debug_line','.debug_line_str'}
report = dict(passed=True,previous=old,current=new,allocated_sections_equal=True,
    allocated_sections=allocated,different_nonallocated_sections=different,
    scope='Raw source-user ELF section equality: allocated section layout/bytes agree, differences are confined to these nonallocated debug sections. Not a complete producer or second native execution')
output.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(dict(passed=True,allocated_sections=len(allocated),different_nonallocated_sections=different)))
