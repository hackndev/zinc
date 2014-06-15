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

template :assert_pt_main_source_equals do
  test :generates_clock_for_lpc11xx, {}, {
    src: '
      mcu@mcu::lpc17xx {
        @clock {
          source = "main-oscillator";
          source_frequency = 12_000_000;
          pll_m = 50;
          pll_n = 3;
          pll_divisor = 4;
        }
      }
    ',
    out: '
      {{
        use zinc::hal::lpc17xx::init;
        init::init_clock(
            init::Clock {
              source: init::Main(Some(12000000)),
              pll: init::PLL0 {
                enabled: true,
                m: 50u,
                n: 3u,
                divisor: 4u,
              },
            }
        );
      };}
    '.gsub(/(\n|\s)*/, '')
  }
end