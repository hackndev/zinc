// Zinc, the bare metal stack for rust.
// Copyright 2014 Dzmitry "kvark" Malyshau <kvarkus@gmail.com>
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

//! BlueNRG low-level SPI communication.
// http://www.st.com/st-web-ui/static/active/en/resource/technical/document/user_manual/DM00114498.pdf

use core::result::Result::{Ok, Err};
use core::result::Result;
use core::slice::SliceExt;

use hal::pin::Gpio;
use hal::spi::Spi;

#[repr(u8)]
enum Control {
  Write = 0x0A,
  Read = 0x0B,
}

/// Spi error codes.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Error {
  /// Device is sleeping.
  Sleeping,
  /// Device is allocating buffers.
  Allocating,
  /// Status is unlnown.
  Unknown(u8),
  /// Given buffer is too large.
  BufferSize(u16),
}

/// BlueNRG driver.
pub struct BlueNrg<G, S> {
  active: G,
  //input: G,
  //output: G,
  spi: S,
}

impl<G: Gpio, S: Spi> BlueNrg<G, S> {
  /// Create a new BlueNRG driver instance.
  pub fn new(active: G, spi: S) -> BlueNrg<G, S> {
    active.set_high();
    BlueNrg {
      active: active,
      spi: spi,
    }
  }

  /// Check device status and return the maximum write/read data sizes.
  pub fn check(&self) -> Result<(u16, u16), Error> {
    self.active.set_low();
    // A return frame is 5 bytes, where the 1st byte is a status,
    // then 2 bytes of the maximum write buffer size,
    // and then 2 bytes for the maximum read buffer.
    let status = self.spi.transfer(Control::Read as u8);
    let w0 = self.spi.transfer(0);
    let w1 = self.spi.transfer(0);
    let r0 = self.spi.transfer(0);
    let r1 = self.spi.transfer(0);
    self.active.set_high();

    match status {
      0x02 if ((w0 | w1 == 0) | (r0 | r1 == 0)) => Err(Error::Allocating),
      0x02 => Ok((
        ((w1 as u16) << 8) | (w0 as u16), // write buffer size
        ((r1 as u16) << 8) | (r0 as u16), // read buffer size
      )),
      0x00 | 0xFF => Err(Error::Sleeping),
      other => Err(Error::Unknown(other)),
    }
  }

  /// Poll the device until it wakes up.
  pub fn wakeup(&self, mut num_tries: u32) -> Result<(u16, u16), Error> {
    loop {
      match self.check() {
        Err(Error::Sleeping)   if num_tries > 0 => {
          num_tries -= 1;
        },
        Err(Error::Allocating) if num_tries > 0 => {
          num_tries -= 1;
        },
        other => return other,
      }
    }
  }

  /// Receive data into the given buffer.
  pub fn receive(&self, buf: &mut [u8]) -> Result<(), Error> {
    self.active.set_low();
    let status = self.spi.transfer(Control::Read as u8);
    self.spi.transfer(0);
    self.spi.transfer(0);
    let r0 = self.spi.transfer(0);
    let r1 = self.spi.transfer(0);
    let size = ((r1 as u16) << 8) | (r0 as u16);
    if status != 0x02 {
      self.active.set_high();
      Err(Error::Unknown(status))
    }else if size < buf.len() as u16 {
      self.active.set_high();
      Err(Error::BufferSize(size))
    }else {
      for b in buf.iter_mut() {
        *b = self.spi.transfer(0);
      }
      self.active.set_high();
      Ok(())
    }
  }

  /// Send data from the given buffer.
  pub fn send(&self, buf: &[u8]) -> Result<(), Error> {
    self.active.set_low();
    let status = self.spi.transfer(Control::Write as u8);
    let w0 = self.spi.transfer(0);
    let w1 = self.spi.transfer(0);
    self.spi.transfer(0);
    self.spi.transfer(0);
    let size = ((w1 as u16) << 8) | (w0 as u16);
    if status != 0x02 {
      self.active.set_high();
      Err(Error::Unknown(status))
    }else if size < buf.len() as u16 {
      self.active.set_high();
      Err(Error::BufferSize(size))
    }else {
      for b in buf.iter() {
        self.spi.transfer(*b);
      }
      self.active.set_high();
      Ok(())
    }
  }
}
