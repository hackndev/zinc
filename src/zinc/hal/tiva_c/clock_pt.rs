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

use std::rc::Rc;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;

use builder::{Builder, TokenString};
use node;

pub fn attach(_: &mut Builder, _: &mut ExtCtxt, node: Rc<node::Node>) {
  node.materializer.set(Some(build_clock as fn(&mut Builder, &mut ExtCtxt, Rc<node::Node>)));
}

fn build_clock(builder: &mut Builder,
               cx:      &mut ExtCtxt,
               node:    Rc<node::Node>) {
  if !node.expect_attributes(cx, &[("source", node::StrAttribute)]) {
    return;
  }

  let source = TokenString(node.get_string_attr("source").unwrap());

  let use_pll = node.get_bool_attr("pll").unwrap_or(false);
  let div     = node.get_int_attr("div").unwrap_or(1);
  let xtal    = TokenString(node.get_string_attr("xtal")
                            .unwrap_or("X16_0MHz".to_string()));

  let ex = quote_expr!(&*cx,
      {
        use zinc::hal::tiva_c::sysctl::clock;
        use core::option::Option::Some;

        clock::sysclk_configure(
          zinc::hal::tiva_c::sysctl::clock::ClockSource::$source,
          Some(zinc::hal::tiva_c::sysctl::clock::MOSCFreq::$xtal),
          $use_pll,
          Some($div));
      }
  );
  builder.add_main_statement(cx.stmt_expr(ex));
}
