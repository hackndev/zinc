// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
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

use std::intrinsics::abort;
use std::option::{Some, None};
use std::slice::{ImmutableVector};
use std::container::Container;
use std::iter::Iterator;

use hal::gpio::GPIOConf;
use hal::spi::SPI;
use hal::timer::Timer;

pub struct Mrf24j40<'a, S, T> {
  spi: &'a S,
  timer: &'a T,
  reset: GPIOConf,
  cs: GPIOConf,
  interrupt: GPIOConf,

  channel: u8,
}

impl<'a, S: SPI, T: Timer> Mrf24j40<'a, S, T> {
  pub fn new(spi: &'a S, timer: &'a T, reset: GPIOConf, cs: GPIOConf,
      interrupt: GPIOConf, initial_channel: u8) -> Mrf24j40<'a, S, T> {
    let radio = Mrf24j40 {
      spi: spi,
      timer: timer,
      reset: *reset.setup(),
      cs: *cs.setup(),
      interrupt: *interrupt.setup(),
      channel: initial_channel,
    };

    radio.hard_reset();
    radio.reinitialize();

    radio
  }

  fn hard_reset(&self) {
    self.reset.set_low();
    self.timer.wait_ms(20);
    self.reset.set_high();
    self.timer.wait_ms(20);
  }

  pub fn set_pan(&self, pan: u16) {
    self.set_PANIDL(pan as u8);
    self.set_PANIDH((pan >> 8) as u8)
  }

  pub fn get_pan(&self) -> u16 {
    self.PANIDL() as u16 | (self.PANIDH() as u16 << 8)
  }

  pub fn set_short_address(&self, adr: u16) {
    self.set_SADRL(adr as u8);
    self.set_SADRH((adr >> 8) as u8)
  }

  pub fn get_short_address(&self) -> u16 {
    self.SADRL() as u16 | (self.SADRH() as u16 << 8)
  }

  pub fn send_to_short_address(&self, dst: u16, data: &[u8]) {
    let mut header: [u8, ..9] = [0, ..9];

    // Header
    // Frame Control
    header[0] = 0b01100001;
    //                 ^^^ - frame type: data
    //                ^    - security disabled (no encryption)
    //               ^     - no frame pending
    //              ^      - acknowledgment required
    //             ^       - pan id is compressed (only destination
    //                       pan present, src_pan == dst_pan)
    //            ^        - reserved

    header[1] = 0b10001000;
    //                  ^^ - reserved
    //                ^^   - destination address field contains short address
    //              ^^     - frame version 0 (802.15.4-2003 compatible)
    //            ^^       - source address field contains short address

    header[2] = 1;                 // sequence number 1
    header[3] = self.PANIDL();     // dest pan low
    header[4] = self.PANIDH();     // dest pan high
    header[5] = dst as u8;         // dest address low
    header[6] = (dst >> 8) as u8;  // dest address high
    header[7] = self.SADRL();      // src address low
    header[8] = self.SADRH();      // src address high

    self.write_packet(header, data);
  }

  fn write_packet(&self, header: &[u8], payload: &[u8]) {
    let mut ofs: u16 = 0;

    // Header Length
    self.reg_write_long(ofs, header.len() as u8);
    ofs += 1;

    // Frame Length (header + payload)
    self.reg_write_long(ofs, (header.len() + payload.len()) as u8);
    ofs += 1;

    for &b in header.iter() {
      self.reg_write_long(ofs, b);
      ofs += 1;
    }

    for &b in payload.iter() {
      self.reg_write_long(ofs, b);
      ofs += 1;
    }

    self.set_TXNCON(0b00101);  // transmit with acknowledgment required
  }

  /// Perform the initialization by the spec
  fn reinitialize(&self) {
    self.set_PACON2(0x98);
    self.set_TXSTBL(0x95);
    self.set_RFCON0(0x03);
    self.set_RFCON1(0x01);
    self.set_RFCON2(0x80);
    self.set_RFCON6(0x90);
    self.set_RFCON7(0x80);
    self.set_RFCON8(0x10);
    self.set_SLPCON1(0x21);

    self.set_BBREG2(0x80);
    self.set_CCAEDTH(0x60);
    self.set_BBREG6(0x40);

    self.enable_interrupts();
    self.set_channel(self.channel);

  }

  pub fn set_channel(&self, channel: u8) {
    if channel > 26 || channel < 11 {
      unsafe { abort() }
    }
    // XXX: this also sets RFOPT to 3 (same as init value) to save on
    // read-modify-write.
    self.set_RFCON0((channel - 11) << 4 | 3);

    self.set_RFCTL(0x04);
    self.timer.wait_us(200);
    self.set_RFCTL(0x00);
    self.timer.wait_us(200);
  }

  fn enable_interrupts(&self) {
    self.set_INTCON(0b11110110);  // enable rx & tx interrupts
  }

  fn reg_read_short(&self, adr: u8) -> u8 {
    self.cs.set_low();
    self.spi.transfer((adr << 1) & 0b01111110);
    let val = self.spi.transfer(0);
    self.cs.set_high();
    val
  }

  fn reg_read_long(&self, adr: u16) -> u8 {
    self.cs.set_low();
    // Long address read
    // 01 A9 A8 A7 A6 A5 A4 A3 A2 A1 A0 00
    //  7  6  5  4  3  2  1  0              MSB
    //                          7  6  5  4  LSB
    let msb: u8 = (adr >> 3) as u8;
    let lsb: u8 = (adr << 5) as u8;
    self.spi.transfer(0x80 | msb);
    self.spi.transfer(lsb);
    let val = self.spi.transfer(0);
    self.cs.set_high();
    val
  }

  fn reg_write_short(&self, adr: u8, val: u8) {
    self.cs.set_low();
    self.spi.transfer((adr << 1 & 0b01111110) | 1);
    self.spi.transfer(val);
    self.cs.set_high();
  }

  fn reg_write_long(&self, adr: u16, val: u8) {
    self.cs.set_low();
    let msb: u8 = (adr >> 3) as u8;
    let lsb: u8 = (adr << 5) as u8;
    self.spi.transfer(0x80 | msb);
    self.spi.transfer(lsb | 0x10);
    self.spi.transfer(val);
    self.cs.set_high();
  }
}

