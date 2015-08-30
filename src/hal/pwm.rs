// Zinc, the bare metal stack for rust.
// Copyright 2015 Paul Osborne <osbpau@gmail.com>
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

//! Common definitions for all PWM on all MCUs

/// Trait for any Puluse Width Modulated output
///
/// This interface is inspired by the mbed `PWMOut` interface.
///
/// Note that on some MCUs, the period may be dictated by
/// a timer shared by several differents PWMs.  It is not
/// guaranteed that the period and pulsewidth specified
/// will remain identical if modified elsewhere.  A proper
/// implementaiton will still seek to maintain a similar
/// duty cycle in the case of a period change.
pub trait PWMOutput {
  /// set the period in microseconds
  fn set_period_us(&mut self, period_us: u32);

  /// get the period in microseconds
  fn get_period_us(&self) -> u32;

  /// Set the pulse width in microseconds
  fn set_pulsewidth_us(&mut self, pulsewidth_us: u32);

  /// get the duty cycle as a percentage
  fn get_pulsewidth_us(&self) -> u32;

  /// Set the duty cycle for this PWMOutput to the given percentage
  ///
  /// duty_cycle is expected to be a number between 0 and 1.
  /// Numbers below 0 will be set to 0 and above 1 will be set to
  /// 1.0.
  fn write(&mut self, duty_cycle: f32) {
    let adj_duty_cycle = if duty_cycle < 0.0 {
      0.0
    } else if duty_cycle > 1.0 {
      1.0
    } else {
      duty_cycle
    };

    // assume the period is acceptable and adjust pulsewidth only
    let pulsewidth_us = (adj_duty_cycle * self.get_period_us() as f32) as u32;
    self.set_pulsewidth_us(pulsewidth_us)
  }
}
