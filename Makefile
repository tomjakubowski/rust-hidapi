RUSTCRATES = hidapi blink1
RUSTBINDIR = bin

blink1_CRATE_DEPS += hidapi

include rust-mk/rust.mk
