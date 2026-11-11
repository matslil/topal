SECTIONS { . = 0x570000; .text : { *(.text .text.*) *(.rodata .rodata.*) } /DISCARD/ : { *(.eh_frame) *(.comment) *(.note*) } }
