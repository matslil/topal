"""Bind the first RTC update-status read in the actual native process function."""
import hashlib
import struct

SYMBOL = 'topal_x86_systems_static_image_process_enter'
PATTERN = bytes.fromhex('b00ae670e4710fb6c0')


def binding(kernel):
    data = kernel.read_bytes()
    assert data[:6] == b'\x7fELF\x02\x01'
    offset = struct.unpack_from('<Q',data,40)[0]
    stride,count = struct.unpack_from('<HH',data,58)
    assert stride == 64 and count > 0
    sections = [struct.unpack_from('<IIQQQQIIQQ',data,offset+i*stride) for i in range(count)]
    found = []
    for table in sections:
        if table[1] != 2:
            continue
        strings = sections[table[6]]
        names = data[strings[4]:strings[4]+strings[5]]
        assert table[9] == 24
        for at in range(table[4],table[4]+table[5],24):
            name,_,_,section,address,size = struct.unpack_from('<IBBHQQ',data,at)
            if names[name:].split(b'\0',1)[0] != SYMBOL.encode():
                continue
            text = sections[section]
            assert text[3] <= address and address+size <= text[3]+text[5]
            start = text[4]+address-text[3]
            code = data[start:start+size]
            locations = [i for i in range(len(code)) if code.startswith(PATTERN,i)]
            assert len(locations) == 3, locations
            found.append(dict(symbol=SYMBOL,address=address,bytes=size,
                sha256=hashlib.sha256(code).hexdigest(),first_update_read_return=address+locations[0]+6,
                instruction_prefix=PATTERN.hex(),matching_status_reads=len(locations)))
    assert len(found) == 1
    return found[0]
