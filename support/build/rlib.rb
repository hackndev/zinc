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

require 'digest'

module Rlib
  def self.crate_name(src)
    crate = File.basename(src, File.extname(src))
    version = '0.0'

    id_regex = /#!\[crate_id.*=.*"([a-zA-Z0-9_]+)(?:#([a-zA-Z0-9_.\-]+))?"\]/
    lines = open(src).read.split("\n")
    lines.each do |l|
      m = id_regex.match(l)
      if m
        crate = m[1]
        version = m[2] ? m[2] : '0.0'
        break
      end
    end

    digest = Digest::SHA256.hexdigest(crate + '-' + version)[0...8]
    "lib#{crate}-#{digest}-#{version}.rlib"
  end
end
