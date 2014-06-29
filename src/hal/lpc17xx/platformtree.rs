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

use std::rc::Rc;
use syntax::ext::base::ExtCtxt;

use builder::Builder;
use node;

mod system_clock_pt;
mod timer_pt;
mod pin_pt;
mod uart_pt;

mod pinmap;

pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
    node: Rc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  node.get_by_path("clock").and_then(|sub| -> Option<bool> {
    system_clock_pt::build_clock(builder, cx, sub);
    None
  });

  node.get_by_path("timer").and_then(|sub| -> Option<bool> {
    timer_pt::build_timer(builder, cx, sub);
    None
  });

  node.get_by_path("uart").and_then(|sub| -> Option<bool> {
    uart_pt::build_uart(builder, cx, sub);
    None
  });

  node.get_by_path("gpio").and_then(|sub| -> Option<bool> {
    pin_pt::build_pin(builder, cx, sub);
    None
  });
}

#[cfg(test)]
mod test {
  use test_helpers::fails_to_build;

  #[test]
  fn fails_to_parse_garbage_attrs() {
    fails_to_build("lpc17xx@mcu { key = 1; }");
  }
}
