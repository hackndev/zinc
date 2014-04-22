load 'support/rake.rb'

TOOLCHAIN = 'arm-none-eabi'
TOOLCHAIN_LIBS_PATH = '/opt/gcc-arm-none-eabi-4_7-2013q3/lib/gcc/arm-none-eabi/4.7.4'
RUSTC = 'rustc'

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
  },
  stm32f4: {
    arch: :cortex_m4,
    config: :mcu_stm32f4,
  },
}

rsflags = %w[-Z no-landing-pads -C relocation_model=static]
ldflags = %w[]

Context.prepare!(rsflags, ldflags, platforms, architectures)

# rust-core crate
compile_rust Context.build_dir(Rlib.name(Context.root_dir('rust-core/core/lib.rs'))) => Context.root_dir('rust-core/core/lib.rs'),
  out_dir: true

# zinc crate
compile_rust Context.build_dir(Rlib.name(Context.src_dir('main.rs'))) => [
  Context.src_dir('main.rs'),
  Context.build_dir(Rlib.name(Context.root_dir('rust-core/core/lib.rs'))),
], out_dir: true

compile_rust Context.build_dir('zinc-crate.ll') => [
  Context.src_dir('main.rs'),
  Context.build_dir(Rlib.name(Context.root_dir('rust-core/core/lib.rs'))),
], out_dir: true

# zinc runtime support lib
compile_rust Context.intermediate_dir('support.o') => Context.src_dir('lib/support.rs'),
  llvm_pass: :inline, lto: false

# zinc isr crate
compile_rust Context.intermediate_dir('isr.o') => [
  Context.src_dir('hal/isr.rs'),
  Context.build_dir(Rlib.name(Context.root_dir('rust-core/core/lib.rs'))),
]

# demo app
compile_rust Context.intermediate_dir('app.o') => [
  Context.app,
  Context.app_dep,
  Context.build_dir(Rlib.name(Context.src_dir('main.rs'))),
]

compile_rust Context.build_dir('zinc.ll') => [
  Context.app,
  Context.app_dep,
  Context.build_dir(Rlib.name(Context.src_dir('main.rs'))),
]

link_binary script: Context.platform_dir('layout.ld'), Context.build_dir('zinc.elf') => [
  Context.intermediate_dir('app.o'),
  Context.intermediate_dir('support.o'),
  Context.intermediate_dir('isr.o'),
]

make_binary Context.build_dir('zinc.bin') => [
  Context.build_dir('zinc.elf'),
  # Context.build_dir('zinc.ll'),
  # Context.build_dir('zinc-crate.ll'),
]

listing Context.build_dir('zinc.lst') => Context.build_dir('zinc.elf')
report_size Context.build_dir('zinc.elf')

task :build => [Context.build_dir('zinc.bin'), Context.build_dir('zinc.lst'), :report_size]

task :default => :build
