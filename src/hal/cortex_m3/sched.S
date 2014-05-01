/*
  Zinc, the bare metal stack for rust.
 Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
*/


.syntax unified
.cpu cortex-m3
.arch armv7-m
.text

#define THUMB_FUNC(NAME) \
.section .text.sched.##NAME; \
.align 2; \
.global NAME; \
.thumb; \
.thumb_func; \
.type NAME, %function; \
NAME:

/* SysTick handler, for cortex-m3 we save r4-r11 and ask to switch context. */
THUMB_FUNC(isr_systick)
  mrs r0, psp
  stmdb r0!, {r4-r11}
  msr psp, r0

  bl task_scheduler

  mrs r0, psp
  ldmfd r0!, {r4-r11}
  msr psp, r0

  ldr lr, =0xfffffffd  /* return to thread mode using stack at PSP */
  bx lr

/* PendSV handler, same as above. */
THUMB_FUNC(isr_pendsv)
  mrs r0, psp
  stmdb r0!, {r4-r11}
  msr psp, r0

  bl task_scheduler

  mrs r0, psp
  ldmfd r0!, {r4-r11}
  msr psp, r0

  ldr lr, =0xfffffffd
  bx lr

/* SVCall handler, calls function at arg0 with arg1. */
THUMB_FUNC(isr_svcall)
  mrs r2, psp /* r2 points to user stack */

  ldr r1, [r2] /* load func addr from stack */
  ldr r0, [r2, 4] /* load func arg from stack */
  push {lr} /* push lr onto msp */

  blx r1

  pop {pc}

THUMB_FUNC(syscall)
  svc 0
  bx lr
