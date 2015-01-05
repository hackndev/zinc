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

//! Driver for the Hitachi HD44780U LCD driver

use core::prelude::*;

use drivers::chario::CharIO;
use hal::pin::Gpio;
use hal::timer::Timer;

/// HD44780U driver context
pub struct Hd44780u<'a> {
  /// Timer used for protocol delays
  timer: &'a (Timer + 'a),
  /// Register Selector wire
  rs:    &'a (Gpio + 'a),
  /// Enable wire
  en:    &'a (Gpio + 'a),
  /// The 4 data wires. Those must be wired to [d4, d5, d6, d7] in 4bit mode.
  data: [&'a (Gpio + 'a); 4],
}

/// The controller supports writing in either direction to accomodate various
/// languages.
#[derive(Copy)]
pub enum MoveDir {
  /// Cursor moves right after write
  Right,
  /// Cursor moves left after write
  Left,
}

/// The controller supports 5x8 and 5x10 dot fonts depending on the LCD used.
#[derive(Copy)]
pub enum Font {
  /// Use 5x8 dot matrix font
  Font5x8,
  /// Use 5x10 dot matrix font
  Font5x10,
}

impl<'a> Hd44780u<'a> {
  /// Construct an Hd44780u instance
  pub fn new(timer: &'a (Timer + 'a),
                 rs:    &'a (Gpio  + 'a),
                 en:    &'a (Gpio  + 'a),
                 data: [&'a (Gpio  + 'a); 4]) -> Hd44780u<'a> {
    Hd44780u { timer: timer, rs: rs, en: en, data: data }
  }

  /// Power up sequence for 4bit mode as detailed in page 46 of the datasheet.
  /// The two_lines and font parameters are used to set the LCD parameters. See
  /// `function_set` for more details.
  pub fn init(&self, two_lines: bool, font: Font) {
    // Start by pulling RS and EN low
    self.rs.set_low();
    self.en.set_low();

    // We need to wait at least 40ms after the chip is powered before we can
    // talk to it. Assume that the power just went on and wait.
    self.timer.wait_ms(50);

    // Set interface to 8bit
    self.write_nibble(0b0011);

    // Wait 4.1ms
    self.timer.wait_us(4100);

    // Set interface to 8bit
    self.write_nibble(0b0011);

    // Wait 100us
    self.timer.wait_us(100);

    // Set interface to 8bit (for the last time...)
    self.write_nibble(0b0011);
    self.timer.wait_us(100);

    // We can now finally switch to 4bit
    self.write_nibble(0b0010);
    self.timer.wait_us(100);

    // Set function
    self.function_set(two_lines, font);

    self.display_control(true, false, false);

    self.clear();

    // Set default mode
    self.mode_set(MoveDir::Right, false);
  }

  /// Set cursor position to (`col`, `row`). (0, 0) is the top left.
  pub fn set_pos(&self, col: u8, row: u8) {
    // Rows are not continuous in DDRAM, there's a 0x40 offset between the
    // begining of each line.
    self.ddram_address_set(row * 0x40 + col);
  }

  /// Clear the entire display
  pub fn clear(&self) {
    self.instruction(0b0001);
    // I can't find in the datasheet how long this is supposed to take but it
    // seems to take at least a few ms.
    self.timer.wait_ms(5);
  }

  /// Return the cursor to the left of the first line of the display. Also
  /// returns the display to its original status if it was shifted (see
  /// `mode_set`).
  pub fn return_home(&self) {
    self.instruction(0b10);
    // This takes 1.52ms
    self.timer.wait_ms(2);
  }

  /// Set the writing direction and optional display shifting.
  ///
  /// If `shift_display` is true then when writing the cursor remains at the
  /// same place on the display and "pushes" existing data in the direction
  /// specified by dir.
  ///
  /// Calling `clear` resets the writing direction Right but does not change
  /// the display shift setting.
  pub fn mode_set(&self, dir: MoveDir, shift_display: bool) {
    let mut cmd = 0b100;

    let d = match dir {
      MoveDir::Right => 1,
      MoveDir::Left  => 0,
    };

    cmd |= d                     << 1;
    cmd |= (shift_display as u8) << 0;

    self.instruction(cmd);
  }

  /// Display control. If `on` is false nothing is displayed but the screen's
  /// contents remain in RAM and can be re-displayed at any moment. This
  /// function can also enable or disable the cursor and blinking.
  pub fn display_control(&self, on: bool, cursor: bool, blink: bool) {
    let mut cmd = 0b1000;

    cmd |= (on     as u8) << 2;
    cmd |= (cursor as u8) << 1;
    cmd |= (blink  as u8) << 0;

    self.instruction(cmd);
  }

  /// Shift cursor or display in a given direction
  pub fn shift(&self, dir: MoveDir, shift_display: bool) {
    let mut cmd = 0b10000;

    let d = match dir {
      MoveDir::Right => 1,
      MoveDir::Left  => 0,
    };

    cmd |= (shift_display as u8) << 3;
    cmd |= d                     << 2;

    self.instruction(cmd);
  }

  /// Set the bus width, number of lines and character font size. This function
  /// can only be called once in the init sequence.
  ///
  /// Only 4bit wide interfaces are supported for now.
  ///
  /// `two_lines` should be
  /// `false` if the display only has one line, true otherwise.
  ///
  /// If each display character is a matrix of 5x10 dots `char_5x10` should be
  /// `true`. If it's 5x8 dots set it to `false`.
  fn function_set(&self, two_lines: bool, font: Font) {
    use self::Font::*;
    let mut cmd = 0b100000;

    // Only 4bit interface is supported at the moment
    cmd |= 0                 << 4;
    cmd |= (two_lines as u8) << 3;
    cmd |= match font {
      Font5x8  => 0,
      Font5x10 => 1,
    }                        << 2;

    self.instruction(cmd);
  }

  /// Set the curent read/write data address to the Character Generator RAM
  /// offset `addr`
  fn cgram_address_set(&self, addr: u8) {
    let mut cmd = 0b1000000;

    cmd |= addr & 0b111111;

    self.instruction(cmd);
  }

  /// Set the curent read/write data address to the Display Data RAM offset
  /// `addr`
  fn ddram_address_set(&self, addr: u8) {
    let mut cmd = 0b10000000;

    cmd |= addr & 0b1111111;

    self.instruction(cmd);
  }

  /// Create custom 5x8 char at index `pos` from `bitmap`. Only the 5LSBs of
  /// each bitmap lines are used. There can be only 8 custom 5x8 chars so pos
  /// must be in the range 0 ... 7.
  ///
  /// This resets the cursor positon to 0.
  pub fn custom_char_5x8(&self, pos: u8, bitmap: [ u8; 8 ]) {
    if pos > 7 {
      panic!("Invalid character position");
    }

    self.cgram_address_set(pos << 3);

    for b in bitmap.iter() {
      self.data(*b & 0b11111);
    }

    // Return the read/write pointer to the Display Data RAM
    self.set_pos(0, 0);
  }

  /// Create custom 5x10 char at index `pos` from `bitmap`. Only the 5LSBs of
  /// each bitmap lines are used. There can be only 8 custom 5x8 chars so pos
  /// must be in the range 0 ... 3.
  pub fn custom_char_5x10(&self, pos: u8, bitmap: [ u8; 10 ]) {
    if pos > 3 {
      panic!("Invalid character position");
    }

    self.cgram_address_set(pos << 4);

    for b in bitmap.iter() {
      self.data(*b & 0b11111);
    }

    // Return the read/write pointer to the Display Data RAM
    self.set_pos(0, 0);
  }

  /// Send a 8bit command code
  fn instruction(&self, cmd: u8) {
    self.rs.set_low();

    self.write_nibble(cmd >> 4);
    self.write_nibble(cmd);
    // Commands take *at least* 37us to execute, so default to that. If the
    // current instruction needs to delay for more than that the calling
    // function will wait some more.
    self.timer.wait_us(40);
  }

  /// Send 8bit of data
  fn data(&self, data: u8) {
    self.rs.set_high();

    self.write_nibble(data >> 4);
    self.write_nibble(data);
    // Data access take 37us
    self.timer.wait_us(40);
  }

  /// Write a 4bit value to the parallel interface.
  fn write_nibble(&self, v: u8) {
    // Set the 4 data lines in the parallel interface
    match v & 1 {
      0 => self.data[0].set_low(),
      _ => self.data[0].set_high(),
    }

    match v & 2 {
      0 => self.data[1].set_low(),
      _ => self.data[1].set_high(),
    }

    match v & 4 {
      0 => self.data[2].set_low(),
      _ => self.data[2].set_high(),
    }

    match v & 8 {
      0 => self.data[3].set_low(),
      _ => self.data[3].set_high(),
    }

    // Pulse the EN wire to notify the controller.
    self.en.set_high();
    // We need to assert EN for at least 450ns
    self.timer.wait_us(1);
    self.en.set_low();
  }
}

impl<'a> CharIO for Hd44780u<'a> {
  fn putc(&self, value: char) {
    self.data(value as u8);
  }
}
