#!/bin/bash
set -ex

echo "TRAVIS_PULL_REQUEST=$TRAVIS_PULL_REQUEST"
echo "TRAVIS_BRANCH=$TRAVIS_BRANCH"

if [[ $TRAVIS_PULL_REQUEST == "false" ]] && [[ $TRAVIS_BRANCH == "master" ]];
then
    curl http://www.rust-ci.org/artifacts/put?t=$RUSTCI_TOKEN | sh
fi

set +x
