MEMORY
{
    FLASH : ORIGIN = 0x20000000, LENGTH = 4M
    RAM : ORIGIN = 0x80000000, LENGTH = 16K
}

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

/* Skip first 64k allocated for bootloader */
_stext = 0x20010000;
