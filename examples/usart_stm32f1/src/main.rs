#![feature(plugin, start)]
#![no_std]
#![plugin(macro_zinc)]

extern crate zinc;

#[zinc_main]
fn main() {
  use zinc::drivers::chario::CharIO;
  use zinc::hal;
  use zinc::hal::pin::Gpio;
  use zinc::hal::stm32f1::{init, pin, usart};

   // configure PLL and set it as System Clock source
  let pll_conf = init::PllConf {
    source:        init::PllClockSource::PllSourceHSE(8_000_000),
    mult:          init::PllMult::PllMul9,
    hse_prediv:    init::PllHsePrediv::PllHsePrediv1,
    usb_prescaler: init::PllUsbDiv::PllUsbDiv1p5,
  };
  let sys_clock = init::ClockConfig {
      source:         init::SystemClockSource::SystemClockPLL(pll_conf),
      ahb_prescaler:  init::ClockAhbPrescaler::AhbDivNone,
      apb1_prescaler: init::ClockApbPrescaler::ApbDiv2,
      apb2_prescaler: init::ClockApbPrescaler::ApbDivNone,
      flash_latency:  init::FlashLatency::FlashLatency2,
      mco:            init::McoSource::McoClockPLL,
  };
  sys_clock.setup();

  let _pin_tx = pin::Pin::new(pin::Port::PortA, 2, pin::PinConf::OutPushPullAlt2MHz);

  let led1 = pin::Pin::new(pin::Port::PortC, 13, pin::PinConf::OutPushPull50MHz);

  led1.set_low();

  let uart = usart::Usart::new(usart::UsartPeripheral::Usart2, 38400, usart::WordLen::WordLen8bits,
                               hal::uart::Parity::Disabled, usart::StopBit::StopBit1bit, &sys_clock);
  uart.puts("Hello, world\n");

  led1.set_high();

  loop {}
}
