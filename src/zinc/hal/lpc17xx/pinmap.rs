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

//! Automatically generated file, do not edit
//! Update the definition in pinmap.rs.rb and re-generate pinmap.rs with
//! support/pingen.rb <src> <dst>
//!
//! This module provides all possible pin configurations for LPC17xx.
use std::collections::HashMap;

pub type PinDef = HashMap<String, usize>;
pub type PinsDef = Vec<Option<PinDef>>;

pub fn port_def() -> HashMap<String, PinsDef> {
  let mut h = HashMap::new();

  {
    let mut pins = Vec::new();
    {
      let mut pin = HashMap::new();
      pin.insert("rd1".to_string(), 1);
      pin.insert("txd3".to_string(), 2);
      pin.insert("sda1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("td1".to_string(), 1);
      pin.insert("rxd3".to_string(), 2);
      pin.insert("scl1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("txd0".to_string(), 1);
      pin.insert("ad0_7".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("rxd0".to_string(), 1);
      pin.insert("ad0_6".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("i2srx_clk".to_string(), 1);
      pin.insert("rd2".to_string(), 2);
      pin.insert("cap2_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("i2srx_ws".to_string(), 1);
      pin.insert("td2".to_string(), 2);
      pin.insert("cap2_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("i2srx_sda".to_string(), 1);
      pin.insert("ssel1".to_string(), 2);
      pin.insert("mat2_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("i2stx_clk".to_string(), 1);
      pin.insert("sck1".to_string(), 2);
      pin.insert("mat2_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("i2stx_ws".to_string(), 1);
      pin.insert("miso1".to_string(), 2);
      pin.insert("mat2_2".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("i2stx_sda".to_string(), 1);
      pin.insert("mosi1".to_string(), 2);
      pin.insert("mat2_3".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("txd2".to_string(), 1);
      pin.insert("sda2".to_string(), 2);
      pin.insert("mat3_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("rxd2".to_string(), 1);
      pin.insert("scl2".to_string(), 2);
      pin.insert("mat3_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      let mut pin = HashMap::new();
      pin.insert("txd1".to_string(), 1);
      pin.insert("sck0".to_string(), 2);
      pin.insert("sck".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("rxd1".to_string(), 1);
      pin.insert("ssel0".to_string(), 2);
      pin.insert("ssel".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("cts1".to_string(), 1);
      pin.insert("miso0".to_string(), 2);
      pin.insert("miso".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("dcd1".to_string(), 1);
      pin.insert("mosi0".to_string(), 2);
      pin.insert("mosi".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("dsr1".to_string(), 1);
      pin.insert("sda1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("dtr1".to_string(), 1);
      pin.insert("scl1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("ri1".to_string(), 1);
      pin.insert("rd1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("rts1".to_string(), 1);
      pin.insert("td1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("ad0_0".to_string(), 1);
      pin.insert("i2srx_clk".to_string(), 2);
      pin.insert("cap3_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("ad0_1".to_string(), 1);
      pin.insert("i2srx_ws".to_string(), 2);
      pin.insert("cap3_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("ad0_2".to_string(), 1);
      pin.insert("i2srx_sda".to_string(), 2);
      pin.insert("txd3".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("ad0_3".to_string(), 1);
      pin.insert("aout".to_string(), 2);
      pin.insert("txd3".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("sda0".to_string(), 1);
      pin.insert("usb_sda".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("scl0".to_string(), 1);
      pin.insert("usb_scl".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("usb_d_pos".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("usb_d_neg".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      pins.push(None);
    }
    h.insert("0".to_string(), pins);
  }
  {
    let mut pins = Vec::new();
    {
      let mut pin = HashMap::new();
      pin.insert("enet_txd0".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_txd1".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_tx_en".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_crs".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_rxd0".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_rxd1".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_rx_er".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_ref_clck".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_mdc".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("enet_mdio".to_string(), 1);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("usb_up_led".to_string(), 1);
      pin.insert("pwm1_1".to_string(), 2);
      pin.insert("cap1_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcoa0".to_string(), 1);
      pin.insert("usb_ppwr".to_string(), 2);
      pin.insert("cap1_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mci0".to_string(), 1);
      pin.insert("pwm1_2".to_string(), 2);
      pin.insert("sck0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcabort".to_string(), 1);
      pin.insert("pwm1_3".to_string(), 2);
      pin.insert("ssel0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcob0".to_string(), 1);
      pin.insert("usb_pwrd".to_string(), 2);
      pin.insert("mat1_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mci1".to_string(), 1);
      pin.insert("pwm1_4".to_string(), 2);
      pin.insert("miso0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mci2".to_string(), 1);
      pin.insert("pwm1_5".to_string(), 2);
      pin.insert("mosi0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcoa1".to_string(), 1);
      pin.insert("mat1_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcob1".to_string(), 1);
      pin.insert("pwm1_6".to_string(), 2);
      pin.insert("cap0_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("clkout".to_string(), 1);
      pin.insert("usb_ovrcr".to_string(), 2);
      pin.insert("cap0_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcoa2".to_string(), 1);
      pin.insert("pcap1_0".to_string(), 2);
      pin.insert("mat0_0".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mcob2".to_string(), 1);
      pin.insert("pcap1_1".to_string(), 2);
      pin.insert("mat0_1".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("vbus".to_string(), 2);
      pin.insert("ad0_4".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("sck1".to_string(), 2);
      pin.insert("ad0_5".to_string(), 3);
      pins.push(Some(pin));
    }
    h.insert("1".to_string(), pins);
  }
  {
    let mut pins = Vec::new();
    {
      let mut pin = HashMap::new();
      pin.insert("pwm1_1".to_string(), 1);
      pin.insert("txd1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("pwm1_2".to_string(), 1);
      pin.insert("rxd1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("pwm1_3".to_string(), 1);
      pin.insert("cts1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("pwm1_4".to_string(), 1);
      pin.insert("dcd1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("pwm1_5".to_string(), 1);
      pin.insert("dsr1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("pwm1_6".to_string(), 1);
      pin.insert("dtr1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("pcap1_0".to_string(), 1);
      pin.insert("ri1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("rd2".to_string(), 1);
      pin.insert("rts1".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("td2".to_string(), 1);
      pin.insert("txd2".to_string(), 2);
      pin.insert("enet_mdc".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("usb_connect".to_string(), 1);
      pin.insert("rxd2".to_string(), 2);
      pin.insert("enet_mdio".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("eint0".to_string(), 1);
      pin.insert("nmi".to_string(), 2);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("eint1".to_string(), 1);
      pin.insert("i2stx_clk".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("eint2".to_string(), 1);
      pin.insert("i2stx_ws".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("eint3".to_string(), 1);
      pin.insert("i2stx_sda".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    h.insert("2".to_string(), pins);
  }
  {
    let mut pins = Vec::new();
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      let mut pin = HashMap::new();
      pin.insert("mat0_0".to_string(), 2);
      pin.insert("pwm1_2".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("stclk".to_string(), 1);
      pin.insert("mat0_1".to_string(), 2);
      pin.insert("pwm1_3".to_string(), 3);
      pins.push(Some(pin));
    }
    h.insert("3".to_string(), pins);
  }
  {
    let mut pins = Vec::new();
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      pins.push(None);
    }
    {
      let mut pin = HashMap::new();
      pin.insert("rx_mclk".to_string(), 1);
      pin.insert("mat2_0".to_string(), 2);
      pin.insert("txd3".to_string(), 3);
      pins.push(Some(pin));
    }
    {
      let mut pin = HashMap::new();
      pin.insert("tx_mclk".to_string(), 1);
      pin.insert("mat2_1".to_string(), 2);
      pin.insert("rxd3".to_string(), 3);
      pins.push(Some(pin));
    }
    h.insert("4".to_string(), pins);
  }

  h
}
