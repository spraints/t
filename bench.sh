#!/bin/bash

set -e

if [ ! -f "$T_DATA_FILE" ]; then
  echo "T_DATA_FILE must be set to a real file"
  exit 1
fi

cargo build

set -x

# Parse the file.
time ruby -Ilib -e '
  require "t/data"
  puts T::Data.new(ENV["T_DATA_FILE"]).entries.size
'

# Parse the file and add up all the time.
time ruby -Ilib -e '
  require "t/data"
  puts T::Data.new(ENV["T_DATA_FILE"]).entries.inject(0) { |sum, e| sum + e.minutes }
'

# Parse the file.
time target/debug/bench-parse

# Add up all the time in the file.
time target/debug/bench-sum
