"""Bind the saved private boot image's protected entry and IDT to linked ELF.

TOPAL-SYSTEMS-USER-EXTENT-001. No VM runs, no binaries are changed, and no
production provenance or all-entry qualification is inferred.
"""
import argparse
import json
from pathlib import Path

from boot_layout import BASE, GATES, IDTR, bind_loads, gdt, idt, load_segments, number, paging, require, take
from inspect_call_inventory import Elf, digest


class Cursor:
    def __init__(self, raw, address=BASE):
        self.raw, self.start, self.at = raw, address, 0

    def fixed(self, text):
        expected = bytes.fromhex(text)
        require(take(self.raw, self.at, len(expected)) == expected,
                'unsupported transition instruction at ' + hex(self.start + self.at))
        self.at += len(expected)

    def immediate(self, size):
        value = number(self.raw, self.at, size)
        self.at += size
        return value


def transition(payload, entry):
    code = Cursor(take(payload, 0, 4096))
    code.fixed('66b81800 8ed8 8ec0 8ed0 bc')
    stack32 = code.immediate(4)
    code.fixed('0f20e0 83c820 0f22e0 b8')
    root = code.immediate(4)
    code.fixed('0f22d8 b9')
    msr = code.immediate(4)
    code.fixed('0f32 0d')
    efer_or = code.immediate(4)
    code.fixed('0f30 0f20c0 0d')
    cr0_or = code.immediate(4)
    code.fixed('0f22c0 ea')
    long_entry, selector = code.immediate(4), code.immediate(2)
    aligned = (code.at + 15) // 16 * 16
    require(long_entry == BASE + aligned and selector == 16, 'protected-to-long far jump')
    require(not any(take(code.raw, code.at, aligned - code.at)), 'transition alignment bytes')
    code.at = aligned
    code.fixed('66b81800 8ed8 8ec0 8ed0 8ee0 8ee8 fc b0ff e621 e6a1 48bc')
    stack64 = code.immediate(8)
    code.fixed('48b8')
    descriptor = code.immediate(8)
    lidt_at = BASE + code.at
    code.fixed('0f0118 31ed 31db 31ff 48b8')
    target = code.immediate(8)
    code.fixed('ffe0')
    require(stack32 == stack64 == 0x180000 and root == 0x101000,
            'transition stack/root binding')
    require(msr == 0xc0000080 and efer_or == 0x100 and cr0_or == 0x80000000,
            'transition EFER/CR0 masks')
    require(descriptor == IDTR and target == entry, 'transition IDTR/kernel entry binding')
    return dict(protected_entry=BASE, long_entry=long_entry, bytes=code.at,
                sha256=digest(code.raw[:code.at]), kernel_entry=target, initial_stack_top=stack64,
                cr3=root, efer_msr=msr, efer_or=efer_or, cr0_or=cr0_or, cr4_or=0x20,
                lidt_address=lidt_at, idtr_descriptor=descriptor,
                scope='Exact selected saved instruction sequence, including register masks and LIDT operand; actual initial CPU state and execution route remain unobserved here.')


def header(image):
    require(take(image, 0x202, 4) == b'HdrS' and number(image, 0x1fe, 2) == 0xaa55,
            'Linux boot header signatures')
    require(number(image, 0x206, 2) == 0x20f and number(image, 0x1f1, 1) == 4,
            'unsupported private boot protocol/setup size')
    setup = (number(image, 0x1f1, 1) + 1) * 512
    require(number(image, 0x214, 4) == BASE and number(image, 0x258, 8) == BASE
            and number(image, 0x234, 1) == 0 and number(image, 0x211, 1) == 0x81,
            'unsupported private boot placement/flags')
    protected = len(image) - setup
    require(protected > 0 and protected % 16 == 0 and protected == number(image, 0x260, 4)
            and protected == number(image, 0x1f4, 4) * 16, 'boot payload length fields')
    flags = number(image, 0x236, 2)
    require(flags == 1, 'unsupported private extended boot flags')
    require(number(image, 0x200, 1) == 0xeb, 'real-mode setup jump')
    real_entry = 0x202 + int.from_bytes(take(image, 0x201, 1), 'little', signed=True)
    require(0x26c <= real_entry < setup, 'real-mode setup entry extent')
    return dict(protocol=0x20f, setup_bytes=setup, protected_bytes=protected,
                code32_start=BASE, xloadflags=flags, real_mode_entry_file_offset=real_entry)


