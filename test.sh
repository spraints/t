#!/bin/bash

set -e
set -o nounset

COMMAND="${COMMAND:-t}"
ROOT="$(cd "$(dirname "$0")"; pwd -P)"
FIXTURES="${ROOT}/tests/fixtures/integration"
TMPDIR="${ROOT}/tmp"
export T_DATA_FILE="${TMPDIR}/test-sh-t.csv"

mkdir -p "$TMPDIR"
rm -f "$T_DATA_FILE"

fixt() {
  local f="$1"; shift
  T_DATA_FILE="$FIXTURES/$f" "$@"
}

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

assert_diff() {
  local expected="$FIXTURES/$1"; shift
  shift # --
  if [ -e "$expected" ]; then
    echo T "$@"
    local actual="$($COMMAND "$@")"
    if [ "$actual" != "$(cat "$expected")" ]; then
      echo FAIL
      echo "$actual" | diff -u "$expected" -
      exit 1
    fi
    echo PASS
  else
    echo T "$@"
    "$COMMAND" "$@" > "$expected"
    echo RECORDED
  fi
}

assert_out "NOT working" -- status
assert_out "You haven't started working yet" -- stop

assert_out "Starting work" -- start
assert_out "You already started working," -- start

assert_out "WORKING" -- status
assert_out "WORKING (0)" -- status --with-week

assert_out "You just worked for " -- stop
assert_out "You haven't started working yet" -- stop

assert_out "NOT working" -- status

fixt 2013-09.csv \
  assert_diff 2013-09.all -- all

fixt 2013-09.csv \
  assert_diff 2013-09.days -- days

fixt 2013-11.csv \
  assert_diff 2013-11.days -- days

TODAY="$(date +%Y-%m-%d)"
EDITOR="(echo $TODAY 00:00,$TODAY 01:00; echo $TODAY 01:45, $TODAY 02:55) | tee" \
  $COMMAND edit >/dev/null

assert_out "You have worked for 130 minutes today." -- today
assert_out "You have worked for 130 minutes since " -- week
