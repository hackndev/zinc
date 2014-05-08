$: << File.dirname(__FILE__)

require 'build/helpers'
require 'build/context'
require 'build/deps'

def report_size(n, h)
  fn = h[:source]
  Rake::Task.define_task n => fn do |t|
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

def compile_rust(n, h)
  h.resolve_deps!
  Context.instance.rules[n] = h

  outflags = h[:out_dir] ? "--out-dir #{File.dirname(h[:produce])}" : "-o #{h[:produce]}"
  llvm_pass = h[:llvm_pass]
  lto = h[:lto]
  lto = true if lto == nil
  optimize = h[:optimize]
  crate_type = h[:crate_type] ? "--crate-type #{h[:crate_type]}" : ""
  ignore_warnings = h[:ignore_warnings] ? h[:ignore_warnings] : []
  ignore_warnings = ignore_warnings.map { |w| "-A #{w}" }.join(' ')

  declared_deps = h[:deps]
  rust_src = h[:source]
  deps = Deps.collect_dep_srcs(rust_src, '__ROOT__').to_a
  all_deps = [rust_src, declared_deps, deps].flatten.compact

  recompile_on = [h[:recompile_on]].flatten.compact
  recompile_on = recompile_on.map do |r|
    case r
    when :triple
      Context.instance.tracking_triple.name
    when :platform
      Context.instance.tracking_platform.name
    when :application_name
      Context.instance.tracking_application_name.name
    end
  end

  all_deps += recompile_on

  search_paths = [h[:search_paths]].flatten.compact

  Rake::FileTask.define_task(h[:produce] => all_deps) do |t|
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

    flags = :rustcflags.in_env.join(' ')
    if optimize
      flags.gsub!(/--opt-level \d/, "--opt-level #{optimize}")
    end

    search_paths = search_paths.map { |s| "-L #{s}"}.join(' ')
    search_paths += " -L #{Context.instance.build_dir}"

    sh "#{:rustc.in_env} #{flags} " +
       "#{do_lto ? '-Z lto' : ''} #{crate_type} #{emit} " +
       "#{search_paths} #{codegen} " +
       "#{outflags} #{ignore_warnings} #{rust_src}"
  end
end

def link_binary(n, h)
  h.resolve_deps!
  script = h[:script]

  Rake::FileTask.define_task(h[:produce] => [h[:deps], script].flatten) do |t|
    t.prerequisites.delete(script)
    mapfn = Context.instance.build_dir(File.basename(t.name, File.extname(t.name)) + '.map')

    sh "#{:ld.in_toolchain} -Map #{mapfn} -o #{t.name} -T #{script} " +
       "#{t.prerequisites.join(' ')} #{:ldflags.in_env.join(' ')} --gc-sections -lgcc"

    # sh "#{TOOLCHAIN}-strip -N ISRVectors -N NVICVectors -N support.rs -N app.rs -N isr.rs #{t.name}"
  end
end

def compile_c(n, h)
  h.resolve_deps!
  Context.instance.rules[n] = h

  Rake::FileTask.define_task(h[:produce] => [h[:source], h[:deps]].flatten.compact) do |t|
    sh "#{:gcc.in_toolchain} #{:cflags.in_env.join(' ')} -o #{h[:produce]} -c #{h[:source]}"
  end
end

def listing(n, h)
  Rake::FileTask.define_task(h[:produce] => h[:source]) do |t|
    sh "#{:objdump.in_toolchain} -D #{t.prerequisites.first} > #{t.name}"
  end
end

def make_binary(n, h)
  Rake::FileTask.define_task(h[:produce] => h[:source]) do |t|
    sh "#{:objcopy.in_toolchain} #{t.prerequisites.first} #{t.name} -O binary"
  end
end

def provide_stdlibs
  liblibc_src = 'thirdparty/liblibc/lib.rs'.in_root
  libstd_src = 'thirdparty/libstd/lib.rs'.in_root

  directory 'thirdparty'.in_root

  Rake::FileTask.define_task 'thirdparty/rust' do |t|
    sh "git clone --single-branch --depth 1 https://github.com/mozilla/rust #{t.name} && " +
    "cd thirdparty/rust/src && patch -p1 -i ../../../support/rust.patch"
  end

  Rake::FileTask.define_task libstd_src => 'thirdparty/rust' do |t|
    sh "ln -s rust/src/libstd thirdparty/libstd"
  end.invoke

  Rake::FileTask.define_task liblibc_src => 'thirdparty/rust' do |t|
    sh "ln -s rust/src/liblibc thirdparty/liblibc"
  end.invoke

  Rake::FileTask.define_task 'librustrt.a'.in_build do |t|
    sh "#{:ar.in_toolchain} cr #{t.name}"
  end.invoke

  Rake::FileTask.define_task 'libbacktrace.a'.in_build do |t|
    sh "#{:ar.in_toolchain} cr #{t.name}"
  end.invoke
end
