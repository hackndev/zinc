load 'support/rake.rb'

TOOLCHAIN = 'arm-none-eabi-'
RUSTC = 'rustc'

Context.create(__FILE__, ENV['PLATFORM'])

provide_stdlibs

# tests
desc "Run tests"
task :test
compile_rust :hamcrest_crate, {
  source:  'thirdparty/hamcrest-rust/src/hamcrest/lib.rs'.in_root,
  produce: 'thirdparty/hamcrest-rust/src/hamcrest/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
  build_for: :host,
  do_not_collect_rust_deps: true,
}

# cross-compiled librlibc
compile_rust :rlibc_crate, {
  source:  'thirdparty/librlibc/lib.rs'.in_root,
  produce: 'thirdparty/librlibc/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
  recompile_on: :triple,
}

# cross-compiled libcore
compile_rust :core_crate, {
  source:  'thirdparty/libcore/lib.rs'.in_root,
  produce: 'thirdparty/libcore/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
  recompile_on: :triple,
}

# ioreg
compile_rust :ioreg_crate, {
  source:    'ioreg/ioreg.rs'.in_root,
  produce:   'ioreg/ioreg.rs'.in_root.as_rlib.in_build,
  out_dir:   true,
  build_for: :host,
}

compile_rust :macro_ioreg, {
  source:    'macro/ioreg.rs'.in_root,
  deps:      [:ioreg_crate],
  produce:   'macro/ioreg.rs'.in_root.as_dylib.in_build,
  out_dir:   true,
  build_for: :host,
}

# zinc crate
compile_rust :zinc_crate, {
  source:  'main.rs'.in_source,
  deps:    [:core_crate, :rlibc_crate, :macro_ioreg],
  produce: 'main.rs'.in_source.as_rlib.in_build,
  out_dir: true,
  recompile_on: [:triple, :platform],
}

# zinc isr crate
compile_rust :zinc_isr, {
  source:  'hal/isr.rs'.in_source,
  deps:    :core_crate,
  produce: 'isr.o'.in_intermediate,
  recompile_on: [:triple],
}

# zinc scheduler assembly
# TODO(farcaller): broken until implemented in PT.
# compile_c :zinc_isr_sched, {
#   source:  'hal/cortex_m3/sched.S'.in_source,
#   produce: 'isr_sched.o'.in_intermediate,
#   recompile_on: [:triple],
# }

# platform tree
compile_rust :platformtree_crate, {
  source:    'platformtree/platformtree.rs'.in_root,
  produce:   'platformtree/platformtree.rs'.in_root.as_rlib.in_build,
  out_dir:   true,
  build_for: :host,
  optimize: 0,
}

rust_tests :platformtree_test, {
  source:  'platformtree/platformtree.rs'.in_root,
  deps:    :hamcrest_crate,
  produce: 'platformtree_test'.in_build,
}

rust_tests :zinc_test, {
  source:  'main.rs'.in_source,
  deps:    [:core_crate, :macro_ioreg],
  produce: 'zinc_test'.in_build,
  recompile_on: [:platform],
  build_for: :host,
}

# macros
compile_rust :macro_platformtree, {
  source:    'macro/platformtree.rs'.in_root,
  deps:      [:platformtree_crate],
  produce:   'macro/platformtree.rs'.in_root.as_dylib.in_build,
  out_dir:   true,
  build_for: :host,
  optimize: 0,
}

desc "Build API documentation"
task build_docs: [:build_docs_html]

task build_docs_html: [] do |t|
  ['src/main.rs', 'platformtree/platformtree.rs', 'ioreg/ioreg.rs'].each do |f|
    build = Context.instance.build_dir
    sh ("rustdoc -w html -o #{build}/doc -L #{build} " \
	+ f + ' ' + :config_flags.in_env.join(' '))
  end
end

app_tasks = Context.instance.applications.map do |a|
  compile_rust "app_#{a}".to_sym, {
    source: "apps/app_#{a}.rs".in_root,
    deps: [
      :zinc_crate,
      :core_crate,
      :macro_platformtree,
    ],
    produce: "app_#{a}.o".in_intermediate(a),
    recompile_on: [:triple, :platform],
  }

  link_binary "app_#{a}_elf".to_sym, {
    script: 'layout.ld'.in_platform,
    deps: ["app_#{a}".to_sym, :zinc_isr],
    # TODO(farcaller): broken until implemented in PT.
    # (features.include?(:multitasking) ? [:zinc_isr_sched] : []),
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
case ENV['PLATFORM']
when 'k20'
  task :build_all => [:build_blink_k20]
when 'stm32f4'
  task :build_all => [:build_blink_stm32f4]
else
  task :build_all => [:build_empty, :build_blink, :build_uart, :build_dht22]
end
