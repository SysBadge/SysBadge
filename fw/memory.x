MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

/* ### HAS TO BE KEPT IN SYNC WITH THE EXPORTER */
/* __ssystem_start = ORIGIN(FLASH) + 0x40000 - 0x100; */
__ssystem_start = ORIGIN(FLASH) + LENGTH(FLASH) - 64K;
__ssystem_end = __ssystem_start + 64K;

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;
