#!/usr/bin/env bash

cargo b
cargo b -r
mv -f target/release/vango vango-linux/vango

