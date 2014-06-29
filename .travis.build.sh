#!/bin/bash
set -ex

rustc --version
cargo build

# FIXME: linux linking issue
# cargo test

rustdoc -L target/ -o doc --test src/hidapi/lib.rs

set +x
