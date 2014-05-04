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

/*!
RTOS support code.

This module is a higher level abstraction over hardware than hal. It might be
incompatible direct hal usage in some cases.
*/

pub mod debug;
pub mod syscall;
#[cfg(cfg_multitasking)] pub mod task;
#[cfg(cfg_multitasking)] pub mod mutex;
