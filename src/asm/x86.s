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
    leal    _stack_top, %esp
    /* Setup segment selector for IPC buffer access. */
    movw    $((7 << 3) | 3), %ax
    movw    %ax, %fs
    /* Setup the global "bootinfo" structure. */
    pushl   %ebx
    call    __sel4_start_init_boot_info
    /* We drop another word off the stack pointer so that rustc's generated
     * main can scrape the "argc" and "argv" off the stack.
     * TODO: why is this necessary? Caller cleanup of above %ebx? */
    addl    $4, %esp

    /* Null terminate auxv */
    pushl $0
    pushl $0
    /* Null terminate envp */
    pushl $0
    /* add at least one environment string (why?) */
    leal environment_string, %eax
    pushl %eax
    /* Null terminate argv */
    pushl $0
    /* Give an argv[0] */
    leal prog_name, %eax
    pushl %eax
    /* Give argc */
    pushl $1
    /* No atexit */
    movl $0, %edx

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
