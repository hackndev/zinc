require 'pry'
require 'rugged'
require 'travis'
require 'json'
require 'set'
require 'github_api'

class TravisLoader
  def load
    builds = JSON.parse(File.open("_data/travis.json", "r:utf-8").read)
    repo = Travis::Repository.find('hackndev/zinc')
    repo.recent_builds.each do |b|
      c = collect_build(b, builds)
      unless c.kind_of?(Integer)
        puts "#{c['number']} #{c['commit']['branch']}/#{c['commit']['sha'][0..8]} #{c['commit']['message'].split("\n").first}"
        builds[c['number']] = c
      else
        puts "early break at #{c}"
        build_prev = builds[(c-1).to_s]
        puts "#{c-1} doesn't exist!" unless build_prev
        break
      end
    end
    open('_data/travis.json', 'w') { |f| f.write(JSON.generate(builds)) }
  end

  def sanitized_log(job)
    unless job.log.body.kind_of?(String)
      binding.pry
    end
    job.log.body.gsub(/[^[:print:]\e\n]/, '').gsub(/\e[^m]+m/, '').split("\n")
  end

  def collect_stats(log)
    stats = {}
    log.length.times do |i|
      line = log[i]
      if line.start_with?('Statistics for')
        app = line.gsub('Statistics for ', '').gsub('app_', '').gsub('.elf', '')
        log[i+1] =~ /(\d+)/
        sz_text = $1.to_i
        log[i+2] =~ /(\d+)/
        sz_data = $1.to_i
        log[i+3] =~ /(\d+)/
        sz_bss  = $1.to_i

        stats[app] = {
          'text'  => sz_text,
          'data'  => sz_data,
          'bss'   => sz_bss,
          'total' => sz_text + sz_data + sz_bss,
        }
      end
    end

    stats
  end

  def lpc_build_job(build)
    build.jobs.detect{|j| (j.config['env'] =~ /stm/)==nil}
  end

  def collect_build(build, old)
    h = build.to_h
    h['build_id'] = build.id
    h['commit'] = build.commit.to_h

    return h['number'].to_i if old[h['number']]

    h.delete('config')
    h['jobs'] = build.jobs.map do |j|
      jh = j.to_h
      jh['env'] = jh['config']['env']
      jh.delete('config')
      jh['stats'] = collect_stats(sanitized_log(j))
      jh
    end

    h
  end
end
