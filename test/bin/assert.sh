#!/usr/bin/env bash

_bold=$(tput bold)
_clear=$(tput sgr0)

# read a STDIN (usually a HEREDOC) into a variable
read_eof () {
  if [ -z "$1" ]; then
    echo "*** Expected variable argument; use: read_eof <into>"
    exit 1
  fi
  local data=""
  set +e
  while IFS= read line; do
    if [ -z "$data" ]; then
      data="$line"
    else
      printf -v data "${data}\n${line}"
    fi
  done
  set -e
  printf -v "$1" "$data"
}

# stacktrace
stacktrace() {
  local last=$((${#FUNCNAME[@]} - 1))
  for (( i=last; i >= 0; i-- )); do
    local line_idx=$((i - 1))
    if (( line_idx >= 0 )); then
      printf '#%d %s:%s\n' $i "${FUNCNAME[i]}" "${BASH_LINENO[line_idx]}"
    else
      printf '#%d %s\n' $i "${FUNCNAME[i]}"
    fi
  done
}

# caller location: the function and line number of the outermost test script
current_line() {
  local last=$((${#FUNCNAME[@]} - 1))
  local line_idx=$((last - 1))
  if (( line_idx >= 0 )); then
    printf '%s:%s' "${FUNCNAME[last]}" "${BASH_LINENO[line_idx]}"
  else
    printf '%s' "${FUNCNAME[last]}"
  fi
}

# run a command and assert a particular exit status
assert_status () {
  if [ -z "$1" ]; then
    echo "assert: no status specified; use: assert_status <status> <command>"
    exit 1
  fi
  expect=$1; shift
  set +e
  $*
  actual=$?
  set -e
  if [[ "$expect" -ne "$actual" ]]; then
    echo -e "assert: not equal @ $(current_line)\n    expected: $expect\n         got: $actual"
    exit 1
  fi
}

# assert an empty string
assert_empty () {
  if [ ! -z "$1" ]; then
    echo "assert: not empty @ $(current_line) [$1]"
    exit 1
  fi
}

# assert an strings equal
assert_equal () {
  if [ "$1" != "$2" ]; then
    expect=$(escape_ws "$1")
    actual=$(escape_ws "$2")
    echo -e "assert: not equal @ $(current_line)\n    expected: ${_bold}{${_clear}$expect${_bold}}${_clear}\n         got: ${_bold}{${_clear}$actual${_bold}}${_clear}"
    exit 1
  fi
}

# assert substring presence in string
# usage: assert_contains substring string
assert_contains () {
  if [[ "$2" != *"$1"* ]]; then
    echo -e "assert: does not contain @ $(current_line)\n    not found: ${_bold}{${_clear}$1${_bold}}${_clear}\n    in: ${_bold}{${_clear}$2${_bold}}${_clear}"
    exit 1
  fi
}

# assert that all lines are present, regardless of order
# usage: assert_all_lines_present EXPECT ACTUAL
# example: assert_all_lines_present "first"$'\n'"second" "second"$'\n'"first"
assert_all_lines_present () {
  wantSorted=$(printf '%s\n' "$1" | sort)
  gotSorted=$(printf '%s\n' "$2" | sort)

  if [ "${wantSorted}" != "${gotSorted}" ]; then
    echo -e "assert: not all lines are present @ $(current_line)\n    expected: ${_bold}{${_clear}${wantSorted}${_bold}}${_clear}\n         got: ${_bold}{${_clear}${gotSorted}${_bold}}${_clear}"
    exit 1
  fi
}

# replace whitespace with visible characters or escape sequences
# SP  0x20 =  · (U+00B7 Middle Dot)
# TAB 0x09 = \t
# CR  0x0D = \r
# LF  0x0A = \n
escape_ws () {
  echo "$@" | sed 's/ /·/g;s/\t/\\\\t/g;s/\r/\\\\r/g;s/$/\\\\n/g'
}
