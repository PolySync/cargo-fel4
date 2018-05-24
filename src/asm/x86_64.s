/* Copyright (c) 2017 The Robigalia Project Developers
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
    leaq    _stack_top, %rsp
    /* Setup the global "bootinfo" structure. */
    call    __sel4_start_init_boot_info

    /* N.B. rsp MUST be aligned to a 16-byte boundary when main is called.
     * Insert or remove padding here to make that happen.
     */
    pushq $0
    /* Null terminate auxv */
    pushq $0
    pushq $0
    /* Null terminate envp */
    pushq $0
    /* add at least one environment string (why?) */
    leaq environment_string, %rax
    pushq %rax
    /* Null terminate argv */
    pushq $0
    /* Give an argv[0] (why?) */
    leaq prog_name, %rax
    pushq %rax
    /* Give argc */
    pushq $1
    /* No atexit */
    movq $0, %rdx

    /* Now go to the "main" stub that rustc generates */
    call main

    /* if main returns, die a loud and painful death. */
    ud2

    .data
    .align 4

environment_string:
    .asciz "seL4=1"
prog_name:
    .asciz "rootserver"

    .bss
    .align  4096
_stack_bottom:
    .space  65536
_stack_top:
