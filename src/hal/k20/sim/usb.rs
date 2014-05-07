// Zinc, the bare metal stack for rust.
// Copyright 2014 Ben Harris <mail@bharr.is>
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

/*!
System Integration Module - USB controls.
*/

use super::reg;

/// Enables the USB voltage regulator (on by default).
pub fn enable_vreg() {
  reg::SIMLP.set_SOPT1CFG(1 << 24);
  reg::SIMLP.set_SOPT1(reg::SIMLP.SOPT1() | (1 << 31));
}

pub fn disable_vreg() {
  reg::SIMLP.set_SOPT1CFG(1 << 24);
  reg::SIMLP.set_SOPT1(reg::SIMLP.SOPT1() & !(1 << 31));
}

/// Causes the USB voltage regulator to enter standby when the core sleeps.
pub fn enable_vreg_stby() {
  reg::SIMLP.set_SOPT1CFG(1 << 26);
  reg::SIMLP.set_SOPT1(reg::SIMLP.SOPT1() | (1 << 30));
}

pub fn disable_vreg_stby() {
  reg::SIMLP.set_SOPT1CFG(1 << 26);
  reg::SIMLP.set_SOPT1(reg::SIMLP.SOPT1() & !(1 << 30));
}

/// Causes the USB voltage regulator to enter standby during low power mode.
pub fn enable_vreg_lp() {
  reg::SIMLP.set_SOPT1CFG(1 << 25);
  reg::SIMLP.set_SOPT1(reg::SIMLP.SOPT1() | (1 << 30));
}

pub fn disable_vreg_lp() {
  reg::SIMLP.set_SOPT1CFG(1 << 25);
  reg::SIMLP.set_SOPT1(reg::SIMLP.SOPT1() & !(1 << 30));
}

