#!/bin/bash
set -eux

www_build=www/public/build
dpkg_build=dpkg/usr

if [ -f "$www_build" ] ; then
    rm -r "$www_build"
fi

if [ -f "$dpkg_build" ] ; then
    rm -r "$dpkg_build"
fi

npm --prefix www run build
cargo clean --release
cargo build --release --locked
mkdir -p dpkg/usr/local/bin
cp target/release/videocaster dpkg/usr/local/bin
dpkg-deb --build dpkg
mv dpkg.deb videocaster_1.2.0.deb
