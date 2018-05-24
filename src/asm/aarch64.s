.global _sel4_start
.global _start
.global _stack_bottom
.text

_start:
_sel4_start:
    ldr x19, =_stack_top
    mov sp, x19
    /* x0, the first arg in the calling convention, is set to the bootinfo
     * pointer on startup. */
    bl __sel4_start_init_boot_info
    /* zero argc, argv */
    mov x0, #0
    mov x1, #0
    /* Now go to the "main" stub that rustc generates */
    bl main

.pool
    .data
    .align 16
    .bss
    .align  16

_stack_bottom:
    .space  65536
_stack_top:
