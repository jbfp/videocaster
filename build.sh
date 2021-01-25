#!/bin/bash

rm -r www/public/build
npm --prefix www run build

cargo clean -- release
cargo build --release
