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

armmem_SYSTICK    = 0xE000E010;

armmem_NVIC_ISER  = 0xE000E100;
armmem_NVIC_ICER  = 0xE000E180;
armmem_NVIC_ISPR  = 0xE000E200;
armmem_NVIC_ICPR  = 0xE000E280;
armmem_NVIC_IABR  = 0xE000E300;
armmem_NVIC_IPR   = 0xE000E400;
armmem_NVIC_STIR  = 0xE000EF00;

armmem_SCB_ACTLR  = 0xE000E008;
armmem_SCB        = 0xE000ED00;

armmem_MPU        = 0xE000ED90;
