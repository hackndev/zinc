require 'digest'
require 'set'

module Context
  class << self
    attr_reader :app_name, :app
  end
  # def self.app_name
  #   @app_name
  # end

  # def self.app
  #   @app
  # end

  def self.src_dir(*args)
    File.join(root_dir, 'src', *args)
  end

  def self.build_dir(*args)
    File.join(root_dir, 'build', *args)
  end

  def self.intermediate_dir(*args)
    File.join(root_dir, 'build', 'intermediate', *args)
  end

  def self.platform_dir(*args)
    src_dir('hal', @current_platform.to_s, *args)
  end

  def self.prepare!(rsflags, ldflags, platforms, architectures)
    platform_file = File.join(build_dir, ".platform")

    current_platform_str = ENV['PLATFORM'] or raise ArgumentError.new("Undefined platform, available platforms: #{platforms.keys.join(', ')}")
    @current_platform = current_platform_str.to_sym
    previous_platform = File.exists?(platform_file) ?
      open(platform_file).read.to_sym : nil

    if previous_platform && (@current_platform != previous_platform)
      FileUtils.rm_rf(build_dir)
    end

    FileUtils.mkdir(build_dir) unless Dir.exists?(build_dir)
    FileUtils.mkdir(intermediate_dir) unless Dir.exists?(intermediate_dir)
    open(platform_file, 'w') { |f| f.write(@current_platform) }

    platform = platforms[@current_platform] or raise ArgumentError.new("Undefined platform #{@current_platform}, available platforms: #{platforms.keys.join(', ')}")
    arch = architectures[platform[:arch]] or raise ArgumentError.new("Undefined arch #{platform[:arch]} for platform #{@current_platform}")

    rsflags.push("--target #{arch[:target]}",
        "-Ctarget-cpu=#{arch[:cpu]}",
        "--cfg #{platform[:config]}")
        # "-g")
    ldflags.push("-L#{File.join(TOOLCHAIN_LIBS_PATH, arch[:arch])}")

    if ENV['DEBUG']
      rsflags.push('--opt-level 0')
    else
      rsflags.push('--opt-level 2')
    end

    ENV['RSFLAGS'] = rsflags.join(' ')
    ENV['LDFLAGS'] = ldflags.join(' ')

    @app_name = ENV['APP'] or ArgumentError.new("Undefined application")
    app_path = root_dir('apps', @app_name + '.rs') or ArgumentError.new("Application #{@app_name} not found in apps")

    @app = app_path
  end

  def self.root_dir(*args)
    File.join(File.dirname(File.dirname(__FILE__)), *args)
  end

  def self.app_dep
    AppDepTask.define_task(build_dir('.app')) do |t|
      t.store_name
    end
  end
end

class AppDepTask < Rake::Task
  def needed?
    if File.exist?(name)
      build_app_name = open(name).read.strip
      if build_app_name != Context.app_name
        true
      else
        false
      end
    else
      true
    end
  end

  def store_name
    open(name, 'w') do |f|
      f.write(Context.app_name)
    end
  end

  def timestamp
    if File.exist?(name)
      File.mtime(name.to_s)
    else
      Rake::EARLY
    end
  end
end

module Rlib
  def self.name(src)
    get_cached(src)
  end

  private
  def self.get_cached(src)
    @cache ||= {}
    unless @cache[src]
      crate, version = crate_id(src)
      digest = Digest::SHA256.hexdigest(crate + '-' + version)[0...8]
      name = "lib#{crate}-#{digest}-#{version}.rlib"
      @cache[src] = name
    end

    @cache[src]
  end

  def self.crate_id(src)
    crate = File.basename(src, File.extname(src))
    version = '0.0'

    id_regex = /#!\[crate_id.*=.*"([a-zA-Z0-9_]+)(#([a-zA-Z0-9_.]+))?"\]/
    lines = open(src).read.split("\n")
    lines.each do |l|
      m = id_regex.match(l)
      if m
        crate = m[1]
        version = m[2] ? m[2] : '0.0'
        return [crate, version]
      end
    end
    return [crate, version]
  end
