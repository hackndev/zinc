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

//! I2C interface

use core::option::{Option, None, Some};
use core::slice::{MutableSlice, ImmutableSlice};
use core::intrinsics::abort;
use core::collections::Collection;
use core::ptr::RawPtr;
use core::ops::Drop;

use core::kinds::marker;
use core::ty::Unsafe;

use os::mutex::{Mutex, MUTEX_INIT, Guard};
use os::cond_var::{CondVar, COND_VAR_INIT};
use hal::cortex_m3::irq::NoInterrupts;
use hal::cortex_m3::nvic;
use util::shared::{Shared};

/// An I2C error
pub enum Error {
  /// Received a NACK from the device
  Nack
}

/// Internal state machine state
enum State {
  Idle,
  Failed(Error),
  RxStart(*mut u8, uint),
  Rx(*mut u8, uint),
  Tx(*const u8, uint)
}

/// An I2C Address
pub struct Address(u8);

impl Address {
  /// Construct an `Address` from an 8-bit address
  pub fn from_8bit(addr: u8) -> Address {
    Address(addr >> 1)
  }

  /// Construct an `Address` from a 7-bit address
  pub fn from_7bit(addr: u8) -> Address {
    Address(addr)
  }
}

/// An I2C peripheral
pub struct I2C {
  lock: Mutex,
  state: Shared<State>,
  irq: CondVar,
  irq_number: uint,
  reg: &'static reg::I2C,
}

/// I2C0 peripheral
pub static mut i2c0: I2C = I2C {
  lock: MUTEX_INIT,
  state: Shared {value: Unsafe{value: Idle}, invariant: marker::InvariantType},
  irq: COND_VAR_INIT,
  irq_number: 11,
  reg: &reg::I2C0,
};

impl I2C {
  /// Start using the peripheral
  pub fn begin<'a>(&'static self) -> Context<'a>{
    let guard = self.lock.lock();
    self.reg.f.set_icr(0x27); // TODO(bgamari): Make speed configurable
    self.reg.c1.set_iicen(true);
    nvic::enable_irq(self.irq_number);
    Context {
      i2c: self,
      guard: guard,
    }
  }
}

/// An I2C context. You must get one of these with `periperal.begin()` to
/// use the peripheral.
pub struct Context<'a> {
  i2c: &'static I2C,
  #[allow(dead_code)]
  guard: Guard<'a>
}

#[unsafe_destructor]
impl<'a> Drop for Context<'a> {
  fn drop(&mut self) {
    self.i2c.reg.c1.set_iicen(false);
    nvic::disable_irq(self.i2c.irq_number);
  }
}

impl<'a> Context<'a> {
  fn finish(&self) -> Option<Error> {
    unsafe {
      let crit = NoInterrupts::new();
      let state = *self.i2c.state.borrow(&crit);
      match state {
        Idle => None,
        Failed(err) => Some(err),
        _ => abort(),
      }
    }
  }

  /// Perform a write to a device
  pub fn write(&self, Address(addr): Address, buffer: &[u8]) -> Option<Error> {
    // ensure STOP symbol has been sent
    while self.i2c.reg.s.get().busy() {}
    self.i2c.reg.c1
      .set_txak(false)
      .set_iicie(true)
      .set_mst(true)
      .set_tx(true);

    self.i2c.reg.d.set_data(addr << 1);
    {
      let crit = NoInterrupts::new();
      let mut state = self.i2c.state.borrow(&crit);
      *state = Tx(buffer.as_ptr(), buffer.len());
    }
    self.i2c.irq.wait();
    self.finish()
  }

  /// Perform a read from a device
  pub fn read(&self, Address(addr): Address, buffer: &mut [u8]) -> Option<Error> {
    // ensure STOP symbol has been sent
    while self.i2c.reg.s.get().busy() {}
    self.i2c.reg.c1
      // send NACK already if we are only to receieve one byte.
      .set_txak(buffer.len() == 1)
      .set_iicie(true)
      .set_mst(true)
      .set_tx(true);

    {
      let crit = NoInterrupts::new();
      let mut state = self.i2c.state.borrow(&crit);
      *state = RxStart(buffer.as_mut_ptr(), buffer.len());
    }
    self.i2c.reg.d.set_data(1 | (addr << 1));
    self.i2c.irq.wait();
    self.finish()
  }
}

