#!/bin/bash
#
#  Test runner script meant to be executed inside of a docker container
#
#  Usage: travis_run.sh [OPTIONS...]
#
#  Where OPTIONS are passed directly to ./configure
#
#  The script is otherwise influenced by the following environment variables:
#
#  JOBS=N        Argument for make's -j option, default=2
#
#  And, obviously, some crucial variables that configure itself cares about:
#
#  CC, CXX, LDFLAGS, CFLAGS, etc.
#

# Fail loudly if any command fails
set -e

export JOBS=${JOBS:-2}

echo "Cargo build"
cargo build -j $JOBS

echo "Cargo test"
RUST_BACKTRACE=1 cargo test -j $JOBS
