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

//! Low level system control (PLL, clock gating, ...)
use core::marker::Copy;

use util::support::get_reg_ref;

#[path="../../util/wait_for.rs"]
#[macro_use] mod wait_for;

fn sysctl_get() -> &'static reg::SysCtl {
  get_reg_ref(reg::SYSCTL)
}

pub mod clock {
  //! Clock tree configuration
  use core::option::Option;
  use core::option::Option::{Some, None};
  use core::num::from_u32;

  /// Clock sources available on the system. The values are the RCC/RCC2 OSCSRC
  /// field encoding.
  #[derive(PartialEq, FromPrimitive)]
  pub enum ClockSource {
    /// The Main Oscillator, external crystal/oscillator on OSC pins.
    /// The possible frequencies are listed in MOSCSource.
    MOSC       = 0,
    /// The Precision Internal Oscillator @16MHz
    PIOSC      = 1,
    /// PIOSC divided by 4, resulting in a 4MHz source.
    PIOSC4MHz  = 2,
    /// The Low Frequency Internal Oscillator @30kHz
    LFIOSC     = 3,
    /// The Hibernation Oscillator, external crystal/oscillator on XOSC pins.
    /// Frequency should always be 32.768kHz
    HOSC       = 7,
  }

  /// The chip supports a finite list of crystal frequencies for the MOSC, each
  /// having its own ID used to configure the PLL to output 400MHz.
  #[allow(missing_docs, non_camel_case_types)]
  #[derive(FromPrimitive, Copy)]
  pub enum MOSCFreq {
    X5_0MHz    = 0x09,
    X5_12MHz   = 0x0A,
    X6_0MHz    = 0x0B,
    X6_144MHz  = 0x0C,
    X7_3728MHz = 0x0D,
    X8_0MHz    = 0x0E,
    X8_192MHz  = 0x0F,
    X10_0MHz   = 0x10,
    X12_0MHz   = 0x11,
    X12_288MHz = 0x12,
    X13_56MHz  = 0x13,
    X14_318MHz = 0x14,
    X16_0MHz   = 0x15,
    X16_384MHz = 0x16,
    X18MHz     = 0x17,
    X20MHz     = 0x18,
    X24MHz     = 0x19,
  }

  /// Configure the System Clock by setting the clock source and divisors.
  pub fn sysclk_configure(source:      ClockSource,
                          mosc_source: Option<MOSCFreq>,
                          use_pll:     bool,
                          div:         Option<usize>) {

    let sysctl = super::sysctl_get();

    // Start off by disabling the PLL and dividers, we'll run from the system's
    // clock source directly
    sysctl.rcc
      .set_bypass(true)      // Bypass PLL
      .set_usesysdiv(false); // Don't use the divider

    sysctl.rcc2
      .set_bypass2(false);   // Bypass PLL2

    // If want to switch to the Main Oscillator but it's disabled, we need to
    // enable it and wait for it to lock
    if source == ClockSource::MOSC && sysctl.rcc.mosdis() {

      // Clear any pending MOSC power upinterrupt since we'll have to poll it
      // below
      sysctl.misc.set_moscpupmis(true);

      // Enable MOSC
      sysctl.rcc.set_mosdis(false);

      // Loop till the MOSC has locked
      wait_for!(sysctl.ris.moscpupris());
    }

    sysctl.rcc2.set_oscsrc2(source as u32);

    let (mosdis, xtal) = if source == ClockSource::MOSC {
      (false, mosc_source.unwrap() as u32)
    } else {
      (true, 0)
    };

    let (usesysdiv, sysdiv) = match div {
      Some(d) => (true,  d - 1),
      _       => (false, 0),
    };

    sysctl.rcc
      .set_pwrdn(!use_pll)
      .set_oscsrc(source as u32)
      .set_mosdis(mosdis)
      .set_xtal(xtal)
      .set_usesysdiv(usesysdiv)
      .set_sysdiv(sysdiv as u32);

    sysctl.rcc2
      .set_usercc2(true)
      .set_div400(true)
      .set_pwrdn2(!use_pll)
      .set_sysdiv2((sysdiv >> 1) as u32)
      .set_sysdiv2lsb(sysdiv & 1 != 0);

    if use_pll {
      // Clear any pending PLL lock interrupt
      sysctl.misc.set_plllmis(true);

      // Wait till PLL is locked
      wait_for!(sysctl.pllstat.lock());

      // Remove PLL bypass
      sysctl.rcc.set_bypass(false);
      sysctl.rcc2.set_bypass2(false);
    }
  }