end

module Rust
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
    mod_rx = /^\s*(?:pub)?\s*mod\s+(\w+)\s*;/
    path_rx = /^\s*#\[path="([^"]+)"\]/
    mod_path_rx = /^\s*#\[path="([^"]+)"\]\s+(?:pub)?\s*mod\s+\w+\s*;/
    prev = ''
    lines.each do |l|
      m = mod_rx.match(l)
      p = path_rx.match(prev)

      if m
        if p
          subs << File.join(File.dirname(src), p[1])
        else
          subs << mod_to_src(src, m[1])
        end
      else
        mp = mod_path_rx.match(l)
        subs << File.join(File.dirname(src), mp[1]) if mp
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

def report_size(fn)
  Rake::Task.define_task :report_size => fn do |t|
    fn = t.prerequisites.first

    stats = `#{TOOLCHAIN}-size #{fn}`.split("\n").last.split("\t").map {|s|s.strip}
    align = stats[3].length
    puts "Statistics for #{File.basename(fn)}"
    puts "  .text: #{stats[0].rjust(align)} bytes"
    puts "  .data: #{stats[1].rjust(align)} bytes"
    puts "  .bss:  #{stats[2].rjust(align)} bytes"
    puts "         #{'='*(align+6)}"
    puts "  TOTAL: #{stats[3]} bytes (0x#{stats[4]})"
  end
end

def compile_rust(h)
  outflags = h.delete(:out_dir) ? "--out-dir #{Context.build_dir}" :
             "-o #{h.to_a.first.first}"
  llvm_pass = h.delete(:llvm_pass)
  lto = h.delete(:lto)
  lto = true if lto == nil
  optimize = h.delete(:optimize)

  declared_deps = h[h.keys.first].kind_of?(Array) ? h[h.keys.first] : [h[h.keys.first]]
  rust_src = declared_deps.shift
  deps = Rust.collect_dep_srcs(rust_src, '__ROOT__')
  h[h.keys.first] = [rust_src] + declared_deps + deps.to_a

  Rake::FileTask.define_task(h) do |t|
    do_lto = lto && t.name.end_with?('.o')
    emit = case File.extname(t.name)
      when '.o'
        '--emit obj'
      when '.ll'
        '--emit ir'
      when '.s'
        '--emit asm'
      else
        ''
    end

    codegen = llvm_pass ? "-C passes=#{llvm_pass}" : ''

    flags = ENV['RSFLAGS']
    if optimize
      flags.gsub!(/--opt-level \d/, "--opt-level #{optimize}")
    end

    sh "#{RUSTC} #{flags} " +
       "#{do_lto ? '-Z lto' : ''} #{emit} -L #{Context.build_dir} #{codegen} " +
       "#{outflags} #{rust_src}"
  end
end

def link_binary(h)
  script = h.delete(:script)
  h[h.keys.first] << script

  Rake::FileTask.define_task(h) do |t|
    t.prerequisites.delete(script)
    mapfn = Context.build_dir(File.basename(t.name, File.extname(t.name)) + '.map')

    sh "#{TOOLCHAIN}-ld -Map #{mapfn} -o #{t.name} -T #{script} " +
       "#{t.prerequisites.join(' ')} #{ENV['LDFLAGS']} --gc-sections -lgcc"

    sh "#{TOOLCHAIN}-strip -N ISRVectors -N NVICVectors -N support.rs -N app.rs -N isr.rs #{t.name}"
  end
end

def listing(h)
  Rake::FileTask.define_task(h) do |t|
    sh "#{TOOLCHAIN}-objdump -D #{t.prerequisites.first} > #{t.name}"
  end
end

def make_binary(h)
  Rake::FileTask.define_task(h) do |t|
    sh "#{TOOLCHAIN}-objcopy #{t.prerequisites.first} #{t.name} -O binary"
  end
end
