load 'support/rake.rb'

TOOLCHAIN = 'arm-none-eabi'
TOOLCHAIN_LIBS_PATH = '/opt/gcc-arm-none-eabi-4_7-2013q3/lib/gcc/arm-none-eabi/4.7.4'
RUSTC = 'rustc'

features = [:tft_lcd, :multitasking]

Context.create(__FILE__, ENV['PLATFORM'], features)

provide_stdlibs

compile_rust :core_crate, {
  source:  'thirdparty/libcore/lib.rs'.in_root,
  produce: 'thirdparty/libcore/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
  recompile_on: :triple,
}

# zinc crate
compile_rust :zinc_crate, {
  source:  'main.rs'.in_source,
  deps:    :core_crate,
  produce: 'main.rs'.in_source.as_rlib.in_build,
  out_dir: true,
  recompile_on: [:triple, :platform, :features],
}

# dummy archives
compile_ar :dummy_morestack, {
  produce: 'libmorestack.a'.in_intermediate,
}
compile_ar :dummy_compiler_rt, {
  produce: 'libcompiler-rt.a'.in_intermediate,
}

# zinc scheduler assembly
# TODO(farcaller): make platform-specific
if features.include?(:multitasking)
  compile_c :zinc_isr_sched_c, {
    source:  'hal/cortex_m3/sched.S'.in_source,
    produce: 'libisr_sched.o'.in_intermediate,
    recompile_on: [:triple, :features],
  }
  compile_ar :zinc_isr_sched, {
    source: 'libisr_sched.o'.in_intermediate,
    produce: 'libisr_sched.a'.in_intermediate,
    recompile_on: [:triple, :features],
  }
end

app_tasks = Context.instance.applications.map do |a|
  compile_rust "app_#{a}_crate".to_sym, {
    source: "apps/app_#{a}.rs".in_root,
    deps: [
      :zinc_crate,
      :core_crate,
    ],
    produce: "apps/app_#{a}.rs".in_root.as_rlib.in_intermediate(a),
    out_dir: true,
    recompile_on: [:triple, :platform, :features],
  }

  compile_rust "app_#{a}_elf".to_sym, {
    script: 'layout.ld'.in_platform,
    source: 'lib/app.rs'.in_source,
    deps: [
      :core_crate,
      :zinc_crate,
      "app_#{a}_crate".to_sym,
      :dummy_morestack,
      :dummy_compiler_rt,
    ] + (features.include?(:multitasking) ? [:zinc_isr_sched] : []), 
    produce: "app_#{a}.elf".in_build,
    search_paths: [a.in_intermediate, "intermediate".in_build],
  }

  t_bin = make_binary "app_#{a}_bin".to_sym, {
    source:  "app_#{a}.elf".in_build,
    produce: "app_#{a}.bin".in_build,
  }

  t_lst = listing "app_#{a}_lst".to_sym, {
    source:  "app_#{a}.elf".in_build,
    produce: "app_#{a}.lst".in_build,
  }

  t_size = report_size "app_#{a}_size".to_sym, {
    source: "app_#{a}.elf".in_build,
  }

  desc "Build application #{a}"
  task "build_#{a}".to_sym => [t_bin.name, t_lst.name, t_size.name]
end

desc "Build all applications"
task :build_all => app_tasks.map { |t| t.name }
