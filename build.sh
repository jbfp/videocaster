#!/bin/bash
rm -r www/public/build
rm -r dpkg/usr

npm --prefix www run build
cargo clean --release
cargo build --release
mkdir -p dpkg/usr/local/bin
cp target/release/videocaster dpkg/usr/local/bin
dpkg-deb --build dpkg
mv dpkg.deb videocaster_1.0.0.deb
