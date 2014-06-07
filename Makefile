RUSTCRATES = hidapi blink1 x360
RUSTBINDIR = bin

blink1_CRATE_DEPS += hidapi
x360_CRATE_DEPS += hidapi

include rust-mk/rust.mk
