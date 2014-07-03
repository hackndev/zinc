#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]

extern crate core;
extern crate zinc;

#[no_mangle]
#[no_split_stack]
#[allow(unused_variable)]
#[allow(dead_code)]
pub unsafe fn main() {
    use zinc::hal::timer::Timer;
    zinc::hal::mem_init::init_stack();
    zinc::hal::mem_init::init_data();

    // Pins for STM32F429I-DISCO
    let led1 =
        zinc::hal::stm32f4::pin::PinConf{port: zinc::hal::stm32f4::pin::PortG,
                                         pin: 13u8,
                                         function: zinc::hal::stm32f4::pin::GPIOOut};
    let led2 =
        zinc::hal::stm32f4::pin::PinConf{port: zinc::hal::stm32f4::pin::PortG,
                                         pin: 14u8,
                                         function: zinc::hal::stm32f4::pin::GPIOOut};
    led1.setup();
    led2.setup();

    let timer =
        zinc::hal::stm32f4::timer::Timer::new(zinc::hal::stm32f4::timer::Timer2,
                                              25u32);

  loop {
    led1.set_high();
    led2.set_low();
    timer.wait_ms(300);
    led1.set_low();
    led2.set_high();
    timer.wait_ms(300);
  }
}
