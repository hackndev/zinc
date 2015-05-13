# ===========================
# BEGIN CONFIGURATION SECTION
# ===========================
STRIP=arm-none-eabi-strip
OBJCOPY=arm-none-eabi-objcopy
OBJDUMP=arm-none-eabi-objdump
# =========================
# END CONFIGURATION SECTION
# =========================
# You shouldn't have to edit anything below this line

# TODO(mcoffin): derive TARGET and TARGET_CPU from PLATFORM and EXAMPLE_NAME
TARGET=thumbv7m-none-eabi
TARGET_CPU=cortex-m3

# Information based on the user-configured PLATFORM
PLATFORM_DIR=src/hal/$(PLATFORM)
LINK_SCRIPT=$(PLATFORM_DIR)/layout.ld

# Information about where output is to go
OUT_DIR=target/$(TARGET)/release
EXAMPLE_DIR=$(OUT_DIR)/examples

# Information about the special ISR crate
ISR_SRC=src/hal/isr.rs
ISR_CRATE=$(shell rustc --print crate-name $(ISR_SRC))
ISR_FILE=$(OUT_DIR)/$(ISR_CRATE).o

# Invocation flags
LDFLAGS=-mthumb -mcpu=$(TARGET_CPU) -T$(LINK_SCRIPT) -lm -lgcc $(ISR_FILE)
RUSTC_FLAGS=--target=$(TARGET) --out-dir=$(OUT_DIR) --cfg mcu_$(PLATFORM) -C opt-level=2

# Convenience variables for specific files
BIN_FILE=$(EXAMPLE_DIR)/$(EXAMPLE_NAME).bin
LST_FILE=$(EXAMPLE_DIR)/$(EXAMPLE_NAME).lst
EXAMPLE_FILE=$(EXAMPLE_DIR)/$(EXAMPLE_NAME)

.PHONY: build clean listing
# Dummy target for building the configured example
build: $(BIN_FILE)

clean:
	-rm $(ISR_FILE)
	cargo clean

listing: $(OUT_DIR)/$(EXAMPLE_NAME).lst

# FIXME(mcoffin): Make this target depend on all the dependencies of the rust
# crate and the example source code
$(EXAMPLE_FILE): $(ISR_FILE)
	cargo rustc --example $(EXAMPLE_NAME) --release --target=$(TARGET) --verbose -- -C link-args="$(LDFLAGS)"

$(BIN_FILE): $(EXAMPLE_FILE)
	$(OBJCOPY) -O binary $< $@

$(LST_FILE): $(EXAMPLE_FILE)
	$(OBJDUMP) -D $< > $@

# FIXME(mcoffin): This target depends only on the root .rs source file of the ISR
# crate. We should find a way to print the dep-info with rustc and make it
# depend on the whole crate because as it stands right now, changes to files
# other than the root file won't cause re-compilation of the crate
$(ISR_FILE): $(ISR_SRC) | $(OUT_DIR)
	rustc $(RUSTC_FLAGS) --emit=obj $<
	$(STRIP) -N rust_begin_unwind -N rust_stack_exhausted -N rust_eh_personality $@

$(OUT_DIR):
	mkdir -p $@
