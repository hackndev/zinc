$: << File.dirname(__FILE__)

require 'build/helpers'
require 'build/context'
require 'build/deps'

ENV["RUST_TARGET_PATH"] = "support/target-specs"

def report_size(n, h)
  fn = h[:source]
  Rake::Task.define_task n => fn do |t|
    fn = t.prerequisites.first

    stats = `#{:size.in_toolchain} #{fn}`.split("\n").last.split("\t").map {|s|s.strip}
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
  more_flags = h[:flags] || ""

  declared_deps = h[:deps]
  rust_src = h[:source]
  deps = h[:do_not_collect_rust_deps] ? [] : Deps.collect_dep_srcs(rust_src, '__ROOT__').to_a
  all_deps = [rust_src, declared_deps, deps].flatten.compact

  recompile_on = [h[:recompile_on]].flatten.compact
  recompile_on = recompile_on.map do |r|
    case r
    when :triple
      Context.instance.tracking_triple.name
    when :platform
      Context.instance.tracking_platform.name
    end
  end

  all_deps += recompile_on

  search_paths = [h[:search_paths]].flatten.compact

  is_test = h[:test] == true
  build_for_host = h[:build_for] == :host || is_test

  should_fail = h[:should_fail] == true

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
    flags += ' ' + :rustcflags_cross.in_env.join(' ') unless build_for_host
    flags += ' --test' if is_test
    flags += ' -g'
    flags += ' -C no-stack-check'
    flags += ' ' + more_flags

    if optimize
      flags.gsub!(/--opt-level \d/, "--opt-level #{optimize}")
    end

    search_paths = search_paths.map { |s| "-L #{s}"}.join(' ')
    search_paths += " -L #{Context.instance.build_dir}"

    fail_wrap = should_fail ? '&>/dev/null' : ''

    sh "#{should_fail ? "! " : ""}#{:rustc.in_env} #{flags} " +
       "#{do_lto ? '-C lto' : ''} #{crate_type} #{emit} " +
       "#{search_paths} #{codegen} " +
       "#{outflags} #{ignore_warnings} #{rust_src} #{fail_wrap}"
    if File.extname(t.name) == '.o' && !should_fail
      sh "#{:strip.in_toolchain} -N rust_stack_exhausted " +
         "-N rust_eh_personality #{t.name}"
    end
  end
end

def link_binary(n, h)
  h.resolve_deps!
  script = h[:script]

  Rake::FileTask.define_task(h[:produce] => [h[:deps], script].flatten) do |t|
    t.prerequisites.delete(script)
    mapfn = Context.instance.build_dir(File.basename(t.name, File.extname(t.name)) + '.map')

    sh "#{:ld.in_toolchain} -Map #{mapfn} -o #{t.name} -T #{script} " +
       "#{t.prerequisites.join(' ')} #{:ldflags.in_env.join(' ')} --gc-sections"
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

def rust_tests(n, h)
  h[:test] = true
  h[:flags] = '-C relocation-model=static'
  h[:optimize] = '0'
  compile_rust n, h
  run_task = Rake::Task.define_task("run_#{n}".to_sym => h[:produce]) do |t|
    sh t.prerequisites.first
  end

  Rake::Task[:test].enhance([run_task])
end

def provide_stdlibs
  directory 'thirdparty'.in_root

  Rake::FileTask.define_task 'thirdparty/rust' do |t|
    sh "git clone --single-branch --depth 1 https://github.com/rust-lang/rust #{t.name}"
  end.invoke

  Rake::FileTask.define_task 'thirdparty/librlibc'.in_root do |t|
    sh "git clone https://github.com/rust-lang/rlibc #{t.name}"
  end.invoke

  Rake::FileTask.define_task 'thirdparty/libcore/lib.rs'.in_root do |t|
    sh "ln -s rust/src/libcore thirdparty/libcore"
  end.invoke

  Rake::FileTask.define_task 'thirdparty/hamcrest-rust'.in_root do |t|
    sh "git clone --single-branch --depth 1 https://github.com/carllerche/hamcrest-rust #{t.name}"
  end.invoke

  Rake::FileTask.define_task 'thirdparty/shiny'.in_root do |t|
    sh "git clone --single-branch --depth 1 https://github.com/farcaller/shiny #{t.name}"
  end.invoke
end
