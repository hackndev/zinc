#!/bin/bash

set -e

echo " * rustc version: `rustc --version`"

if [ "$PLATFORM" == "native" ]; then
  # build unit tests
  cargo test --lib --verbose
  (cd ./ioreg; cargo build --verbose; cargo test --verbose)
  (cd ./platformtree; cargo build --verbose; cargo test --verbose)
  (cd ./macro_platformtree; cargo build --verbose; cargo test --verbose)
  (cd ./macro_zinc; cargo test --verbose)

  echo " * generating coverage data"
  kcov cov/ target/debug/zinc-*
  kcov cov/ ioreg/target/debug/test-*
  kcov cov/ platformtree/target/debug/platformtree-*
else
  # build cross-compiled lib and examples
  case "$PLATFORM" in
    lpc17xx )
      TARGET=thumbv7m-none-eabi
      EXAMPLES="blink blink_pt uart dht22 empty"
      ;;
    k20 )
      TARGET=thumbv7em-none-eabi
      EXAMPLES="blink_k20 blink_k20_isr empty"
      ;;
    stm32f4 )
      TARGET=thumbv7em-none-eabi
      EXAMPLES="blink_stm32f4 empty"
      ;;
    stm32l1 )
      TARGET=thumbv7m-none-eabi
      EXAMPLES="blink_stm32l1 bluenrg_stm32l1 usart_stm32l1 empty"
      ;;
  esac

  ./configure --host=arm-none-eabi
  cargo build --target=$TARGET --verbose --features "mcu_$PLATFORM"

  for e in $EXAMPLES; do
    EXAMPLE_NAME=$e make build
  done
fi
