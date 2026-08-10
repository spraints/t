#!/bin/bash

set -e

if [ ! -f "$T_DATA_FILE" ]; then
  echo "T_DATA_FILE must be set to a real file"
  exit 1
fi

bench() {
  # Parse the file.
  time target/debug/bench-parse

  # Add up all the time in the file.
  time target/debug/bench-sum
}

# warmup
bench >&/dev/null
set -x
bench
