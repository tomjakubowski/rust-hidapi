#!/bin/bash
set -ex

if [[ $TRAVIS_PULL_REQUEST == "false" ]] && [[ $TRAVIS_BRANCH == "master" ]];
then
    echo "Uploading..."
    set +x
    curl http://www.rust-ci.org/artifacts/put?t=$RUSTCI_TOKEN | sh
    set -x
fi

set +x
