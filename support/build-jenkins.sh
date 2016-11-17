#!/bin/bash

set -e

rustup show

print_info() {
  echo " * $1 at: $(which $1)"
  echo " * $1 version: `$1 --version`"
}

print_info rustc
print_info cargo
print_info xargo

if [ "$PLATFORM" == "native" ]; then
  # build unit tests
  echo " * building zinc"
  cargo test --features test --lib --verbose
  echo " * building ioreg"
  (cd ./ioreg; cargo build --verbose; cargo test --verbose)
  echo " * building platformtree"
  (cd ./platformtree; cargo build --verbose; cargo test --verbose)
  echo " * building platformtree macro"
  (cd ./macro_platformtree; cargo build --verbose; cargo test --verbose)
  echo " * building zinc macro"
  (cd ./macro_zinc; cargo test --verbose)

  echo " * generating coverage data"
  if [ "$TRAVIS_JOB_ID" != "" ]; then
    kcov-master/tmp/usr/local/bin/kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo target/kcov target/debug/zinc-*
    kcov-master/tmp/usr/local/bin/kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo target/kcov ioreg/target/debug/test-*
    kcov-master/tmp/usr/local/bin/kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo target/kcov platformtree/target/debug/platformtree-*
  else
    kcov cov/ target/debug/zinc-*
    support/fixcov.py src/ cov/zinc-????????????????/cobertura.xml
    kcov cov/ ioreg/target/debug/test-*
    support/fixcov.py ioreg/src/ cov/test-????????????????/cobertura.xml
    kcov cov/ platformtree/target/debug/platformtree-*
    support/fixcov.py platformtree/src/ cov/platformtree-????????????????/cobertura.xml
  fi

else
  # build cross-compiled lib and examples
  case "$PLATFORM" in
    lpc11xx )
      TARGET=thumbv6m-none-eabi
      EXAMPLES="empty"
      ;;
    lpc17xx )
      TARGET=thumbv7m-none-eabi
      EXAMPLES="empty blink_lpc17xx blink_pt uart dht22 rgb_pwm_lpc17xx adc_lpc17xx"
      ;;
    k20 )
      TARGET=thumbv7em-none-eabi
      EXAMPLES="empty blink_k20 blink_k20_isr"
      ;;
    stm32f1 )
      TARGET=thumbv7m-none-eabi
      EXAMPLES="empty blink_stm32f1 usart_stm32f1"
      ;;
    stm32f4 )
      TARGET=thumbv7em-none-eabi
      EXAMPLES="empty blink_stm32f4"
      ;;
    stm32l1 )
      TARGET=thumbv7m-none-eabi
      EXAMPLES="empty blink_stm32l1 bluenrg_stm32l1 usart_stm32l1"
      ;;
  esac

  xargo build --target=$TARGET --verbose --features "mcu_$PLATFORM" --lib

  for e in $EXAMPLES; do
    pushd "examples/$e"
    xargo build --target=$TARGET --verbose --features "mcu_$PLATFORM" --release
    popd
  done
fi
