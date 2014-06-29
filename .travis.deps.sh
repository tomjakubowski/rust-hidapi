#!/bin/bash
set -ex

sudo add-apt-repository -y ppa:hansjorg/rust
sudo apt-get update -qq

sudo apt-get install rust-nightly

mkdir -p tmp/
cd tmp/

# FIXME: linking is broken on linux, so don't actually need to install
# hidapi
# sudo apt-get install libudev-dev libusb-1.0-0-dev
# sudo apt-get install autotools-dev autoconf automake libtool
# curl -L -O https://github.com/signal11/hidapi/archive/hidapi-0.8.0-rc1.tar.gz
# tar xfz hidapi-0.8.0-rc1.tar.gz
# (cd hidapi-hidapi-0.8.0-rc1 && ./bootstrap && ./configure && make && sudo make install)
git clone --depth 1 --recursive https://github.com/rust-lang/cargo
(cd cargo && make && sudo make install)

cd ..
rm -rf tmp/

set +x
