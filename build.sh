#!/bin/bash

rm -r www/public/build
npm --prefix www run build

cargo clean
cargo build --release
