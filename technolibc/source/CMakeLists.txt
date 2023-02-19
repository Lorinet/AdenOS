set(CMAKE_ASM_FLAGS "${CFLAGS} -x assembler-with-cpp")

set(TARGET_ARCH "x86_64")
set(ARCH_FOLDER "arch/${TARGET_ARCH}")

set(CRT0_SOURCE "${ARCH_FOLDER}/crt0.asm")
set(CRTI_SOURCE "${ARCH_FOLDER}/crt0.asm")
set(CRTN_SOURCE "${ARCH_FOLDER}/crt0.asm")

add_library(crt0 STATIC "${CRT0_SOURCE}")

add_library(crti STATIC "${CRT0_SOURCE}")

add_library(crtn STATIC "${CRT0_SOURCE}")

add_library(technort STATIC "${ARCH_FOLDER}/crt0.asm" "${ARCH_FOLDER}/crti.asm" "${ARCH_FOLDER}/crtn.asm" "${ARCH_FOLDER}/syscall.asm")