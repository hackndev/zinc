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

#[cfg(all(not(test), not(feature = "test")))]
use core::fmt::Arguments;

#[cfg(all(not(test), not(feature = "test")))]
#[lang="stack_exhausted"]
extern fn stack_exhausted() {}

#[cfg(all(not(test), not(feature = "test")))]
#[lang="eh_personality"]
extern fn eh_personality() {}

#[cfg(all(not(test), not(feature = "test")))]
#[lang="begin_unwind"]
extern fn begin_unwind() {}

#[cfg(all(not(test), not(feature = "test")))]
#[lang="panic_fmt"]
pub fn panic_fmt(_fmt: &Arguments, _file_line: &(&'static str, usize)) -> ! {
  loop { }
}
