ENTRY(main)
OUTPUT_FORMAT(elf64-x86-64)
INPUT(main)
OUTPUT(main.elf)

MEMORY {
    TEXT(rx): ORIGIN = 0x40000000, LENGTH = 256M
}

SECTIONS {
    .text : ALIGN(CONSTANT(MAXPAGESIZE)) {
        main
    } > TEXT
}
