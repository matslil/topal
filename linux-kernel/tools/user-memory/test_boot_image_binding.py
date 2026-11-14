"""Retained boot guards: model counterexamples, not injected native states."""
import struct
import unittest
from boot_image_binding import GDT,HANDLERS,cache_index,check_gdt,check_idt,check_idt_pointer,check_pages,check_registers,entry_words


def registers():
    r=[0]*128
    values={0:1,1:0x80000011,2:0x101000,3:32,4:0x500,5:0xfee00900,
            6:0x7040600070406,7:0x508,8:0xc06,9:0x104000,10:47,11:0x105000,
            12:4095,13:0x17ff00,14:16,15:2,16:0x10000,17:0x10000000,
            18:0x10001000,19:0x3000000,20:70248,22:537919488,23:100,
            24:2048,25:512,26:126,27:256,28:8}
    for i,v in values.items(): r[i]=v
    return r


class BootBinding(unittest.TestCase):
    def test_entry_tables_allow_only_applicable_hardware_updates(self):
        values=entry_words()
        values[0]|=32; values[1024]|=96
        check_pages(struct.pack('<2048Q',*values))
        for index,bit in [(0,6),(1,5),(1024,8),(1024,4),(515,12)]:
            changed=values.copy(); changed[index]^=1<<bit
            with self.subTest(index=index,bit=bit),self.assertRaises(ValueError):
                check_pages(struct.pack('<2048Q',*changed))

    def test_gdt_accessed_is_allowed_only_for_nonnull_descriptors(self):
        values=GDT.copy(); values[2]|=1<<40
        check_gdt(struct.pack('<6Q',*values))
        for index,bit in [(0,40),(2,41),(3,47)]:
            changed=values.copy(); changed[index]^=1<<bit
            with self.subTest(index=index,bit=bit),self.assertRaises(ValueError):
                check_gdt(struct.pack('<6Q',*changed))

    def test_actual_handler_gates_and_unused_idt_words_are_exact(self):
        check_idt_pointer(struct.pack('<HQ',4095,0x105000))
        for pointer in [b'',struct.pack('<HQ',4094,0x105000),struct.pack('<HQ',4095,0x106000)]:
            with self.subTest(pointer=pointer),self.assertRaises(ValueError): check_idt_pointer(pointer)
        handlers={name:0x200000+i*4096 for i,name in enumerate(HANDLERS)}
        words=[0]*512
        for vector,index in [(3,1),(14,4),(241,2),(242,3)]:
            target=handlers[HANDLERS[index]]
            words[2*vector]=(target&0xffff)|(16<<16)|((0x8f if vector==3 else 0x8e)<<40)|((target&0xffff0000)<<32)
        check_idt(struct.pack('<512Q',*words),handlers)
        for index,bit in [(4,0),(6,16),(28,40),(29,32)]:
            changed=words.copy(); changed[index]^=1<<bit
            with self.subTest(index=index,bit=bit),self.assertRaises(ValueError):
                check_idt(struct.pack('<512Q',*changed),handlers)

    def test_modes_capabilities_handoff_and_unadvertised_state_are_refused(self):
        good=registers(); check=lambda r:check_registers(r,[1,1,5,100],0x10000,0x3000000)
        check(good)
        for index,value in [(1,0x80000010),(3,160),(4,0),(5,0xfee00800),
                            (7,0x511),(9,0x104008),(10,63),(13,0x100000),
                            (16,0x20000),(19,0x3001000),(20,70248&~65536),(20,70248&~32),
                            (22,537919488&~1048576),(29,1),(96,1)]:
            changed=good.copy(); changed[index]=value
            with self.subTest(index=index),self.assertRaises(ValueError): check(changed)

    def test_large_and_small_pat_positions_do_not_confuse_page_size(self):
        self.assertEqual(cache_index(0x83,True),0)
        self.assertEqual(cache_index(0x83,False),4)
        self.assertEqual(cache_index(0x9b,True),3)
        self.assertEqual(cache_index(0x101b,True),7)


if __name__=='__main__': unittest.main()
