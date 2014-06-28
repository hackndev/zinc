require 'pry'
require 'rugged'
require 'travis'
require 'json'
require 'set'
require 'github_api'
load 'travis.rb'

module GitHist
  class Generator < Jekyll::Generator
    def builds
      @builds ||= begin
        TravisLoader.new.load
        JSON.parse(File.open("_data/travis.json", "r:utf-8").read)
      end
    end

    def builds_by_commit(sha, branch)
      builds.map do |_, b|
        if b['commit']['sha'] == sha && b['commit']['branch'] == branch
          b
        else
          nil
        end
      end.compact
    end

    def build_by_id(bid)
      b = builds.detect { |_, b| b['build_id'] == bid }
      if b == nil || b.empty?
        puts "!!! cannot find build by id #{bid}"
        nil
      else
        b.last
      end
    end

    def latest_build_for_commit(sha)
      builds = builds_by_commit(sha, 'master').sort { |a,b| a['number'].to_i <=> b['number'].to_i }
      return nil if builds.empty?
      builds.last
    end

    def generate(site)
      page = site.pages.detect {|page| page.data['git_history']}

      repo_path = ENV['ZINC_REPO'] || '.'
      repo = Rugged::Repository.new(repo_path)
      master = repo.refs.detect {|r| r.name == 'refs/heads/master'}

      walker = Rugged::Walker.new(repo)
      walker.sorting(Rugged::SORT_TOPO)
      walker.push(master.target)
      commits = walker.each.to_a
      puts "#{commits.length} commits"

      github = Github.new(auto_pagination: true)

      apps = Set.new
      closed_prs = github.pull_requests.list('hackndev', 'zinc', state: 'closed')

      puts "** history **"
      page.data['commits'] = commits.map do |ci|
        build = latest_build_for_commit(ci.oid)
        if build
          message = ci.message.split("\n").first
          if message =~ /Merge pull request #(\d+) from ([\w_\-\/]+)/
            pr_number = $1.to_i
            pr_branch = $2
            pr = closed_prs.detect {|p| p.number == pr_number}
            message = "<span class=\"octicon octicon-git-pull-request\"></span>
            <a href=\"https://github.com/hackndev/zinc/pull/#{pr_number}\">
              PR #{pr_number}
            </a> " + (pr ? pr.title : "(#{pr_branch})")
          end

          lpc_job = build['jobs'].detect{|j| (j['env'] =~ /stm/)==nil}
          stats = lpc_job['stats']
          stats.keys.each { |k| apps << k }

          puts "#{ci.oid[0..8]} - #{ci.message.split("\n").first}"
          {
            'oid' => ci.oid[0..8],
            'message' => message,
            'stats' => stats,
            'state' => lpc_job['state'],
          }
        else
          puts "#{ci.oid[0..8]}!  no builds for commit"
          nil
        end
      end.compact

      page.data['apps'] = (apps - ['zinc']).to_a

      pr_apps = Set.new
      prs = github.pull_requests.list('hackndev', 'zinc', state: 'open')
      page.data['pulls'] = prs.map do |pr|
        statuses = github.repos.statuses.all('hackndev', 'zinc', pr['head']['sha'])
            .sort{ |a,b| a['created_at'] <=> b['created_at'] }
        if statuses.empty?
          build = nil
        elsif statuses.last['context'] == 'continuous-integration/travis-ci'
          build_id = statuses.last['target_url'].gsub(/.+\//,'').to_i
          build = build_by_id(build_id)
        else
          puts "Unknown status #{statuses.last} num=#{pr['number']} sha=#{pr['head']['sha']}"
          build = nil
        end

        if build
          lpc_job = build['jobs'].detect{|j| (j['env'] =~ /stm/)==nil}
          stats = lpc_job['stats']
          stats.keys.each { |k| pr_apps << k }
          title = "<span class=\"octicon octicon-git-pull-request\"></span>
            <a href=\"https://github.com/hackndev/zinc/pull/#{pr.number}\">
              PR #{pr.number}
            </a> #{pr.title}"
          {
            'author' => pr.user.login,
            'title'  => title,
            'stats'  => stats,
            'state'  => lpc_job['state'],
          }
        else
          nil
        end
      end.compact

      page.data['pr_apps'] = pr_apps.to_a
    end
  end
end

module Jekyll
  module GitFilter
    def build_size(ci, app)
      a = ci['stats'][app]
      if a
        a['total']
      else
        ''
      end
    end
  end
end

Liquid::Template.register_filter(Jekyll::GitFilter)
