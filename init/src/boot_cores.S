.section ".text.boot"

.global _boot_cores

_boot_cores:
    // Read the CPU id from the processor special register into x1
    mrs     x1, mpidr_el1
    and     x1, x1, #3 // Just interested in the lower 8 bits (the rest have different information)
    cbz     x1, core0
halt: // Non-zero cores fall through to here and are stopped
    wfe
    b       halt
core0: // Core zero branches to here - set up the stack
    ldr     x1, =_boot_cores // Load the location of _boot_cores into x1
    mov     sp, x1 // Let the stack build up from the location of _boot_cores

    // Jump to rust, and include a return pointer (though it should never return)
    bl      reset

    // If we return somehow, halt this core too
    b       halt