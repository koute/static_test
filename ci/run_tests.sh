#!/bin/bash

set -euo pipefail
cd "$(dirname $(readlink -f "$0"))/.."

export RUST_BACKTRACE=1

set +e
echo "$(rustc --version)" | grep -q "nightly"
if [ "$?" = "0" ]; then
    export IS_NIGHTLY=1
else
    export IS_NIGHTLY=0
fi
set -e

echo "Is Rust from nightly: $IS_NIGHTLY"

if [ "$IS_NIGHTLY" = "1" ]; then
    cargo test --features external_doc --release
else
    cargo test --release
fi

set +e

echo "Running 'test_fail'..."
rm -f target/test_fail.txt
cargo rustc --example test_fail --profile test &> target/test_fail.txt
grep -q "error: unreachable code can be reached in 'test_fail'" target/test_fail.txt

if [ "$?" -ne "0" ]; then
    echo "The 'test_fail' test failed!"
    cat target/test_fail.txt
    exit 1
else
    echo "Test 'test_fail' succeeded!"
fi
