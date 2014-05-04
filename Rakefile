load 'support/rake.rb'

TOOLCHAIN = 'arm-none-eabi'
TOOLCHAIN_LIBS_PATH = '/opt/gcc-arm-none-eabi-4_7-2013q3/lib/gcc/arm-none-eabi/4.7.4'
RUSTC = 'rustc'
FORCE_NATIVE_BUILD = false

features = [:tft_lcd, :multitasking]

architectures = {
  cortex_m3: {
    arch: 'armv7-m',
    cpu: 'cortex-m3',
    target: 'thumbv7m-linux-eabi',
  },
  cortex_m4: {
    arch: 'armv7e-m',
    cpu: 'cortex-m4',
    target: 'thumbv7em-linux-eabi',
  },
}

platforms = {
  lpc17xx: {
    arch: :cortex_m3,
    config: :mcu_lpc17xx,
    features: [:mcu_has_spi],
  },
  stm32f4: {
    arch: :cortex_m4,
    config: :mcu_stm32f4,
  },
}

rsflags = %w[-Z no-landing-pads -C relocation_model=static]
ldflags = %w[]

Context.prepare!(rsflags, ldflags, platforms, architectures, features)

compile_rust :libc_crate, {
  source:  'liblibc/lib.rs'.in_root,
  produce: 'liblibc/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
}

compile_rust :std_crate, {
  source:  'libstd/lib.rs'.in_root,
  deps:    :libc_crate,
  produce: 'libstd/lib.rs'.in_root.as_rlib.in_build,
  out_dir: true,
}

# zinc crate
compile_rust :zinc_crate, {
  source:  'main.rs'.in_source,
  deps:    :std_crate,
  produce: 'main.rs'.in_source.as_rlib.in_build,
  out_dir: true,
}

# zinc runtime support lib
compile_rust :zinc_support, {
  source:  'lib/support.rs'.in_source,
  produce: 'support.o'.in_intermediate,
  llvm_pass: :inline,
  lto: false,
}

# zinc isr crate
compile_rust :zinc_isr, {
  source:  'hal/isr.rs'.in_source,
  deps:    :std_crate,
  produce: 'isr.o'.in_intermediate,
}

# zinc isr weak symbols
# TODO(farcaller): in_platform?
compile_c :zinc_isr_weak, {
  source:  'hal/cortex_m3/default_handlers.S'.in_source,
  produce: 'isr_weak.o'.in_intermediate,
}

# zinc scheduler assembly
# TODO(farcaller): make platform-specific
if features.include?(:multitasking)
  compile_c :zinc_isr_sched, {
    source:  'hal/cortex_m3/sched.S'.in_source,
    produce: 'isr_sched.o'.in_intermediate,
  }
end

compile_rust :app_crate, {
  source: Context.app,
  deps: [
    :zinc_crate,
    Context.track_application_name,
  ],
  produce: Context.app.as_rlib.in_build,
  out_dir: true,
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
  deps: [:app, :zinc_isr, :zinc_support, :zinc_isr_weak] +
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
