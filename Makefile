RUSTCRATES = hidapi blink1 x360
RUSTBINDIR = bin

blink1_CRATE_DEPS += hidapi
x360_CRATE_DEPS += hidapi

blink1_DONT_DOC = 1
x360_DONT_DOC = 1

include rust-mk/rust.mk
