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

class String
  def in_build; Context.instance.build_dir(self); end
  def in_intermediate(*a); Context.instance.intermediate_dir(*a, self); end
  def in_root; Context.instance.root_dir(self); end
  def in_source; Context.instance.src_dir(self); end
  def in_platform; Context.instance.platform_dir(self); end
  def as_rlib; Context.instance.rlib_name(self); end
end

class Hash
  def resolve_deps!
    return unless self[:deps]
    self[:deps] = [self[:deps]].flatten

    self[:deps] = self[:deps].map do |d|
      if d.kind_of?(Symbol)
        raise RuntimeError.new("Missing rule #{d}") unless Context.instance.rules[d]
        Context.instance.rules[d][:produce]
      else
        d
      end
    end
    self
  end
end

class Symbol
  def in_env; Context.instance.env[self]; end
  def in_toolchain; "#{Context.instance.env[:toolchain]}-#{self}"; end
end