macro_rules! reg_short_rw(
  ($getter_name:ident, $setter_name:ident, $adr:expr) => (
    impl<'a, S: SPI, T: Timer> Mrf24j40<'a, S, T> {
      #[inline(always)]
      pub fn $getter_name(&self) -> u8 {
        self.reg_read_short($adr)
      }

      #[inline(always)]
      pub fn $setter_name(&self, val: u8) {
        self.reg_write_short($adr, val);
      }
    }
  )
)

macro_rules! reg_long_rw(
  ($getter_name:ident, $setter_name:ident, $adr:expr) => (
    impl<'a, S: SPI, T: Timer> Mrf24j40<'a, S, T> {
      #[inline(always)]
      pub fn $getter_name(&self) -> u8 {
        self.reg_read_long($adr)
      }

      #[inline(always)]
      pub fn $setter_name(&self, val: u8) {
        self.reg_write_long($adr, val);
      }
    }
  )
)

reg_short_rw!(RXMCR,      set_RXMCR,      0x00)
reg_short_rw!(PANIDL,     set_PANIDL,     0x01)
reg_short_rw!(PANIDH,     set_PANIDH,     0x02)
reg_short_rw!(SADRL,      set_SADRL,      0x03)
reg_short_rw!(SADRH,      set_SADRH,      0x04)
reg_short_rw!(EADR0,      set_EADR0,      0x05)
reg_short_rw!(EADR1,      set_EADR1,      0x06)
reg_short_rw!(EADR2,      set_EADR2,      0x07)
reg_short_rw!(EADR3,      set_EADR3,      0x08)
reg_short_rw!(EADR4,      set_EADR4,      0x09)
reg_short_rw!(EADR5,      set_EADR5,      0x0a)
reg_short_rw!(EADR6,      set_EADR6,      0x0b)
reg_short_rw!(EADR7,      set_EADR7,      0x0c)
reg_short_rw!(RXFLUSH,    set_RXFLUSH,    0x0d)
reg_short_rw!(ORDER,      set_ORDER,      0x10)
reg_short_rw!(TXMCR,      set_TXMCR,      0x11)
reg_short_rw!(ACKTIMEOUT, set_ACKTIMEOUT, 0x12)
reg_short_rw!(ESLOTG1,    set_ESLOTG1,    0x13)
reg_short_rw!(SYMTICKL,   set_SYMTICKL,   0x14)
reg_short_rw!(SYMTICKH,   set_SYMTICKH,   0x15)
reg_short_rw!(PACON0,     set_PACON0,     0x16)
reg_short_rw!(PACON1,     set_PACON1,     0x17)
reg_short_rw!(PACON2,     set_PACON2,     0x18)
reg_short_rw!(TXBCON0,    set_TXBCON0,    0x1a)
reg_short_rw!(TXNCON,     set_TXNCON,     0x1b)
reg_short_rw!(TXG1CON,    set_TXG1CON,    0x1c)
reg_short_rw!(TXG2CON,    set_TXG2CON,    0x1d)
reg_short_rw!(ESLOTG23,   set_ESLOTG23,   0x1e)
reg_short_rw!(ESLOTG45,   set_ESLOTG45,   0x1f)
reg_short_rw!(ESLOTG67,   set_ESLOTG67,   0x20)
reg_short_rw!(TXPEND,     set_TXPEND,     0x21)
reg_short_rw!(WAKECON,    set_WAKECON,    0x22)
reg_short_rw!(FRMOFFSET,  set_FRMOFFSET,  0x23)
reg_short_rw!(TXSTAT,     set_TXSTAT,     0x24)
reg_short_rw!(TXBCON1,    set_TXBCON1,    0x25)
reg_short_rw!(GATECLK,    set_GATECLK,    0x26)
reg_short_rw!(TXTIME,     set_TXTIME,     0x27)
reg_short_rw!(HSYMTMRL,   set_HSYMTMRL,   0x28)
reg_short_rw!(HSYMTMRH,   set_HSYMTMRH,   0x29)
reg_short_rw!(SOFTRST,    set_SOFTRST,    0x2a)
reg_short_rw!(SECCON0,    set_SECCON0,    0x2c)
reg_short_rw!(SECCON1,    set_SECCON1,    0x2d)
reg_short_rw!(TXSTBL,     set_TXSTBL,     0x2e)
reg_short_rw!(RXSR,       set_RXSR,       0x30)
reg_short_rw!(INTSTAT,    set_INTSTAT,    0x31)
reg_short_rw!(INTCON,     set_INTCON,     0x32)
reg_short_rw!(GPIO,       set_GPIO,       0x33)
reg_short_rw!(TRISGPIO,   set_TRISGPIO,   0x34)
reg_short_rw!(SLPACK,     set_SLPACK,     0x35)
reg_short_rw!(RFCTL,      set_RFCTL,      0x36)
reg_short_rw!(SECCR2,     set_SECCR2,     0x37)
reg_short_rw!(BBREG0,     set_BBREG0,     0x38)
reg_short_rw!(BBREG1,     set_BBREG1,     0x39)
reg_short_rw!(BBREG2,     set_BBREG2,     0x3a)
reg_short_rw!(BBREG3,     set_BBREG3,     0x3b)
reg_short_rw!(BBREG4,     set_BBREG4,     0x3c)
reg_short_rw!(BBREG6,     set_BBREG6,     0x3e)
reg_short_rw!(CCAEDTH,    set_CCAEDTH,    0x3f)

