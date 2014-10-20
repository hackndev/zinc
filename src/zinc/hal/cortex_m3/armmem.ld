/* Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Harris <mail@bharr.is>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
*/

PROVIDE(isr_nmi         = isr_default_fault);
PROVIDE(isr_hardfault   = isr_default_fault);
PROVIDE(isr_mmfault     = isr_default_fault);
PROVIDE(isr_busfault    = isr_default_fault);
PROVIDE(isr_usagefault  = isr_default_fault);
PROVIDE(isr_svcall      = isr_default_fault);
PROVIDE(isr_pendsv      = isr_default_fault);
PROVIDE(isr_systick     = isr_default_fault);

armmem_SYSTICK    = 0xE000E010;

armmem_NVIC       = 0xE000E000;

armmem_SCB_ACTLR  = 0xE000E008;
armmem_SCB        = 0xE000ED00;

armmem_MPU        = 0xE000ED90;
