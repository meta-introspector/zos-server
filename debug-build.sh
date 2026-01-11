#!/bin/bash

echo "Building ZOS Oracle plugin..."
cd zos-oracle || exit
cargo build --features full 2>&1 | tee ../build-output.log
echo "Exit code: $?" >> ../build-output.log