reg_long_rw!(RFCON0,    set_RFCON0,    0x200)
reg_long_rw!(RFCON1,    set_RFCON1,    0x201)
reg_long_rw!(RFCON2,    set_RFCON2,    0x202)
reg_long_rw!(RFCON3,    set_RFCON3,    0x203)
reg_long_rw!(RFCON5,    set_RFCON5,    0x205)
reg_long_rw!(RFCON6,    set_RFCON6,    0x206)
reg_long_rw!(RFCON7,    set_RFCON7,    0x207)
reg_long_rw!(RFCON8,    set_RFCON8,    0x208)
reg_long_rw!(SLPCAL0,   set_SLPCAL0,   0x209)
reg_long_rw!(SLPCAL1,   set_SLPCAL1,   0x20a)
reg_long_rw!(SLPCAL2,   set_SLPCAL2,   0x20b)
reg_long_rw!(RFSTATE,   set_RFSTATE,   0x20f)
reg_long_rw!(RSSI,      set_RSSI,      0x210)
reg_long_rw!(SLPCON0,   set_SLPCON0,   0x211)
reg_long_rw!(SLPCON1,   set_SLPCON1,   0x220)
reg_long_rw!(WAKETIMEL, set_WAKETIMEL, 0x222)
reg_long_rw!(WAKETIMEH, set_WAKETIMEH, 0x223)
reg_long_rw!(REMCNTL,   set_REMCNTL,   0x224)
reg_long_rw!(REMCNTH,   set_REMCNTH,   0x225)
reg_long_rw!(MAINCNT1,  set_MAINCNT1,  0x227)
reg_long_rw!(MAINCNT2,  set_MAINCNT2,  0x228)
reg_long_rw!(MAINCNT3,  set_MAINCNT3,  0x229)
reg_long_rw!(TESTMODE,  set_TESTMODE,  0x22f)
reg_long_rw!(ASSOEADR0, set_ASSOEADR0, 0x230)
reg_long_rw!(ASSOEADR1, set_ASSOEADR1, 0x231)
reg_long_rw!(ASSOEADR2, set_ASSOEADR2, 0x232)
reg_long_rw!(ASSOEADR3, set_ASSOEADR3, 0x233)
reg_long_rw!(ASSOEADR4, set_ASSOEADR4, 0x234)
reg_long_rw!(ASSOEADR5, set_ASSOEADR5, 0x235)
reg_long_rw!(ASSOEADR6, set_ASSOEADR6, 0x236)
reg_long_rw!(ASSOEADR7, set_ASSOEADR7, 0x237)
reg_long_rw!(ASSOSADR0, set_ASSOSADR0, 0x238)
reg_long_rw!(ASSOSADR1, set_ASSOSADR1, 0x239)
reg_long_rw!(UPNONCE0,  set_UPNONCE0,  0x240)
reg_long_rw!(UPNONCE1,  set_UPNONCE1,  0x241)
reg_long_rw!(UPNONCE2,  set_UPNONCE2,  0x242)
reg_long_rw!(UPNONCE3,  set_UPNONCE3,  0x243)
reg_long_rw!(UPNONCE4,  set_UPNONCE4,  0x244)
reg_long_rw!(UPNONCE5,  set_UPNONCE5,  0x245)
reg_long_rw!(UPNONCE6,  set_UPNONCE6,  0x246)
reg_long_rw!(UPNONCE7,  set_UPNONCE7,  0x247)
reg_long_rw!(UPNONCE8,  set_UPNONCE8,  0x248)
reg_long_rw!(UPNONCE9,  set_UPNONCE9,  0x249)
reg_long_rw!(UPNONCE10, set_UPNONCE10, 0x24a)
reg_long_rw!(UPNONCE11, set_UPNONCE11, 0x24b)
reg_long_rw!(UPNONCE12, set_UPNONCE12, 0x24c)