fn irq_handler(i2c: &I2C) {
  let crit = NoInterrupts::new(); // TODO(bgamari): Shouldn't need this
  let mut state = i2c.state.borrow(&crit);
  let signal = || {i2c.irq.signal()};
  let status = i2c.reg.s.get();
  i2c.reg.s.set_iicif(true);

  match *state {
    // TODO(bgamari): Check for spurious interrupts
    Idle | Failed(_) => {}, //unsafe { abort() }, // spurious interrupt
    RxStart(_, _) if status.rxak() => {
      // premature nack
      i2c.reg.c1.set_mst(false);      // send stop
      *state = Failed(Nack);
      signal();
    },
    RxStart(d, rem) => {
      i2c.reg.c1.set_tx(false);       // we are now receiving
      let _ = i2c.reg.d.get().data(); // throw away byte
      *state = Rx(d, rem);
    },
    Rx(d, rem) => {
      match rem {
        1 => {
          // last byte has been recieved, finished with rx
          i2c.reg.c1.set_mst(false);  // send stop
          unsafe { *d = i2c.reg.d.get().data(); }
          *state = Idle;
          signal();
        },
        _ => {
          if rem == 2 {
            // second-to-last byte has been recieved
            // send NACK with last byte
            i2c.reg.c1.set_txak(true);
          }
          unsafe {
            *d = i2c.reg.d.get().data();
            *state = Rx(d.offset(1), rem-1);
          }
        }
      }
    },
    Tx(_, _) if status.rxak() => {
      i2c.reg.c1.set_mst(false);
      *state = Failed(Nack);
      signal();
    },
    Tx(d, rem) => {
      unsafe {
        match rem {
          0 => {
            // finished with tx
            i2c.reg.c1.set_mst(false);
            *state = Idle;
            signal();
          },
          _ => {
            i2c.reg.d.set_data(*d);
            *state = Tx(d.offset(1), rem-1)
          }
        }
      }
    },
  }
}

mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(I2C = {
    0x0     => reg8 a1 { //! Slave mode address register
      1..7  => ad,
    }
    0x1     => reg8 f {  //! Frequency divider register
      0..5  => icr,      //= Multiplier factor
      6..7  => mult,     //= Bus clock prescaler
    }
    0x2     => reg8 c1 { //! Control register 1
      0     => dmaen,
      1     => wuen,
      2     => rsta,
      3     => txak,
      4     => tx,
      5     => mst,
      6     => iicie,
      7     => iicen,
    }
    0x3     => reg8 s {  //! Status register
      0     => rxak,
      1     => iicif,
      2     => srw,
      3     => ram,
      4     => arbl,
      5     => busy,
      6     => iaas,
      7     => tcf,
    }
    0x4     => reg8 d {  //! Data register
      0..7  => data,
    }
    0x5     => reg8 c2 { //! Control register 2
      0..2  => ad,       //= High bits of slave address
      3     => rmen,
      4     => sbrc,
      5     => hdrs,
      6     => adext,
      7     => gcaen,
    }
    0x6     => reg8 flt { //! Input glitch filter register
      0..4  => flt,       //= Filter factor
    }
    0x7     => reg8 ra {  //! Range slave address register
      1..7  => rad,
    }
  })

  extern {
    #[link_name="k20_iomem_I2C0"] pub static I2C0: I2C;
  }
}

#[doc(hidden)]
#[allow(dead_code)]
#[allow(non_snake_case)]
#[no_mangle]
pub extern fn isr_i2c_0() {
  unsafe { // This will soon be unnecessary
    irq_handler(&i2c0);
  }
}
