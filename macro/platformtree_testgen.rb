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

template :assert_pt_compiles do
  test :doesnt_parse_empty_pt, {should_fail: true}, {
    src: ''
  }

  test :doesnt_parse_node_with_no_body, {should_fail: true}, {
    src: 'node@root'
  }

  test :doesnt_parse_node_with_no_path, {should_fail: true}, {
    src: 'node@ {}'
  }

  test :doesnt_parse_node_with_broken_path, {should_fail: true}, {
    src: 'node@::root::::blah {}'
  }

  test :doesnt_parse_trailing_garbage, {should_fail: true}, {
    src: 'node@root {} node@root {}'
  }
end
