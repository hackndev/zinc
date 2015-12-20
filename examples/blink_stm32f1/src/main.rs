#![feature(plugin)]
#![no_std]
#![plugin(macro_zinc)]

extern crate zinc;

#[zinc_main]
fn main() {
  use core::option::Option;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32f1::{init, pin, timer};
  use zinc::hal::timer::Timer;
  zinc::hal::mem_init::init_stack();
  zinc::hal::mem_init::init_data();

   // configure PLL and set it as System Clock source
  let pll_conf = init::PllConf {
    source: init::PllClockSource::PllSourceHSIDiv2,
    mult: init::PllMult::PllMul12,
    hse_prediv: init::PllHsePrediv::PllHsePrediv1,
    usb_prescaler: init::PllUsbDiv::PllUsbDiv1,
  };
  let sys_clock = init::ClockConfig {
      source:         init::SystemClockSource::SystemClockPLL(pll_conf),
      ahb_prescaler:  init::ClockAhbPrescaler::AhbDivNone,
      apb1_prescaler: init::ClockApbPrescaler::ApbDiv2,
      apb2_prescaler: init::ClockApbPrescaler::ApbDivNone,
      flash_latency:  init::FlashLatency::FlashLatency1,
      mco:            init::McoSource::McoClockNone,
  };
  sys_clock.setup();

  let led1 = pin::Pin::new(pin::Port::PortC, 13, pin::PinConf::OutPushPull50MHz);

  // TODO(kvark): why doesn't "sys_clock.get_apb1_frequency()" work better?
  let timer_clock = sys_clock.source.frequency();
  let timer = timer::Timer::new(timer::TimerPeripheral::Timer2, timer_clock/1000, 0);

  loop {
    led1.set_high();
    timer.wait_ms(1);
    led1.set_low();
    timer.wait_ms(1);
  }
}
