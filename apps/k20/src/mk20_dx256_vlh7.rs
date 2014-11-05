use core::option::{Option, Some, None};

extern {
  fn __STACK_BASE();
}

#[cfg(not(test))]
#[no_mangle]
#[no_stack_check]
pub unsafe extern fn isr_default_fault() {
  asm!("mrs r0, psp
      mrs r1, msp
      ldr r2, [r0, 0x18]
      ldr r3, [r1, 0x18]
      bkpt")
}

#[cfg(test)]
pub extern fn isr_default_fault() { unimplemented!() }

#[allow(non_upper_case_globals)]
const ISRCount: uint = 16;

#[link_section=".isr_vector"]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static ISRVectors: [Option<unsafe extern fn()>, ..ISRCount] = [
  Some(__STACK_BASE),
  Some(::main),               // Reset entry point
  None,                       // NMI
  None,                       // Hard Fault
  None,                       // CM3 Memory Management Fault
  None,                       // CM3 Bus Fault
  None,                       // CM3 Usage Fault
  None,                       // Reserved
  None,                       // Reserved
  None,                       // Reserved
  None,                       // Reserved
  None,                       // SVCall
  None,                       // Reserved for debug
  None,                       // Reserved
  None,                       // PendSV
  None,                       // SysTick
];

#[allow(non_upper_case_globals)]
const VISRCount: uint = 95;

#[link_section=".isr_vector_nvic"]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static NVICVectors: [Option<unsafe extern fn()>, ..VISRCount] = [
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
  None,
];