def inspect(kernel, image):
    elf = Elf(kernel)
    segments = load_segments(elf.data)
    boot = header(image)
    payload = image[boot['setup_bytes']:]
    bind_loads(elf.data, payload, segments)
    targets = {}
    symbols = []
    for name in ['_topal_kernel_entry', *(value[0] for value in GATES.values())]:
        symbol, body = elf.function(name)
        matches = [row for row in segments if row['flags'] & 1
                   and row['address'] <= symbol['value']
                   and symbol['value'] + symbol['size'] <= row['address'] + row['file_bytes']]
        require(len(matches) == 1, 'boot symbol outside unique file-backed executable load')
        row = matches[0]
        offset = row['file_offset'] + symbol['value'] - row['address']
        require(body == take(elf.data, offset, len(body)), 'symbol section/load bytes differ')
        targets[name] = symbol['value']
        symbols.append(dict(symbol=symbol, sha256=digest(body), load_index=row['index']))
    require(number(elf.data, 24, 8) == targets['_topal_kernel_entry'], 'ELF entry/symbol mismatch')
    handoff = transition(payload, targets['_topal_kernel_entry'])
    gates = idt(payload, targets)
    # Linux's separately advertised 64-bit protocol enters at payload + 0x200,
    # not at the internal 32-to-64 transition destination. Never equate them.
    legacy64 = BASE + 0x200
    zero_end = next((i for i in range(0x200, 4096) if payload[i]), 4096)
    roots = [dict(address=handoff['kernel_entry'], source='linked boot handoff', symbol='_topal_kernel_entry')]
    roots += [dict(address=row['target'], source='saved initial IDT', symbol=row['symbol'], vector=row['vector'])
              for row in gates['gates']]
    return dict(passed=True, schema='topal.saved-boot-entries/1', kernel_sha256=digest(elf.data),
                boot_image_sha256=digest(image), header=boot, load_segments=segments, symbols=symbols,
                transition=handoff, gdt=gdt(payload), idt=gates, paging=paging(payload), roots=roots,
                advertised_64bit_entry=dict(address=legacy64, file_offset=boot['setup_bytes'] + 0x200,
                    first_bytes=take(payload, 0x200, 16).hex(), zero_bytes_before_next_nonzero=zero_end - 0x200,
                    qualified=False, blocker='XLF_KERNEL_64 advertises payload+0x200; that route is not the internal long-mode handoff and has no qualified implementation in this inspection.'),
                qualification_complete=False,
                remaining=['Real-mode setup/BIOS/GDT loading and actual loader route/CPU prerequisites.',
                           'Advertised independent 64-bit entry and other boot interfaces.',
                           'Complete handler effects, TSS/stack switching, empty-vector behavior and asynchronous nesting.',
                           'Runtime entry/table/code mutation, AP, DMA, mapping lifetime and continuation ownership.',
                           'Production provenance/reconstruction and full Linux/application/container/VM-host acceptance.'],
                scope='Read-only binding of saved private component bytes. Passing validates the selected structural checks, not all boot interfaces or all-entry/runtime/publication qualification. No guest executes; no previously installed gate or privilege check is changed.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kernel', type=Path)
    parser.add_argument('image', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    report = inspect(args.kernel, args.image.read_bytes())
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ['passed', 'schema', 'kernel_sha256', 'roots',
                                                 'advertised_64bit_entry', 'qualification_complete']}))
