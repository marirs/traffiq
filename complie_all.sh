#!/bin/bash

cargo update

# compile for Apple Silicon
cargo b --release --target aarch64-apple-darwin
# compile for Apple Intel
cargo b --release --target x86_64-apple-darwin
# compile for Linux Arm
cargo b --release --target aarch64-unknown-linux-gnu
# compile for Linux Intel
cargo b --release --target x86_64-unknown-linux-gnu
# compile for Windows Intel
cargo b --release --target x86_64-pc-windows-gnu

# create the dist folder
[ -d dist ] || mkdir -p dist/

# copy the release files into the dist folder
cp target/aarch64-apple-darwin/release/traffiq dist/traffiq_macos-aarch64
cp target/x86_64-apple-darwin/release/traffiq dist/traffiq_macos-x86_64
cp target/aarch64-unknown-linux-gnu/release/traffiq dist/traffiq_linux-aarch64
cp target/x86_64-unknown-linux-gnu/release/traffiq dist/traffiq_linux-x86_64
cp target/x86_64-pc-windows-gnu/release/traffiq.exe dist/traffiq_win-x86_64.exe

# EOF