#!/bin/bash
RUST_LOG=debug \
ROCKET_ADDRESS=0.0.0.0 \
ROCKET_PORT=8080 \
cargo run
