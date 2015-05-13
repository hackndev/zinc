// Zinc, the bare metal stack for rust.
// Copyright 2014 Dawid Ciężarkiewcz <dpc@ucore.info>
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

//! Watchdog for Kinetis SIM module.

use util::support::nop;

#[path="../../util/ioreg.rs"] mod ioreg;

/// Watchdog state
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum State {
  Disabled,
  Enabled,
}

/// Init watchdog
pub fn init(state : State) {
  use self::State::*;
  unlock();
  match state {
    Disabled => {
      reg::WDOG.stctrlh.set_en(false);
    },
    Enabled => {
      reg::WDOG.stctrlh.set_allowupdate(true);
    },
  }
}

fn unlock() {
  use self::reg::WDOG_unlock_unlock::*;
  reg::WDOG.unlock.set_unlock(UnlockSeq1);
  reg::WDOG.unlock.set_unlock(UnlockSeq2);

  // Enforce one cycle delay
  nop();
}

/// Write refresh sequence to refresh watchdog
pub fn refresh() {
  use self::reg::WDOG_refresh_refresh::*;
  reg::WDOG.refresh.set_refresh(RefreshSeq1);
  reg::WDOG.refresh.set_refresh(RefreshSeq2);
}

#[allow(dead_code)]
mod reg {
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(WDOG = {
  /// Status and Control Register High
    0x0 => reg16 stctrlh
    {
      0 => en,             //= Watchdog enable
      4 => allowupdate     //= Enables updates to watchdog write-once registers,
                           //= after the reset-triggered initial configuration window
    },

    /// Refresh Register
    0xc => reg16 refresh {
      0..15 => refresh: wo
      {
        0xa602 => RefreshSeq1,
        0xb480 => RefreshSeq2,
      },
    },

    /// Unlock Register
    0xe => reg16 unlock {
      0..15 => unlock: wo
      {
        0xc520 => UnlockSeq1,
        0xd928 => UnlockSeq2,
      },
    },

  });


  extern {
    #[link_name="k20_iomem_WDOG"] pub static WDOG: WDOG;
  }
}
