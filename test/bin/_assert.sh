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
  while IFS= read -r line; do
    if [ -z "$data" ]; then
      data="$line"
    else
      data="${data}"$'\n'"${line}"
    fi
  done
  set -e
  printf -v "$1" '%s' "$data"
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
    echo "assert: not equal @ $(current_line)"
    echo "    expected: $expect"
    echo "         got: $actual"
    exit 1
  fi
}

# assert an empty string
assert_empty () {
  if [ ! -z "$1" ]; then
    actual=$(echo -n "$1" | escape_dsp)
    echo "assert: not empty @ $(current_line) [${actual}]"
    exit 1
  fi
}

# assert an strings equal
assert_equal () {
  if [ "$1" != "$2" ]; then
    expect=$(echo -n "$1" | escape_dsp)
    actual=$(echo -n "$2" | escape_dsp)
    echo "assert: not equal @ $(current_line)"
    echo "    expected: ${_bold}{${_clear}${expect}${_bold}}${_clear}"
    echo "         got: ${_bold}{${_clear}${actual}${_bold}}${_clear}"
    exit 1
  fi
}

# assert substring presence in string
# usage: assert_contains substring string
assert_contains () {
  if [[ "$2" != *"$1"* ]]; then
    needle=$(echo -n "$1" | escape_dsp)
    haystack=$(echo -n "$2" | escape_dsp)
    echo "assert: does not contain @ $(current_line)"
    echo "    not found: ${_bold}{${_clear}${needle}${_bold}}${_clear}"
    echo "    in: ${_bold}{${_clear}${haystack}${_bold}}${_clear}"
    exit 1
  fi
}

# assert that all lines are present, regardless of order
# usage: assert_all_lines_present EXPECT ACTUAL
# example: assert_all_lines_present "first"$'\n'"second" "second"$'\n'"first"
assert_all_lines_present () {
  expect=$(printf '%s\n' "$1" | sort)
  actual=$(printf '%s\n' "$2" | sort)

  if [ "${expect}" != "${actual}" ]; then
    expect=$(echo -n "$expect" | escape_dsp)
    actual=$(echo -n "$actual" | escape_dsp)
    echo "assert: not all lines are present @ $(current_line)"
    echo "    expected: ${_bold}{${_clear}${expect}${_bold}}${_clear}"
    echo "         got: ${_bold}{${_clear}${actual}${_bold}}${_clear}"
    exit 1
  fi
}

# escape literal backslashes by doubling them, so content like `\u000a`
# survives any downstream escape interpretation; reads from STDIN
escape_esc () {
  sed 's/\\/\\\\/g'
}

# replace whitespace with visible characters or escape sequences
# SP  0x20 =  · (U+00B7 Middle Dot)
# TAB 0x09 = \t
# CR  0x0D = \r
# LF  0x0A = \n
escape_dsp () {
  sed 's/ /·/g; s/\t/\\t/g; s/\r/\\r/g; s/$/\\n/g; s/\\/\\\\/g'
}
