#!/bin/bash

set -e
set -o nounset

COMMAND="${COMMAND:-t}"
ROOT="$(cd "$(dirname "$0")"; pwd -P)"
TMPDIR="${ROOT}/tmp"
export T_DATA_FILE="${TMPDIR}/test-sh-t.csv"

mkdir -p "$TMPDIR"
rm -f "$T_DATA_FILE"

assert_out() {
  local expected="${1}"
  shift # "expected"
  shift # --
  echo T "$@"
  local actual="$($COMMAND "$@")"
  if ! echo "$actual" | grep -q "$expected"; then
    echo FAIL
    echo expected to find:
    echo "$expected"
    echo but got:
    echo "$actual"
    exit 1
  fi
  echo PASS
}

assert_out "NOT working" -- status
assert_out "You haven't started working yet" -- stop

assert_out "Starting work" -- start
assert_out "You already started working," -- start

assert_out "WORKING" -- status

assert_out "You just worked for " -- stop
assert_out "You haven't started working yet" -- stop

assert_out "NOT working" -- status

TODAY="$(date +%Y-%m-%d)"
EDITOR="(echo $TODAY 00:00,$TODAY 01:00; echo $TODAY 01:45, $TODAY 02:55) | tee" \
  $COMMAND edit >/dev/null

assert_out "You have worked for 130 minutes today." -- today
assert_out "You have worked for 130 minutes since " -- week
