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

class TrackingTask < Rake::Task
  attr_accessor :track_file, :track_expected

  def self.define_task(file, expected)
    t = super file do |t|
      t.save!
    end

    t.track_file = file
    t.track_expected = expected.to_s

    t
  end

  def needed?
    if File.exist?(track_file)
      track_expected != open(track_file).read.strip
    else
      true
    end
  end

  def save!
    open(track_file, 'w') do |f|
      f.write(track_expected)
    end
  end

  def timestamp
    File.exist?(track_file) ? File.mtime(track_file) : Rake::EARLY
  end
end
