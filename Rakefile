load 'support/rake.rb'

TOOLCHAIN = 'arm-none-eabi-'
RUNTIME_LIB = '/opt/gcc-arm-none-eabi-4_7-2013q3/lib/gcc/arm-none-eabi/4.7.4/<%= @platform.arch.arch %>/libgcc.a'
RUSTC = 'rustc'

features = [:tft_lcd, :multitasking]

Context.create(__FILE__, ENV['PLATFORM'], features)

provide_stdlibs

desc "Run tests"
task :test

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

# zinc runtime support lib
compile_rust :zinc_support, {
  source:  'lib/support.rs'.in_source,
  produce: 'support.o'.in_intermediate,
  llvm_pass: :inline,
  lto: false,
  recompile_on: :triple,
}

# zinc isr crate
compile_rust :zinc_isr, {
  source:  'hal/isr.rs'.in_source,
  deps:    :core_crate,
  produce: 'isr.o'.in_intermediate,
  recompile_on: [:triple, :features],
}

# zinc scheduler assembly
# TODO(farcaller): make platform-specific
if features.include?(:multitasking)
  compile_c :zinc_isr_sched, {
    source:  'hal/cortex_m3/sched.S'.in_source,
    produce: 'isr_sched.o'.in_intermediate,
    recompile_on: [:triple, :features],
  }
end

# platform tree
compile_rust :platformtree_crate, {
  source:    'platformtree/platformtree.rs'.in_root,
  produce:   'platformtree/platformtree.rs'.in_root.as_rlib.in_build,
  out_dir:   true,
  build_for: :host,
}

rust_tests :platformtree_test, {
  source:  'platformtree/platformtree.rs'.in_root,
  produce: 'platformtree_test'.in_build,
}

# macros
compile_rust :macro_platformtree, {
  source:    'macro/platformtree.rs'.in_root,
  deps:      [:platformtree_crate],
  produce:   'macro/platformtree.rs'.in_root.as_dylib.in_build,
  out_dir:   true,
  build_for: :host,
}

app_tasks = Context.instance.applications.map do |a|
  compile_rust "app_#{a}".to_sym, {
    source: "apps/app_#{a}.rs".in_root,
    deps: [
      :zinc_crate,
      :core_crate,
      :macro_platformtree,
    ],
    produce: "app_#{a}.o".in_intermediate(a),
    recompile_on: [:triple, :platform, :features],
  }

  link_binary "app_#{a}_elf".to_sym, {
    script: 'layout.ld'.in_platform,
    deps: ["app_#{a}".to_sym, :zinc_isr, :zinc_support] +
          (features.include?(:multitasking) ? [:zinc_isr_sched] : []),
    produce: "app_#{a}.elf".in_build,
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
task :build_all => [:build_empty, :build_blink]
