#!/usr/bin/env bash

set -eo pipefail

# where am i?
me="$0"
me_home=$(dirname "$0")
me_home=$(cd "$me_home" && pwd)

# import assert utils
. "$me_home/assert.sh"

NEXUS=${NEXUS:-nexus}
NEXUS_ROLE=${NEXUS_ROLE:-tester}
NEXUS_PROJECT=${NEXUS_PROJECT}

# parse arguments
set -- $(getopt p:r: $*)
for i; do
  case "$i"
  in
    -p)
      NEXUS_PROJECT="$2"; shift;
      shift;;
    -r)
      NEXUS_ROLE="$2"; shift;
      shift;;
    --)
      shift; break;;
  esac
done

export NEXUS_ROLE
export NEXUS_AGENT=$($NEXUS agent new --role $NEXUS_ROLE)

another_agent=$($NEXUS agent new --role another)

project_dir="$(mktemp -d)"
trap "rm -r ${project_dir}" EXIT
export NEXUS_PROJECT="${project_dir}"

echo "project: $NEXUS_PROJECT"
echo "agent: $NEXUS_AGENT"

# empty state to start
assert_equal $($NEXUS ticket list | jq -sc 'map(.id) | sort') '[]'

# add two tickets
actual="$($NEXUS ticket new --summary "This is the first ticket we've created" | jq -scr '.[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":1,"summary":"This is the first ticket we've created","owner_id":null,"roles":null}
EOF
assert_equal "$expect" "$actual"

actual="$($NEXUS ticket new --summary "This is the second ticket we've created" --role "reviewer" | jq -scr '.[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":2,"summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list all tickets
actual="$($NEXUS ticket list | jq -scr 'sort_by(.id) | .[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":1,"summary":"This is the first ticket we've created","owner_id":null,"roles":[]}
{"id":2,"summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list filtered by role
actual="$($NEXUS ticket list --role reviewer | jq -scr 'sort_by(.id) | .[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":2,"summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list filtered by multiple roles
actual="$($NEXUS ticket list --role reviewer --role nonexistent | jq -scr 'sort_by(.id) | .[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":2,"summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# take a ticket
actual="$($NEXUS ticket take --id 1 | jq -scr '.[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":1,"summary":"This is the first ticket we've created","owner_id":"${NEXUS_AGENT}","roles":[]}
EOF
assert_equal "$expect" "$actual"

# take a ticket (using another persona)
actual="$($NEXUS ticket --agent $another_agent take --id 2 | jq -scr '.[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":2,"summary":"This is the second ticket we've created","owner_id":"${another_agent}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# take a ticket that is owned by someone else, which is not allowed
assert_status 1 $NEXUS ticket take --id 2

# take a ticket by force
actual="$($NEXUS ticket take --id 2 --force | jq -scr '.[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
read_eof expect <<EOF
{"id":2,"summary":"This is the second ticket we've created","owner_id":"${NEXUS_AGENT}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"
