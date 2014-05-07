load 'support/rake.rb'

TOOLCHAIN = 'arm-none-eabi'
TOOLCHAIN_LIBS_PATH = '/opt/gcc-arm-none-eabi-4_7-2013q3/lib/gcc/arm-none-eabi/4.7.4'
RUSTC = 'rustc'

features = [:tft_lcd, :multitasking]

Context.create(__FILE__, ENV['PLATFORM'], features)

provide_stdlibs

compile_rust :libc_crate, {
  source:  'thirdparty/liblibc/lib.rs'.in_root,
  produce: 'thirdparty/liblibc/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
  recompile_on: :triple,
}

compile_rust :std_crate, {
  source:  'thirdparty/libstd/lib.rs'.in_root,
  deps:    :libc_crate,
  produce: 'thirdparty/libstd/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
  ignore_warnings: ['unused_variable'],
  recompile_on: :triple,
}

# zinc crate
compile_rust :zinc_crate, {
  source:  'main.rs'.in_source,
  deps:    :std_crate,
  produce: 'main.rs'.in_source.as_rlib.in_build,
  out_dir: true,
  recompile_on: [:triple, :platform],
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
  deps:    :std_crate,
  produce: 'isr.o'.in_intermediate,
  recompile_on: :triple,
}

# zinc scheduler assembly
# TODO(farcaller): make platform-specific
if features.include?(:multitasking)
  compile_c :zinc_isr_sched, {
    source:  'hal/cortex_m3/sched.S'.in_source,
    produce: 'isr_sched.o'.in_intermediate,
    recompile_on: :triple,
  }
end

compile_rust :app_crate, {
  source: Context.instance.application,
  deps: [
    :zinc_crate,
  ],
  produce: Context.instance.application.as_rlib.in_build,
  out_dir: true,
  recompile_on: [:triple, :platform, :application_name],
}

compile_rust :app, {
  source: 'lib/app.rs'.in_source,
  deps: [
    :std_crate,
    :zinc_crate,
    :app_crate,
  ],
  produce: 'app.o'.in_intermediate,
}

link_binary :app_elf, {
  script: 'layout.ld'.in_platform,
  deps: [:app, :zinc_isr, :zinc_support] +
        (features.include?(:multitasking) ? [:zinc_isr_sched] : []),
  produce: 'zinc.elf'.in_build,
}

make_binary :app_bin, {
  source:  'zinc.elf'.in_build,
  produce: 'zinc.bin'.in_build,
}

listing :app_lst, {
  source:  'zinc.elf'.in_build,
  produce: 'zinc.lst'.in_build,
}

report_size 'zinc.elf'.in_build

task :build => ['zinc.bin'.in_build, 'zinc.lst'.in_build, :report_size]
task :default => :build
