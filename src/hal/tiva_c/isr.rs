// Zinc, the bare metal stack for rust.
// Copyright 2014 Lionel Flandrin <lionel@svkt.org>
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

//! ISR data for tiva_c

use core::option::Option::{self, None};

const ISRCOUNT: usize = 139;

#[link_section=".isr_vector_nvic"]
#[no_mangle]
pub static NVIC_VECTOR: [Option<unsafe extern fn()>; ISRCOUNT] = [
    None,                      // GPIO Port A
    None,                      // GPIO Port B
    None,                      // GPIO Port C
    None,                      // GPIO Port D
    None,                      // GPIO Port E
    None,                      // UART0 Rx and Tx
    None,                      // UART1 Rx and Tx
    None,                      // SSI0 Rx and Tx
    None,                      // I2C0 Master and Slave
    None,                      // PWM Fault
    None,                      // PWM Generator 0
    None,                      // PWM Generator 1
    None,                      // PWM Generator 2
    None,                      // Quadrature Encoder 0
    None,                      // ADC Sequence 0
    None,                      // ADC Sequence 1
    None,                      // ADC Sequence 2
    None,                      // ADC Sequence 3
    None,                      // Watchdog timer
    None,                      // Timer 0 subtimer A
    None,                      // Timer 0 subtimer B
    None,                      // Timer 1 subtimer A
    None,                      // Timer 1 subtimer B
    None,                      // Timer 2 subtimer A
    None,                      // Timer 2 subtimer B
    None,                      // Analog Comparator 0
    None,                      // Analog Comparator 1
    None,                      // Analog Comparator 2
    None,                      // System Control (PLL, OSC, BO)
    None,                      // FLASH Control
    None,                      // GPIO Port F
    None,                      // GPIO Port G
    None,                      // GPIO Port H
    None,                      // UART2 Rx and Tx
    None,                      // SSI1 Rx and Tx
    None,                      // Timer 3 subtimer A
    None,                      // Timer 3 subtimer B
    None,                      // I2C1 Master and Slave
    None,                      // Quadrature Encoder 1
    None,                      // CAN0
    None,                      // CAN1
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Hibernate
    None,                      // USB0
    None,                      // PWM Generator 3
    None,                      // uDMA Software Transfer
    None,                      // uDMA Error
    None,                      // ADC1 Sequence 0
    None,                      // ADC1 Sequence 1
    None,                      // ADC1 Sequence 2
    None,                      // ADC1 Sequence 3
    None,                      // Reserved
    None,                      // Reserved
    None,                      // GPIO Port J
    None,                      // GPIO Port K
    None,                      // GPIO Port L
    None,                      // SSI2 Rx and Tx
    None,                      // SSI3 Rx and Tx
    None,                      // UART3 Rx and Tx
    None,                      // UART4 Rx and Tx
    None,                      // UART5 Rx and Tx
    None,                      // UART6 Rx and Tx
    None,                      // UART7 Rx and Tx
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // I2C2 Master and Slave
    None,                      // I2C3 Master and Slave
    None,                      // Timer 4 subtimer A
    None,                      // Timer 4 subtimer B
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Reserved
    None,                      // Timer 5 subtimer A
    None,                      // Timer 5 subtimer B
    None,                      // Wide Timer 0 subtimer A
    None,                      // Wide Timer 0 subtimer B
    None,                      // Wide Timer 1 subtimer A
    None,                      // Wide Timer 1 subtimer B
    None,                      // Wide Timer 2 subtimer A
    None,                      // Wide Timer 2 subtimer B
    None,                      // Wide Timer 3 subtimer A
    None,                      // Wide Timer 3 subtimer B
    None,                      // Wide Timer 4 subtimer A
    None,                      // Wide Timer 4 subtimer B
    None,                      // Wide Timer 5 subtimer A
    None,                      // Wide Timer 5 subtimer B
    None,                      // FPU
    None,                      // Reserved
    None,                      // Reserved
    None,                      // I2C4 Master and Slave
    None,                      // I2C5 Master and Slave
    None,                      // GPIO Port M
    None,                      // GPIO Port N
    None,                      // Quadrature Encoder 2
    None,                      // Reserved
    None,                      // Reserved
    None,                      // GPIO Port P (Summary or P0)
    None,                      // GPIO Port P1
    None,                      // GPIO Port P2
    None,                      // GPIO Port P3
    None,                      // GPIO Port P4
    None,                      // GPIO Port P5
    None,                      // GPIO Port P6
    None,                      // GPIO Port P7
    None,                      // GPIO Port Q (Summary or Q0)
    None,                      // GPIO Port Q1
    None,                      // GPIO Port Q2
    None,                      // GPIO Port Q3
    None,                      // GPIO Port Q4
    None,                      // GPIO Port Q5
    None,                      // GPIO Port Q6
    None,                      // GPIO Port Q7
    None,                      // GPIO Port R
    None,                      // GPIO Port S
    None,                      // PWM 1 Generator 0
    None,                      // PWM 1 Generator 1
    None,                      // PWM 1 Generator 2
    None,                      // PWM 1 Generator 3
    None,                      // PWM 1 Fault*/
];
