#!/bin/bash
set -ex

REPO=$(git rev-parse --show-toplevel)
CONTRACTS="$REPO/contracts"

# the target for the binaries and documentation
TARGET="wasm32-unknown-unknown"

# triggers all build.rs steps
find $CONTRACTS \
    -maxdepth 2 \
    -name build.rs \
    -prune \
    -exec touch -c {} \; 
# in this way the wasm files will have up to date
# versioning information

# build the contract's wasm binaries
cargo build --target $TARGET --release "$@"
# they are stored on $CONTRACTS/target/$TARGET/release

# makes the wasm copy into $CONTRACTS/res/
find "$CONTRACTS/target/$TARGET/release" \
    -maxdepth 1 \
    -name \*.wasm \
    -prune \
    -exec cp {} "$CONTRACTS/res/" \;

# reduces the wasm size
for f in $CONTRACTS/res/*.wasm
do
    wasm-opt -Oz -o "$f" "$f"
done
# note: for more info, check:
# https://github.com/WebAssembly/binaryen
# https://rustwasm.github.io/book/reference/code-size.html#use-the-wasm-opt-tool

# shows the wasm binaries and their size
ls -lah $CONTRACTS/res/*.wasm | awk '{print $5 " " $9}'
