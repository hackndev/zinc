# Zinc, the bare metal stack for rust.
# Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

class TestLoader
  def initialize(f)
    @src = open(f).read
    @f = f
  end

  def load
    li = LoaderInstance.new
    li.instance_eval(@src, @f, 1)
    li.tests
  end
end

class LoaderInstance
  attr_reader :tests

  def initialize
    @tests = {}
  end

  def test(name, h, &block)
    tpl = TEMPLATE.
        gsub('TEST_NAME', name.to_s).
        gsub('TEST_CODE', block.call()).
        gsub('MOD_PT_PATH', "platformtree/pt.rs".in_root)
    @tests[name] = {
      conditions: h,
      source: tpl,
    }
  end
end

TEMPLATE = <<EOF
#![feature(phase)]
#![allow(unused_mut,dead_code)]

#[phase(syntax)] extern crate platformtree_macro;

use std::collections::hashmap;

#[path="MOD_PT_PATH"] mod pt;

#[test]
fn TEST_NAME() {
  let p = platformtree_parse!(
    TEST_CODE
  );

  assert!(true);
}
EOF
