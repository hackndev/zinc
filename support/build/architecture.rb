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

require 'yaml'

class Architecture
  attr_reader :name

  # Arch name as defined by gcc library path
  attr_reader :arch

  # CPU name, passed to -mcpu
  attr_reader :cpu

  # Target triple
  attr_reader :target

  def self.from_yaml(filename)
    y = YAML.load_file(filename)
    y.merge(y) do |k, v|
      Architecture.new(k, v['arch'], v['cpu'], v['target'])
    end
  end

  def initialize(name, arch, cpu, target)
    @name = name
    @arch = arch
    @cpu = cpu
    @target = target
  end
end
