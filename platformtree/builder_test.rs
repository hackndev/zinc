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

use test_helpers::fails_to_build;

#[test]
fn fails_to_parse_pt_with_unknown_root_node() {
  fails_to_build("unknown@node {}");
}

#[test]
fn fails_to_parse_pt_with_unknown_mcu() {
  fails_to_build("mcu@bad {}");
}
