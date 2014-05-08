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

require 'set'

module Deps
  def self.collect_dep_srcs(src, root)
    dep_files = submodules(src, root)
    return Set.new if dep_files.empty?

    collected_deps = dep_files.dup

    dep_files.each do |f|
      collected_deps += collect_dep_srcs(f, src)
    end

    collected_deps
  end

  def self.submodules(src, root=nil)
    subs = Set.new
    unless File.exists?(src)
      raise RuntimeError.new("Cannot find #{src} included from #{root}")
    end
    lines = open(src).read.split("\n")
    mod_rx = /^\s*(?:#\[.+\]\s*)*(?:pub)?\s*mod\s+(\w+)\s*;/
    path_rx = /^\s*#\[path\s*=\s*"([^"]+)"\]/
    mod_path_rx = /^\s*#\[path\s*=\s*"([^"]+)"\]\s+(?:pub)?\s*mod\s+\w+\s*;/
    prev = ''
    lines.each do |l|
      mp = mod_path_rx.match(l)
      if mp
        subs << File.join(File.dirname(src), mp[1]) if mp
      else
        m = mod_rx.match(l)
        p = path_rx.match(prev)

        if m
          if p
            subs << File.join(File.dirname(src), p[1])
          else
            subs << mod_to_src(src, m[1])
          end
        end
      end
      prev = l
    end
    subs
  end

  def self.mod_to_src(src, mod)
    fn1 = File.join(File.dirname(src), mod + '.rs')
    return fn1 if File.exists?(fn1)
    fn2 = File.join(File.dirname(src), mod, 'mod.rs')
    return fn2 if File.exists?(fn2)
    raise ArgumentError.new("Cannot resolve mod #{mod} in scope of #{src}, tried #{fn1} and #{fn2}")
  end
end
