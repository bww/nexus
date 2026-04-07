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

project_dir="$(mktemp -d)"
trap "rm -r ${project_dir}" EXIT
export NEXUS_PROJECT="${project_dir}"

echo "project: $NEXUS_PROJECT"
echo "agent: $NEXUS_AGENT"

# empty state to start
assert_equal $($NEXUS ticket list | jq -sc 'map(.id) | sort') '[]'

# add two tickets
$NEXUS ticket new --summary "This is the first ticket we've created"
$NEXUS ticket new --summary "This is the second ticket we've created"
read_eof expect <<EOF
{"id":1,"summary":"This is the first ticket we've created","owner_id":null,"roles":[]}
{"id":2,"summary":"This is the second ticket we've created","owner_id":null,"roles":[]}
EOF
assert_equal "$expect" "$($NEXUS ticket list | jq -scr 'sort_by(.id) | .[] | {id: .id, summary: .summary, owner_id: .owner_id, roles: .roles}')"
