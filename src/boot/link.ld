ENTRY(_start)

MEMORY {
  lp_ram (rwx) : ORIGIN = 0x50000000, LENGTH = 16K
  hp_ram (rwx) : ORIGIN = 0x40800000, LENGTH = 512K
}

SECTIONS {
  .init : {
    *(.init.*)
  } > hp_ram

  .text : {
    KEEP(*(.wake.entry))
    KEEP(*(.text.entry))
    *(.text*)
  } > lp_ram

  .rodata : {
    *(.rodata*)
    *(.srodata*)
    . = ALIGN(4);
    _crc_end = .;
  } > lp_ram

  /* ROM loads initialized data directly into RAM; startup only clears BSS */
  .data : {
    *(.data*)
    *(.sdata*)
  } > lp_ram

  . = ALIGN(16);
  __global_pointer$ = . + 0x800;
  .bss (NOLOAD) : {
    _bss_start = .;
    *(.bss*)
    *(.sbss*)
    *(COMMON)
    . = ALIGN(16);
    _bss_end = .;
  } > lp_ram

  _retained_end = .;
  _stack_top = ORIGIN(lp_ram) + LENGTH(lp_ram);
  ASSERT(_retained_end < _stack_top - 4096, "RAM overflow")

  /DISCARD/ : {
    *(.eh_frame*)
    *(.comment*)
  }
}
