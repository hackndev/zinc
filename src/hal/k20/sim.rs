// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Gamari <bgamari@gmail.com>
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

//! HAL for Kinetis SIM module.

use super::pin;

/// Enable clock to a PORTx peripheral
#[allow(non_snake_case)]
pub fn enable_PORT(port: pin::Port) {
  use hal::k20::pin::Port::*;
  match port {
    PortA => {reg::SIM.scgc5.set_porta(true);},
    PortB => {reg::SIM.scgc5.set_portb(true);},
    PortC => {reg::SIM.scgc5.set_portc(true);},
    PortD => {reg::SIM.scgc5.set_portd(true);},
    PortE => {reg::SIM.scgc5.set_porte(true);},
  }
}

/// Registers
#[allow(dead_code)]
pub mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(SIM = {
    0x0    => reg32 sopt1 {
      12..15 => ramsize: ro,
      19..18 => osc32ksel,
      29     => usbvstby,
      30     => usbstby,
      31     => usbregen,
    },

    0x4    => reg32 sopt1cfg {
      24     => urwe,
      25     => uvswe,
      26     => usswe,
    },

    0x1004 => reg32 sopt2 {
      4      => rtcclkoutsel,
      5..7   => clkoutsel,
      11     => ptd7pad,
      12     => traceclksel,
      16     => pllfllsel,
      18     => usbsrc,
    },

    0x100c => reg32 sopt4 {
      0      => ftm0flt0,
      1      => ftm0flt1,
      4      => ftm1flt0,
      18..19 => ftm1ch0src,
      24     => ftm0clksel,
      25     => ftm1clksel,
      28     => ftm0trg0src,
    },

    0x1010 => reg32 sopt5 {
      0      => uart0txsrc,
      2..3   => uart0rxsrc,
      4      => uart1txsrc,
      6..7   => uart1rxsrc,
    },

    0x1018 => reg32 sopt7 {
      0..3   => adc0trgsel {
        0x0 => PDB0_EXTRG,
        0x1 => CMP0_OUT,
        0x2 => CMP1_OUT,
        // reserved
        0x4 => PIT_TRG0,
        0x5 => PIT_TRG1,
        0x6 => PIT_TRG2,
        0x7 => PIT_TRG3,
        0x8 => FTM0_TRG,
        0x9 => FTM1_TRG,
        // unused
        // unused
        0x12 => RTC_ALARM,
        0x13 => RTC_SECONDS,
        0x14 => LPT_TRG,
        // unused
      },
      4      => adc0pretrgsel,
      7      => adc0alttrgen,
    },

    0x1034 => reg32 scgc4 {
      1      => ewm,
      2      => cmt,
      6      => i2c0,
      10     => uart0,
      11     => uart1,
      12     => uart2,
      18     => usbotg,
      19     => cmp,
      20     => vreg,
    },

    0x1038 => reg32 scgc5 {
      0      => lptimer,
      5      => tsi,
      9      => porta,
      10     => portb,
      11     => portc,
      12     => portd,
      13     => porte,
    },

    0x103c => reg32 scgc6 {
      0      => ftfl,
      1      => dmamux,
      12     => spi0,
      15     => i2s,
      18     => crc,
      21     => usbdcd,
      22     => pdb,
      23     => pit,
      24     => ftm0,
      25     => ftm1,
      27     => adc0,
      29     => rtc,
    },

    0x1040 => reg32 scgc7 {
      1      => dma,
    },

    0x1044 => reg32 clkdiv1 {
      16..19 => outdiv4,
      24..27 => outdiv2,
      28..31 => outdiv1,
    },

    0x1048 => reg32 clkdiv2 {
      0      => usbfrac,
      1..3   => usbdiv,
    },

    0x104c => reg32 fcfg1 {
      0      => flashdis,
      1      => flashdoze,
      8..11  => depart,
      16..19 => eesize,
      24..27 => pfsize,
      28..31 => nvmsize,
    },

    0x1050 => reg32 fcfg2 {
      16..22 => maxaddr1,
      23     => pflsh,
      24..30 => maxaddr0,
    },

    0x1058 => reg32 uidmh {
      0..31  => uid,
    },

    0x1060 => reg32 uidl {
      0..31  => uid,
    },
  });

  extern {
    #[link_name="k20_iomem_SIM"] pub static SIM: SIM;
  }
}
