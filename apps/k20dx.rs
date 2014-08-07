use core::mem::transmute;
use zinc::hal::k20::pin::GPIO;
use zinc::hal::k20::sim::reg::SIM;

fn get_sim() -> &'static SIM {
  unsafe { transmute(0x40047000 as *mut ()) }
}

pub struct GPIOB;

impl GPIOB {
  pub fn enable() -> GPIO {
    let sim = get_sim();
    
    sim.set_SCGC5(sim.SCGC5() | (1 << 10));
    
    unsafe {
      GPIO::new(
        transmute(0x400FF040 as *mut ()),
        transmute(0x4004A000 as *mut ())) 
    }
  }
}
