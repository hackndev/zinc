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
.section .text.isr.##NAME; \
.align 2; \
.global NAME; \
.thumb; \
.thumb_func; \
.weak NAME; \
.type NAME, %function; \
NAME:

THUMB_FUNC(isr_nmi)
  bkpt


THUMB_FUNC(isr_hardfault)
  mrs r0, psp
  mrs r1, msp
  ldr r2, [r0, 0x18]
  ldr r3, [r1, 0x18]
  bkpt


THUMB_FUNC(isr_mmfault)
  bkpt


THUMB_FUNC(isr_busfault)
  bkpt


THUMB_FUNC(isr_usagefault)
  bkpt


THUMB_FUNC(isr_svcall)
  bkpt


THUMB_FUNC(isr_pendsv)
  bkpt


THUMB_FUNC(isr_systick)
  bkpt

THUMB_FUNC(isr_hang)
  b isr_hang