  /// Retrieve the current sysclk frequency
  pub fn sysclk_get() -> usize {
    let sysctl = super::sysctl_get();

    let rcc  = sysctl.rcc.get();
    let rcc2 = sysctl.rcc2.get();

    let use_rcc2 = rcc2.usercc2();

    let div400 = match use_rcc2 {
      true  => rcc2.div400(),
      false => false,
    };

    let oscsrc = match use_rcc2 {
      true  => rcc2.oscsrc2(),
      false => rcc.oscsrc(),
    };

    let clock_source = match from_u32::<ClockSource>(oscsrc) {
      Some(src) => src,
      None      => panic!("Unknown clock source"),
    };

    let input_freq = match clock_source {
      ClockSource::PIOSC     => 16_000_000,
      ClockSource::PIOSC4MHz =>  4_000_000,
      ClockSource::LFIOSC    =>     30_000,
      ClockSource::HOSC      =>     32_768,
      ClockSource::MOSC      => {
        // We're running from the external clock source, we need to figure out
        // what crystal we're using
        let crystal = match from_u32::<MOSCFreq>(rcc.xtal()) {
          Some(c) => c,
          None    => panic!("Unknown crystal"),
        };

        match crystal {
          MOSCFreq::X5_0MHz    =>  5_000_000,
          MOSCFreq::X5_12MHz   =>  5_120_000,
          MOSCFreq::X6_0MHz    =>  6_000_000,
          MOSCFreq::X6_144MHz  =>  6_144_000,
          MOSCFreq::X7_3728MHz =>  7_372_800,
          MOSCFreq::X8_0MHz    =>  8_000_000,
          MOSCFreq::X8_192MHz  =>  8_192_000,
          MOSCFreq::X10_0MHz   => 10_000_000,
          MOSCFreq::X12_0MHz   => 12_000_000,
          MOSCFreq::X12_288MHz => 12_288_000,
          MOSCFreq::X13_56MHz  => 13_560_000,
          MOSCFreq::X14_318MHz => 14_318_000,
          MOSCFreq::X16_0MHz   => 16_000_000,
          MOSCFreq::X16_384MHz => 16_384_000,
          MOSCFreq::X18MHz     => 18_000_000,
          MOSCFreq::X20MHz     => 20_000_000,
          MOSCFreq::X24MHz     => 24_000_000,
        }
      }
    };

    let use_pll = use_rcc2 && rcc2.bypass2() || rcc.bypass();

    // Compute pre-divider frequency
    let div_freq =
      if use_pll {
        // PLL is bypassed, we're running directly from the source clock
        input_freq
      } else {
        // We're running from the PLL output

        let mint  = sysctl.pllfreq0.mint()  as usize;
        let mfrac = sysctl.pllfreq0.mfrac() as usize;
        let n = sysctl.pllfreq1.n()         as usize;
        let q = sysctl.pllfreq1.q()         as usize;

        let mut pllfreq = input_freq / ((n + 1) * (q + 1));
        pllfreq         = (pllfreq * mint) + ((pllfreq * mfrac) >> 10);

        match div400 {
          true  => pllfreq,
          false => pllfreq / 2,
        }
      };

    // TODO(simias): there's atually a lower bound to the sysdiv that
    // can be read in the device capabilities and which is used when
    // sysdiv is too low. For now I'm going to assume that the
    // register value make sense.
    let sysdiv = match rcc.usesysdiv() || use_pll {
      false => 1,
      true   => 1 + match use_rcc2 {
        false => rcc.sysdiv(),
        true => match div400 {
          true  => (rcc2.sysdiv2() << 1) | (rcc2.sysdiv2lsb() as u32),
          false => rcc2.sysdiv2(),
        },
      }
    };

    div_freq / sysdiv as usize
  }
}

impl Copy for clock::ClockSource {}

pub mod periph {
  //! peripheral system control

  use core::iter::range;
  use core::ptr::PtrExt;

  /// Sysctl can reset/clock gate each module, as well as set various sleep and
  /// deep-sleep mode behaviour.
  #[derive(Copy)]
  pub struct PeripheralClock {
    /// Hardware register offset for this peripheral class within a system
    /// control block.
    class: u8,
    /// Bit offset within the class register for this particular peripheral
    id   : u8,
  }

  impl PeripheralClock {

    /// Retrieve the clock gating control register
    fn clock_gating_reg(&self) -> &'static super::reg::SysCtl_rmcgc {
      let base = &super::sysctl_get().rmcgc;

      let p = base as *const super::reg::SysCtl_rmcgc;

      unsafe {
        &*p.offset(self.class as isize)
      }
    }

    /// Enable a peripheral
    #[inline(never)]
    pub fn enable(&self) {
      let cgr = self.clock_gating_reg();

      // Enable peripheral clock
      cgr.set_enabled(self.id as usize, true);

      // The manual says we have to wait for 3 clock cycles before we can access
      // the peripheral. Waiting for 3 NOPs don't seem to be enough on my board,
      // maybe because we also have to take the bus write time into account or
      // the CPU is more clever than I think. Anyway, looping 5 times seems to
      // work
      for _ in range(0usize, 10) {
        unsafe {
          asm!("nop" :::: "volatile");
        }
      }
    }

