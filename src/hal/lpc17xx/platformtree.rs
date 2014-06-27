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

use std::gc::Gc;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;

use builder::{Builder, TokenString};
use node;

pub fn build_mcu(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  node.get_by_path("clock").and_then(|sub| -> Option<bool> {
    build_clock(builder, cx, sub);
    None
  });

  node.get_by_path("timer").and_then(|sub| -> Option<bool> {
    build_timer(builder, cx, sub);
    None
  });

  node.get_by_path("gpio").and_then(|sub| -> Option<bool> {
    build_gpio(builder, cx, sub);
    None
  });
}

pub fn build_clock(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_attributes(cx, [("source", node::StringAttribute)]) {
    return;
  }

  let source = node.get_string_attr("source").unwrap();
  let clock_source = TokenString::new(match source.as_slice() {
    "internal-oscillator" => "init::Internal".to_str(),
    "rtc-oscillator"      => "init::RTC".to_str(),
    "main-oscillator"     => {
      let some_source_frequency =
          node.get_required_int_attr(cx, "source_frequency");
      if some_source_frequency == None {
        "BAD".to_str()
      } else {
        format!("init::Main({})", some_source_frequency.unwrap())
      }
    },
    other => {
      cx.span_err(
          node.get_attr("source").value_span,
          format!("unknown oscillator value `{}`", other).as_slice());
      "BAD".to_str()
    },
  });

  let some_pll_conf = node.get_by_path("pll").and_then(|sub|
      -> Option<(uint, uint, uint)> {
    if !sub.expect_no_subnodes(cx) || !sub.expect_attributes(cx, [
        ("m", node::IntAttribute),
        ("n", node::IntAttribute),
        ("divisor", node::IntAttribute)]) {
      None
    } else {
      let m = sub.get_int_attr("m").unwrap();
      let n = sub.get_int_attr("n").unwrap();
      let divisor = sub.get_int_attr("divisor").unwrap();
      Some((m, n, divisor))
    }
  });
  if some_pll_conf.is_none() {
    cx.parse_sess().span_diagnostic.span_err(node.name_span,
        "required subnode `pll` is missing");
    return;
  }

  let (m, n, divisor) = some_pll_conf.unwrap();
  let pll_m: u8 = m as u8;
  let pll_n: u8 = n as u8;
  let pll_divisor: u8 = divisor as u8;

  let ex = quote_expr!(&*cx,
      {
        use zinc::hal::lpc17xx::init;
        init::init_clock(
            &init::Clock {
              source: $clock_source,
              pll: init::PLL0 {
                enabled: true,
                m: $pll_m,
                n: $pll_n,
                divisor: $pll_divisor,
              }
            }
        );
      }
  );
  builder.add_main_statement(cx.stmt_expr(ex));
}

pub fn build_timer(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) {
    return;
  }

  for (path, sub) in node.subnodes.iter() {
    if !sub.expect_attributes(cx, [
        ("counter", node::IntAttribute),
        ("divisor", node::IntAttribute)]) {
      continue;
    }

    if sub.name.is_none() {
      cx.parse_sess().span_diagnostic.span_err(sub.name_span,
          "timer node must have a name");
      continue;
    }

    let name = TokenString::new(sub.name.clone().unwrap());
    let timer_index: uint = from_str(path.as_slice()).unwrap();
    let counter: u32 = sub.get_int_attr("counter").unwrap() as u32;
    let divisor: u8 = sub.get_int_attr("divisor").unwrap() as u8;

    let timer_name = match timer_index {
      0..3 => TokenString::new(format!("timer::Timer{}", timer_index)),
      other => {
        cx.parse_sess().span_diagnostic.span_err(sub.path_span,
            format!("unknown timer index `{}`, allowed indexes: 0, 1, 2, 3",
                other).as_slice());
        continue;
      }
    };

    sub.type_name.set(Some("zinc::hal::lpc17xx::timer::Timer"));

    let st = quote_stmt!(&*cx,
        let $name = {
          use zinc::hal::lpc17xx::timer;
          let conf = timer::TimerConf {
            timer: $timer_name,
            counter: $counter,
            divisor: $divisor,
          };
          conf.setup()
        }
    );
    builder.add_main_statement(st);
  }
}

pub fn build_gpio(builder: &mut Builder, cx: &mut ExtCtxt,
    node: &Gc<node::Node>) {
  if !node.expect_no_attributes(cx) { return }

  for (port_path, port_node) in node.subnodes.iter() {
    if !port_node.expect_no_attributes(cx) { continue }

    let port_str = format!("pin::Port{}", match from_str::<uint>(port_path.as_slice()).unwrap() {
      0..4 => port_path,
      other => {
        cx.parse_sess().span_diagnostic.span_err(port_node.path_span,
            format!("unknown port `{}`, allowed values: 0..4",
                other).as_slice());
        continue;
      }
    });
    let port = TokenString::new(port_str);

    for (pin_path, pin_node) in port_node.subnodes.iter() {
      if pin_node.name.is_none() {
        cx.parse_sess().span_diagnostic.span_err(pin_node.name_span,
            "pin node must have a name");
        continue;
      }

      let direction_str = match pin_node.get_string_attr("direction")
          .unwrap().as_slice() {
        "out" => "hal::gpio::Out",
        "in"  => "hal::gpio::In",
        other => {
          let attr = pin_node.get_attr("direction");
          cx.parse_sess().span_diagnostic.span_err(attr.value_span,
              format!("unknown direction `{}`, allowed values: `in`, `out`",
                  other).as_slice());
          continue;
        }
      };
      let direction = TokenString::new(direction_str.to_str());

      let pin_str = match from_str::<uint>(pin_path.as_slice()).unwrap() {
        0..31 => pin_path,
        other => {
          cx.parse_sess().span_diagnostic.span_err(pin_node.path_span,
              format!("unknown pin `{}`, allowed values: 0..31",
                  other).as_slice());
          continue;
        }
      };
      let pin = TokenString::new(format!("{}u8", pin_str));
      let pin_name = TokenString::new(pin_node.name.clone().unwrap());
      let pin_name_conf = TokenString::new(format!(
          "{}_conf", pin_node.name.clone().unwrap()));

      pin_node.type_name.set(Some("zinc::hal::lpc17xx::gpio::GPIO"));

      let st_conf = quote_stmt!(&*cx,
          let $pin_name_conf = {
            use zinc::hal;
            use zinc::hal::lpc17xx::{pin, gpio};
            let conf = gpio::GPIOConf {
              pin: pin::PinConf {
                port: $port,
                pin: $pin,
                function: pin::GPIO,
              },
              direction: $direction,
            };
            conf.pin.setup();
            conf
          }
      );
      let st = quote_stmt!(&*cx, let $pin_name = $pin_name_conf.setup() );
      builder.add_main_statement(st_conf);
      builder.add_main_statement(st);
    }
  }
}
