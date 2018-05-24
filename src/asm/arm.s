/* Copyright (c) 2015 The Robigalia Project Developers
 * Licensed under the Apache License, Version 2.0
 * <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT
 * license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
 * at your option. All files in the project carrying such
 * notice may not be copied, modified, or distributed except
 * according to those terms.
 */
.global _sel4_start
.global _start
.global _stack_bottom
.text

_start:
_sel4_start:
    ldr sp, =_stack_top
    /* r0, the first arg in the calling convention, is set to the bootinfo
     * pointer on startup. */
    bl __sel4_start_init_boot_info
    /* zero argc, argv */
    mov r0, #0
    mov r1, #0
    /* Now go to the "main" stub that rustc generates */
    bl main

.pool
    .data
    .align 4
    .bss
    .align  8
_stack_bottom:
    .space  65536
_stack_top:
