#!/usr/bin/env bash

cargo b
cargo b -r
cp -f ./target/release/vango ./vango-linux
cp -r ./testframework        ./vango-linux

