#!/bin/bash
# This script checks that the Ruby and Rust programs behave the same.

set -e
set -o nounset
set -o pipefail

COMMAND="${COMMAND:-t}"
ROOT="$(cd "$(dirname "$0")"; pwd -P)"
FIXTURES="${ROOT}/tests/fixtures/integration"
TMPDIR="${ROOT}/tmp"
export T_DATA_FILE="${TMPDIR}/test-sh-t.csv"

mkdir -p "$TMPDIR"
rm -f "$T_DATA_FILE"

case "$MODE" in
  ruby)
    days_args=
    ;;
  *)
    # Assume we're using rust
    days_args=-s
    ;;
esac

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
  if ! echo "$actual" | grep -q -E "$expected"; then
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
assert_out "WORKING \\(0\\)" -- status --with-week

assert_out "You just worked for . minutes" -- stop
assert_out "You haven't started working yet|You stopped . minutes ago" -- stop

assert_out "NOT working" -- status

fixt 2013-09.csv \
  assert_diff 2013-09.all -- all

fixt 2013-09.csv \
  assert_diff 2013-09.days -- days $days_args

fixt 2013-11.csv \
  assert_diff 2013-11.days -- days $days_args

fixt 2013-09.csv \
  assert_diff 2013-09.pto -- pto

fixt 2013-11.csv \
  assert_diff 2013-11.pto -- pto

echo T edit
cat <<EOF > "${TMPDIR}/editor.rb"
File.open(ARGV[0], "w") do |f|
  today = Time.now.strftime("%Y-%m-%d")
  f.puts "#{today} 00:00,#{today} 01:00"
  f.puts "#{today} 01:45,#{today} 02:55"
end
EOF
EDITOR="ruby ${TMPDIR}/editor.rb" \
  $COMMAND edit >/dev/null
echo PASS

assert_out "You have worked for 130 minutes today." -- today
assert_out "You have worked for 130 minutes since " -- week

if [ "$COMMAND" != "bin/t" ]; then
  echo 'T now (tz)'
  real_zone="$(date +%z)"
  t_zone="$($COMMAND now | awk '{ print $3 }')"
  if [ "$real_zone" == "$t_zone" ]; then
    echo PASS
  else
    echo FAIL
    echo "expected TZ to be '${real_zone}' but it was '${t_zone}'."
    exit 1
  fi
fi
