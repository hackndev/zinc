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

// Automatically generated file, do not edit
// Update the definition in pinmap.rs.rb and re-generate pinmap.rs with
// support/pingen.rb <src> <dst>

pub mod port0 {
    pub mod pin0 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      0,
        function: super::super::super::GPIO,
      };
      pub static RD1: PinConf = PinConf {
        port:     Port0,
        pin:      0,
        function: AltFunction1,
      };
      pub static TXD3: PinConf = PinConf {
        port:     Port0,
        pin:      0,
        function: AltFunction2,
      };
      pub static SDA1: PinConf = PinConf {
        port:     Port0,
        pin:      0,
        function: AltFunction3,
      };
    }
    pub mod pin1 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      1,
        function: super::super::super::GPIO,
      };
      pub static TD1: PinConf = PinConf {
        port:     Port0,
        pin:      1,
        function: AltFunction1,
      };
      pub static RXD3: PinConf = PinConf {
        port:     Port0,
        pin:      1,
        function: AltFunction2,
      };
      pub static SCL1: PinConf = PinConf {
        port:     Port0,
        pin:      1,
        function: AltFunction3,
      };
    }
    pub mod pin2 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      2,
        function: super::super::super::GPIO,
      };
      pub static TXD0: PinConf = PinConf {
        port:     Port0,
        pin:      2,
        function: AltFunction1,
      };
      pub static AD0_7: PinConf = PinConf {
        port:     Port0,
        pin:      2,
        function: AltFunction2,
      };
    }
    pub mod pin3 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      3,
        function: super::super::super::GPIO,
      };
      pub static RXD0: PinConf = PinConf {
        port:     Port0,
        pin:      3,
        function: AltFunction1,
      };
      pub static AD0_6: PinConf = PinConf {
        port:     Port0,
        pin:      3,
        function: AltFunction2,
      };
    }
    pub mod pin4 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      4,
        function: super::super::super::GPIO,
      };
      pub static I2SRX_CLK: PinConf = PinConf {
        port:     Port0,
        pin:      4,
        function: AltFunction1,
      };
      pub static RD2: PinConf = PinConf {
        port:     Port0,
        pin:      4,
        function: AltFunction2,
      };
      pub static CAP2_0: PinConf = PinConf {
        port:     Port0,
        pin:      4,
        function: AltFunction3,
      };
    }
    pub mod pin5 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      5,
        function: super::super::super::GPIO,
      };
      pub static I2SRX_WS: PinConf = PinConf {
        port:     Port0,
        pin:      5,
        function: AltFunction1,
      };
      pub static TD2: PinConf = PinConf {
        port:     Port0,
        pin:      5,
        function: AltFunction2,
      };
      pub static CAP2_1: PinConf = PinConf {
        port:     Port0,
        pin:      5,
        function: AltFunction3,
      };
    }
    pub mod pin6 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      6,
        function: super::super::super::GPIO,
      };
      pub static I2SRX_SDA: PinConf = PinConf {
        port:     Port0,
        pin:      6,
        function: AltFunction1,
      };
      pub static SSEL1: PinConf = PinConf {
        port:     Port0,
        pin:      6,
        function: AltFunction2,
      };
      pub static MAT2_0: PinConf = PinConf {
        port:     Port0,
        pin:      6,
        function: AltFunction3,
      };
    }
    pub mod pin7 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      7,
        function: super::super::super::GPIO,
      };
      pub static I2STX_CLK: PinConf = PinConf {
        port:     Port0,
        pin:      7,
        function: AltFunction1,
      };
      pub static SCK1: PinConf = PinConf {
        port:     Port0,
        pin:      7,
        function: AltFunction2,
      };
      pub static MAT2_1: PinConf = PinConf {
        port:     Port0,
        pin:      7,
        function: AltFunction3,
      };
    }
    pub mod pin8 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      8,
        function: super::super::super::GPIO,
      };
      pub static I2STX_WS: PinConf = PinConf {
        port:     Port0,
        pin:      8,
        function: AltFunction1,
      };
      pub static MISO1: PinConf = PinConf {
        port:     Port0,
        pin:      8,
        function: AltFunction2,
      };
      pub static MAT2_2: PinConf = PinConf {
        port:     Port0,
        pin:      8,
        function: AltFunction3,
      };
    }
    pub mod pin9 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      9,
        function: super::super::super::GPIO,
      };
      pub static I2STX_SDA: PinConf = PinConf {
        port:     Port0,
        pin:      9,
        function: AltFunction1,
      };
      pub static MOSI1: PinConf = PinConf {
        port:     Port0,
        pin:      9,
        function: AltFunction2,
      };
      pub static MAT2_3: PinConf = PinConf {
        port:     Port0,
        pin:      9,
        function: AltFunction3,
      };
    }
    pub mod pin10 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      10,
        function: super::super::super::GPIO,
      };
      pub static TXD2: PinConf = PinConf {
        port:     Port0,
        pin:      10,
        function: AltFunction1,
      };
      pub static SDA2: PinConf = PinConf {
        port:     Port0,
        pin:      10,
        function: AltFunction2,
      };
      pub static MAT3_0: PinConf = PinConf {
        port:     Port0,
        pin:      10,
        function: AltFunction3,
      };
    }
    pub mod pin11 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      11,
        function: super::super::super::GPIO,
      };
      pub static RXD2: PinConf = PinConf {
        port:     Port0,
        pin:      11,
        function: AltFunction1,
      };
      pub static SCL2: PinConf = PinConf {
        port:     Port0,
        pin:      11,
        function: AltFunction2,
      };
      pub static MAT3_1: PinConf = PinConf {
        port:     Port0,
        pin:      11,
        function: AltFunction3,
      };
    }
    pub mod pin15 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      15,
        function: super::super::super::GPIO,
      };
      pub static TXD1: PinConf = PinConf {
        port:     Port0,
        pin:      15,
        function: AltFunction1,
      };
      pub static SCK0: PinConf = PinConf {
        port:     Port0,
        pin:      15,
        function: AltFunction2,
      };
      pub static SCK: PinConf = PinConf {
        port:     Port0,
        pin:      15,
        function: AltFunction3,
      };
    }
    pub mod pin16 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      16,
        function: super::super::super::GPIO,
      };
      pub static RXD1: PinConf = PinConf {
        port:     Port0,
        pin:      16,
        function: AltFunction1,
      };
      pub static SSEL0: PinConf = PinConf {
        port:     Port0,
        pin:      16,
        function: AltFunction2,
      };
      pub static SSEL: PinConf = PinConf {
        port:     Port0,
        pin:      16,
        function: AltFunction3,
      };
    }
    pub mod pin17 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      17,
        function: super::super::super::GPIO,
      };
      pub static CTS1: PinConf = PinConf {
        port:     Port0,
        pin:      17,
        function: AltFunction1,
      };
      pub static MISO0: PinConf = PinConf {
        port:     Port0,
        pin:      17,
        function: AltFunction2,
      };
      pub static MISO: PinConf = PinConf {
        port:     Port0,
        pin:      17,
        function: AltFunction3,
      };
    }
    pub mod pin18 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      18,
        function: super::super::super::GPIO,
      };
      pub static DCD1: PinConf = PinConf {
        port:     Port0,
        pin:      18,
        function: AltFunction1,
      };
      pub static MOSI0: PinConf = PinConf {
        port:     Port0,
        pin:      18,
        function: AltFunction2,
      };
      pub static MOSI: PinConf = PinConf {
        port:     Port0,
        pin:      18,
        function: AltFunction3,
      };
    }
    pub mod pin19 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      19,
        function: super::super::super::GPIO,
      };
      pub static DSR1: PinConf = PinConf {
        port:     Port0,
        pin:      19,
        function: AltFunction1,
      };
      pub static SDA1: PinConf = PinConf {
        port:     Port0,
        pin:      19,
        function: AltFunction3,
      };
    }
    pub mod pin20 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      20,
        function: super::super::super::GPIO,
      };
      pub static DTR1: PinConf = PinConf {
        port:     Port0,
        pin:      20,
        function: AltFunction1,
      };
      pub static SCL1: PinConf = PinConf {
        port:     Port0,
        pin:      20,
        function: AltFunction3,
      };
    }
    pub mod pin21 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      21,
        function: super::super::super::GPIO,
      };
      pub static RI1: PinConf = PinConf {
        port:     Port0,
        pin:      21,
        function: AltFunction1,
      };
      pub static RD1: PinConf = PinConf {
        port:     Port0,
        pin:      21,
        function: AltFunction3,
      };
    }
    pub mod pin22 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      22,
        function: super::super::super::GPIO,
      };
      pub static RTS1: PinConf = PinConf {
        port:     Port0,
        pin:      22,
        function: AltFunction1,
      };
      pub static TD1: PinConf = PinConf {
        port:     Port0,
        pin:      22,
        function: AltFunction3,
      };
    }
    pub mod pin23 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      23,
        function: super::super::super::GPIO,
      };
      pub static AD0_0: PinConf = PinConf {
        port:     Port0,
        pin:      23,
        function: AltFunction1,
      };
      pub static I2SRX_CLK: PinConf = PinConf {
        port:     Port0,
        pin:      23,
        function: AltFunction2,
      };
      pub static CAP3_0: PinConf = PinConf {
        port:     Port0,
        pin:      23,
        function: AltFunction3,
      };
    }
    pub mod pin24 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      24,
        function: super::super::super::GPIO,
      };
      pub static AD0_1: PinConf = PinConf {
        port:     Port0,
        pin:      24,
        function: AltFunction1,
      };
      pub static I2SRX_WS: PinConf = PinConf {
        port:     Port0,
        pin:      24,
        function: AltFunction2,
      };
      pub static CAP3_1: PinConf = PinConf {
        port:     Port0,
        pin:      24,
        function: AltFunction3,
      };
    }
    pub mod pin25 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      25,
        function: super::super::super::GPIO,
      };
      pub static AD0_2: PinConf = PinConf {
        port:     Port0,
        pin:      25,
        function: AltFunction1,
      };
      pub static I2SRX_SDA: PinConf = PinConf {
        port:     Port0,
        pin:      25,
        function: AltFunction2,
      };
      pub static TXD3: PinConf = PinConf {
        port:     Port0,
        pin:      25,
        function: AltFunction3,
      };
    }
    pub mod pin26 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      26,
        function: super::super::super::GPIO,
      };
      pub static AD0_3: PinConf = PinConf {
        port:     Port0,
        pin:      26,
        function: AltFunction1,
      };
      pub static AOUT: PinConf = PinConf {
        port:     Port0,
        pin:      26,
        function: AltFunction2,
      };
      pub static TXD3: PinConf = PinConf {
        port:     Port0,
        pin:      26,
        function: AltFunction3,
      };
    }
    pub mod pin27 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      27,
        function: super::super::super::GPIO,
      };
      pub static SDA0: PinConf = PinConf {
        port:     Port0,
        pin:      27,
        function: AltFunction1,
      };
      pub static USB_SDA: PinConf = PinConf {
        port:     Port0,
        pin:      27,
        function: AltFunction2,
      };
    }
    pub mod pin28 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      28,
        function: super::super::super::GPIO,
      };
      pub static SCL0: PinConf = PinConf {
        port:     Port0,
        pin:      28,
        function: AltFunction1,
      };
      pub static USB_SCL: PinConf = PinConf {
        port:     Port0,
        pin:      28,
        function: AltFunction2,
      };
    }
    pub mod pin29 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      29,
        function: super::super::super::GPIO,
      };
      pub static USB_D_pos: PinConf = PinConf {
        port:     Port0,
        pin:      29,
        function: AltFunction1,
      };
    }
    pub mod pin30 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port0,
        pin:      30,
        function: super::super::super::GPIO,
      };
      pub static USB_D_neg: PinConf = PinConf {
        port:     Port0,
        pin:      30,
        function: AltFunction1,
      };
    }
}
pub mod port1 {
    pub mod pin0 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      0,
        function: super::super::super::GPIO,
      };
      pub static ENET_TXD0: PinConf = PinConf {
        port:     Port1,
        pin:      0,
        function: AltFunction1,
      };
    }
    pub mod pin1 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      1,
        function: super::super::super::GPIO,
      };
      pub static ENET_TXD1: PinConf = PinConf {
        port:     Port1,
        pin:      1,
        function: AltFunction1,
      };
    }
    pub mod pin4 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      4,
        function: super::super::super::GPIO,
      };
      pub static ENET_TX_EN: PinConf = PinConf {
        port:     Port1,
        pin:      4,
        function: AltFunction1,
      };
    }
    pub mod pin8 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      8,
        function: super::super::super::GPIO,
      };
      pub static ENET_CRS: PinConf = PinConf {
        port:     Port1,
        pin:      8,
        function: AltFunction1,
      };
    }
    pub mod pin9 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      9,
        function: super::super::super::GPIO,
      };
      pub static ENET_RXD0: PinConf = PinConf {
        port:     Port1,
        pin:      9,
        function: AltFunction1,
      };
    }
    pub mod pin10 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      10,
        function: super::super::super::GPIO,
      };
      pub static ENET_RXD1: PinConf = PinConf {
        port:     Port1,
        pin:      10,
        function: AltFunction1,
      };
    }
    pub mod pin14 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      14,
        function: super::super::super::GPIO,
      };
      pub static ENET_RX_ER: PinConf = PinConf {
        port:     Port1,
        pin:      14,
        function: AltFunction1,
      };
    }
    pub mod pin15 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      15,
        function: super::super::super::GPIO,
      };
      pub static ENET_REF_CLCK: PinConf = PinConf {
        port:     Port1,
        pin:      15,
        function: AltFunction1,
      };
    }
    pub mod pin16 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      16,
        function: super::super::super::GPIO,
      };
      pub static ENET_MDC: PinConf = PinConf {
        port:     Port1,
        pin:      16,
        function: AltFunction1,
      };
    }
    pub mod pin17 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      17,
        function: super::super::super::GPIO,
      };
      pub static ENET_MDIO: PinConf = PinConf {
        port:     Port1,
        pin:      17,
        function: AltFunction1,
      };
    }
    pub mod pin18 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      18,
        function: super::super::super::GPIO,
      };
      pub static USB_UP_LED: PinConf = PinConf {
        port:     Port1,
        pin:      18,
        function: AltFunction1,
      };
      pub static PWM1_1: PinConf = PinConf {
        port:     Port1,
        pin:      18,
        function: AltFunction2,
      };
      pub static CAP1_0: PinConf = PinConf {
        port:     Port1,
        pin:      18,
        function: AltFunction3,
      };
    }
    pub mod pin19 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      19,
        function: super::super::super::GPIO,
      };
      pub static MCOA0: PinConf = PinConf {
        port:     Port1,
        pin:      19,
        function: AltFunction1,
      };
      pub static USB_PPWR: PinConf = PinConf {
        port:     Port1,
        pin:      19,
        function: AltFunction2,
      };
      pub static CAP1_1: PinConf = PinConf {
        port:     Port1,
        pin:      19,
        function: AltFunction3,
      };
    }
    pub mod pin20 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      20,
        function: super::super::super::GPIO,
      };
      pub static MCI0: PinConf = PinConf {
        port:     Port1,
        pin:      20,
        function: AltFunction1,
      };
      pub static PWM1_2: PinConf = PinConf {
        port:     Port1,
        pin:      20,
        function: AltFunction2,
      };
      pub static SCK0: PinConf = PinConf {
        port:     Port1,
        pin:      20,
        function: AltFunction3,
      };
    }
    pub mod pin21 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      21,
        function: super::super::super::GPIO,
      };
      pub static MCABORT: PinConf = PinConf {
        port:     Port1,
        pin:      21,
        function: AltFunction1,
      };
      pub static PWM1_3: PinConf = PinConf {
        port:     Port1,
        pin:      21,
        function: AltFunction2,
      };
      pub static SSEL0: PinConf = PinConf {
        port:     Port1,
        pin:      21,
        function: AltFunction3,
      };
    }
    pub mod pin22 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      22,
        function: super::super::super::GPIO,
      };
      pub static MCOB0: PinConf = PinConf {
        port:     Port1,
        pin:      22,
        function: AltFunction1,
      };
      pub static USB_PWRD: PinConf = PinConf {
        port:     Port1,
        pin:      22,
        function: AltFunction2,
      };
      pub static MAT1_0: PinConf = PinConf {
        port:     Port1,
        pin:      22,
        function: AltFunction3,
      };
    }
    pub mod pin23 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      23,
        function: super::super::super::GPIO,
      };
      pub static MCI1: PinConf = PinConf {
        port:     Port1,
        pin:      23,
        function: AltFunction1,
      };
      pub static PWM1_4: PinConf = PinConf {
        port:     Port1,
        pin:      23,
        function: AltFunction2,
      };
      pub static MISO0: PinConf = PinConf {
        port:     Port1,
        pin:      23,
        function: AltFunction3,
      };
    }
    pub mod pin24 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      24,
        function: super::super::super::GPIO,
      };
      pub static MCI2: PinConf = PinConf {
        port:     Port1,
        pin:      24,
        function: AltFunction1,
      };
      pub static PWM1_5: PinConf = PinConf {
        port:     Port1,
        pin:      24,
        function: AltFunction2,
      };
      pub static MOSI0: PinConf = PinConf {
        port:     Port1,
        pin:      24,
        function: AltFunction3,
      };
    }
    pub mod pin25 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      25,
        function: super::super::super::GPIO,
      };
      pub static MCOA1: PinConf = PinConf {
        port:     Port1,
        pin:      25,
        function: AltFunction1,
      };
      pub static MAT1_1: PinConf = PinConf {
        port:     Port1,
        pin:      25,
        function: AltFunction3,
      };
    }
    pub mod pin26 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      26,
        function: super::super::super::GPIO,
      };
      pub static MCOB1: PinConf = PinConf {
        port:     Port1,
        pin:      26,
        function: AltFunction1,
      };
      pub static PWM1_6: PinConf = PinConf {
        port:     Port1,
        pin:      26,
        function: AltFunction2,
      };
      pub static CAP0_0: PinConf = PinConf {
        port:     Port1,
        pin:      26,
        function: AltFunction3,
      };
    }
    pub mod pin27 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      27,
        function: super::super::super::GPIO,
      };
      pub static CLKOUT: PinConf = PinConf {
        port:     Port1,
        pin:      27,
        function: AltFunction1,
      };
      pub static USB_OVRCR: PinConf = PinConf {
        port:     Port1,
        pin:      27,
        function: AltFunction2,
      };
      pub static CAP0_1: PinConf = PinConf {
        port:     Port1,
        pin:      27,
        function: AltFunction3,
      };
    }
    pub mod pin28 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      28,
        function: super::super::super::GPIO,
      };
      pub static MCOA2: PinConf = PinConf {
        port:     Port1,
        pin:      28,
        function: AltFunction1,
      };
      pub static PCAP1_0: PinConf = PinConf {
        port:     Port1,
        pin:      28,
        function: AltFunction2,
      };
      pub static MAT0_0: PinConf = PinConf {
        port:     Port1,
        pin:      28,
        function: AltFunction3,
      };
    }
    pub mod pin29 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      29,
        function: super::super::super::GPIO,
      };
      pub static MCOB2: PinConf = PinConf {
        port:     Port1,
        pin:      29,
        function: AltFunction1,
      };
      pub static PCAP1_1: PinConf = PinConf {
        port:     Port1,
        pin:      29,
        function: AltFunction2,
      };
      pub static MAT0_1: PinConf = PinConf {
        port:     Port1,
        pin:      29,
        function: AltFunction3,
      };
    }
    pub mod pin30 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      30,
        function: super::super::super::GPIO,
      };
      pub static Vbus: PinConf = PinConf {
        port:     Port1,
        pin:      30,
        function: AltFunction2,
      };
      pub static AD0_4: PinConf = PinConf {
        port:     Port1,
        pin:      30,
        function: AltFunction3,
      };
    }
    pub mod pin31 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port1,
        pin:      31,
        function: super::super::super::GPIO,
      };
      pub static SCK1: PinConf = PinConf {
        port:     Port1,
        pin:      31,
        function: AltFunction2,
      };
      pub static AD0_5: PinConf = PinConf {
        port:     Port1,
        pin:      31,
        function: AltFunction3,
      };
    }
}
pub mod port2 {
    pub mod pin0 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      0,
        function: super::super::super::GPIO,
      };
      pub static PWM1_1: PinConf = PinConf {
        port:     Port2,
        pin:      0,
        function: AltFunction1,
      };
      pub static TXD1: PinConf = PinConf {
        port:     Port2,
        pin:      0,
        function: AltFunction2,
      };
    }
    pub mod pin1 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      1,
        function: super::super::super::GPIO,
      };
      pub static PWM1_2: PinConf = PinConf {
        port:     Port2,
        pin:      1,
        function: AltFunction1,
      };
      pub static RXD1: PinConf = PinConf {
        port:     Port2,
        pin:      1,
        function: AltFunction2,
      };
    }
    pub mod pin2 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      2,
        function: super::super::super::GPIO,
      };
      pub static PWM1_3: PinConf = PinConf {
        port:     Port2,
        pin:      2,
        function: AltFunction1,
      };
      pub static CTS1: PinConf = PinConf {
        port:     Port2,
        pin:      2,
        function: AltFunction2,
      };
    }
    pub mod pin3 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      3,
        function: super::super::super::GPIO,
      };
      pub static PWM1_4: PinConf = PinConf {
        port:     Port2,
        pin:      3,
        function: AltFunction1,
      };
      pub static DCD1: PinConf = PinConf {
        port:     Port2,
        pin:      3,
        function: AltFunction2,
      };
    }
    pub mod pin4 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      4,
        function: super::super::super::GPIO,
      };
      pub static PWM1_5: PinConf = PinConf {
        port:     Port2,
        pin:      4,
        function: AltFunction1,
      };
      pub static DSR1: PinConf = PinConf {
        port:     Port2,
        pin:      4,
        function: AltFunction2,
      };
    }
    pub mod pin5 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      5,
        function: super::super::super::GPIO,
      };
      pub static PWM1_6: PinConf = PinConf {
        port:     Port2,
        pin:      5,
        function: AltFunction1,
      };
      pub static DTR1: PinConf = PinConf {
        port:     Port2,
        pin:      5,
        function: AltFunction2,
      };
    }
    pub mod pin6 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      6,
        function: super::super::super::GPIO,
      };
      pub static PCAP1_0: PinConf = PinConf {
        port:     Port2,
        pin:      6,
        function: AltFunction1,
      };
      pub static RI1: PinConf = PinConf {
        port:     Port2,
        pin:      6,
        function: AltFunction2,
      };
    }
    pub mod pin7 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      7,
        function: super::super::super::GPIO,
      };
      pub static RD2: PinConf = PinConf {
        port:     Port2,
        pin:      7,
        function: AltFunction1,
      };
      pub static RTS1: PinConf = PinConf {
        port:     Port2,
        pin:      7,
        function: AltFunction2,
      };
    }
    pub mod pin8 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      8,
        function: super::super::super::GPIO,
      };
      pub static TD2: PinConf = PinConf {
        port:     Port2,
        pin:      8,
        function: AltFunction1,
      };
      pub static TXD2: PinConf = PinConf {
        port:     Port2,
        pin:      8,
        function: AltFunction2,
      };
      pub static ENET_MDC: PinConf = PinConf {
        port:     Port2,
        pin:      8,
        function: AltFunction3,
      };
    }
    pub mod pin9 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      9,
        function: super::super::super::GPIO,
      };
      pub static USB_CONNECT: PinConf = PinConf {
        port:     Port2,
        pin:      9,
        function: AltFunction1,
      };
      pub static RXD2: PinConf = PinConf {
        port:     Port2,
        pin:      9,
        function: AltFunction2,
      };
      pub static ENET_MDIO: PinConf = PinConf {
        port:     Port2,
        pin:      9,
        function: AltFunction3,
      };
    }
    pub mod pin10 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      10,
        function: super::super::super::GPIO,
      };
      pub static EINT0: PinConf = PinConf {
        port:     Port2,
        pin:      10,
        function: AltFunction1,
      };
      pub static NMI: PinConf = PinConf {
        port:     Port2,
        pin:      10,
        function: AltFunction2,
      };
    }
    pub mod pin11 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      11,
        function: super::super::super::GPIO,
      };
      pub static EINT1: PinConf = PinConf {
        port:     Port2,
        pin:      11,
        function: AltFunction1,
      };
      pub static I2STX_CLK: PinConf = PinConf {
        port:     Port2,
        pin:      11,
        function: AltFunction3,
      };
    }
    pub mod pin12 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      12,
        function: super::super::super::GPIO,
      };
      pub static EINT2: PinConf = PinConf {
        port:     Port2,
        pin:      12,
        function: AltFunction1,
      };
      pub static I2STX_WS: PinConf = PinConf {
        port:     Port2,
        pin:      12,
        function: AltFunction3,
      };
    }
    pub mod pin13 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port2,
        pin:      13,
        function: super::super::super::GPIO,
      };
      pub static EINT3: PinConf = PinConf {
        port:     Port2,
        pin:      13,
        function: AltFunction1,
      };
      pub static I2STX_SDA: PinConf = PinConf {
        port:     Port2,
        pin:      13,
        function: AltFunction3,
      };
    }
}
pub mod port3 {
    pub mod pin25 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port3,
        pin:      25,
        function: super::super::super::GPIO,
      };
      pub static MAT0_0: PinConf = PinConf {
        port:     Port3,
        pin:      25,
        function: AltFunction2,
      };
      pub static PWM1_2: PinConf = PinConf {
        port:     Port3,
        pin:      25,
        function: AltFunction3,
      };
    }
    pub mod pin26 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port3,
        pin:      26,
        function: super::super::super::GPIO,
      };
      pub static STCLK: PinConf = PinConf {
        port:     Port3,
        pin:      26,
        function: AltFunction1,
      };
      pub static MAT0_1: PinConf = PinConf {
        port:     Port3,
        pin:      26,
        function: AltFunction2,
      };
      pub static PWM1_3: PinConf = PinConf {
        port:     Port3,
        pin:      26,
        function: AltFunction3,
      };
    }
}
pub mod port4 {
    pub mod pin28 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port4,
        pin:      28,
        function: super::super::super::GPIO,
      };
      pub static RX_MCLK: PinConf = PinConf {
        port:     Port4,
        pin:      28,
        function: AltFunction1,
      };
      pub static MAT2_0: PinConf = PinConf {
        port:     Port4,
        pin:      28,
        function: AltFunction2,
      };
      pub static TXD3: PinConf = PinConf {
        port:     Port4,
        pin:      28,
        function: AltFunction3,
      };
    }
    pub mod pin29 {
      pub use super::super::super::{PinConf, Port0, Port1, Port2, Port3, Port4, AltFunction1, AltFunction2, AltFunction3};

      pub static GPIO: PinConf = PinConf {
        port:     Port4,
        pin:      29,
        function: super::super::super::GPIO,
      };
      pub static TX_MCLK: PinConf = PinConf {
        port:     Port4,
        pin:      29,
        function: AltFunction1,
      };
      pub static MAT2_1: PinConf = PinConf {
        port:     Port4,
        pin:      29,
        function: AltFunction2,
      };
      pub static RXD3: PinConf = PinConf {
        port:     Port4,
        pin:      29,
        function: AltFunction3,
      };
    }
}