    /// Check if the peripheral is enabled. If not, enable it.
    #[inline(never)]
    pub fn ensure_enabled(&self) {
      let cgr = self.clock_gating_reg();

      if !cgr.enabled(self.id as usize) {
        self.enable();
      }
    }
  }

  pub mod gpio {
    //! GPIO system control peripherals. Split into ports of 8 GPIO each.

    const CLASS: u8 = 0x8 / 4;

    pub const PORT_A: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 0 };
    pub const PORT_B: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 1 };
    pub const PORT_C: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 2 };
    pub const PORT_D: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 3 };
    pub const PORT_E: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 4 };
    pub const PORT_F: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 5 };
  }

  pub mod timer {
    //! Timer system control peripherals. Each timer has two independent
    //! counters (A and B).

    const TIMER_CLASS:   u8 = 0x4 / 4;
    const TIMER_W_CLASS: u8 = 0x5c / 4;

    pub const TIMER_0: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_CLASS, id: 0 };
    pub const TIMER_1: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_CLASS, id: 1 };
    pub const TIMER_2: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_CLASS, id: 2 };
    pub const TIMER_3: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_CLASS, id: 3 };
    pub const TIMER_4: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_CLASS, id: 4 };
    pub const TIMER_5: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_CLASS, id: 5 };

    pub const TIMER_W_0: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_W_CLASS, id: 0 };
    pub const TIMER_W_1: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_W_CLASS, id: 1 };
    pub const TIMER_W_2: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_W_CLASS, id: 2 };
    pub const TIMER_W_3: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_W_CLASS, id: 3 };
    pub const TIMER_W_4: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_W_CLASS, id: 4 };
    pub const TIMER_W_5: super::PeripheralClock =
      super::PeripheralClock { class: TIMER_W_CLASS, id: 5 };
  }

  pub mod uart {
    //! UART peripherals instances
    const CLASS: u8 = 0x18 / 4;

    pub const UART_0: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 0 };
    pub const UART_1: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 1 };
    pub const UART_2: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 2 };
    pub const UART_3: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 3 };
    pub const UART_4: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 4 };
    pub const UART_5: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 5 };
    pub const UART_6: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 6 };
    pub const UART_7: super::PeripheralClock =
      super::PeripheralClock { class: CLASS, id: 7 };
  }
}

pub mod reg {
  //! Sysctl registers definition
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(SysCtl = {
    0x050 => reg32 ris {
      1  => bor1ris:    ro,  //= VDD under BOR1 raw interrupt status
      3  => mofris:     ro,  //= Main oscillator failure raw interrupt status
      6  => plllris:    ro,  //= PLL lock raw interrupt status
      7  => usbplllris: ro,  //= USB PLL lock raw interrupt status
      8  => moscpupris: ro,  //= MOSC Power Up raw interrupt status
      10 => vddaris:    ro,  //= VDDA Power OK event raw interrupt status
      11 => bor0ris:    ro,  //= VDD under BOR0 raw interrupt status
    }
    0x060 => reg32 rcc {
      0      => mosdis,      //= Main oscillator disable
      4..5   => oscsrc,      //= Oscillator source
      6..10  => xtal,        //= Crystal value
      11     => bypass,      //= PLL bypass
      13     => pwrdn,       //= PLL power down
      17..19 => pwmdiv,      //= PWM unit clock divisor
      20     => usepwmdiv,   //= Enable PWM clock divisor
      22     => usesysdiv,   //= Enable system clock divider
      23..26 => sysdiv,      //= System clock divisor
      27     => acg,         //= Auto clock gating
    }
    0x058 => reg32 misc {
      1  => bor1mis,         //= VDD under BOR1 masked interrupt status
      3  => mofmis,          //= Main oscillator failure masked interrupt status
      6  => plllmis,         //= PLL lock masked interrupt status
      7  => usbplllmis,      //= USB PLL lock masked interrupt status
      8  => moscpupmis,      //= MOSC Power Up masked interrupt status
      10 => vddamis,         //= VDDA Power OK event masked interrupt status
      11 => bor0mis,         //= VDD under BOR0 masked interrupt status
    }
    0x070 => reg32 rcc2 {
      4..6   => oscsrc2,     //= Oscillator source 2
      11     => bypass2,     //= PLL bypass 2
      13     => pwrdn2,      //= PLL power down 2
      14     => usbpwrdn,    //= Power down USB PLL
      22     => sysdiv2lsb,  //= Additional LSB for sysdiv2
      23..28 => sysdiv2,     //= System clock divisor 2
      30     => div400,      //= Divide PLL as 400MHz vs. 200Mhz
      31     => usercc2,     //= Use RCC2
    }
    0x160 => reg32 pllfreq0 {
      0..9   => mint,        //= PLL M integer value
      10..19 => mfrac,       //= PLL M fractional value (*1024)
    }
    0x164 => reg32 pllfreq1 {
      0..4   => n,           //= PLL N value
      8..12  => q,           //= PLL Q value
    }
    0x168 => reg32 pllstat {
      0 => lock,             //= PLL is powered and locked
    }
    0x600 => reg32 rmcgc {
      0..31  => enabled[32], //= Module clock gating control (0 means gated)
    }
  });

  pub const SYSCTL: *const SysCtl = 0x400FE000 as *const SysCtl;
}
