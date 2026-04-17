#!/usr/bin/env bash

# where am i?
me="$0"
me_home=$(dirname "$0")
me_home=$(cd "$me_home" && pwd)

# import test suite setup
. "$me_home/_suite.sh"

# empty state to start
assert_equal $($NEXUS ticket list | jq -sca 'map(.id) | sort') '[]'

# add one ticket
actual="$($NEXUS ticket new --summary "This is the first ticket we've created" | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":null,"roles":null}
EOF
assert_equal "$expect" "$actual"

# add another ticket
actual="$($NEXUS ticket new --summary "This is the second ticket we've created" --role "reviewer" | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list all tickets
actual="$($NEXUS ticket list | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":null,"roles":[]}
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list filtered by role
actual="$($NEXUS ticket list --role reviewer | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list filtered by multiple roles
actual="$($NEXUS ticket list --role reviewer --role nonexistent | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list all available tickets
actual="$($NEXUS ticket list --available | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":null,"roles":[]}
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# take a ticket
actual="$($NEXUS ticket take --id 1 | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":"${NEXUS_AGENT}","roles":[]}
EOF
assert_equal "$expect" "$actual"

# list all available tickets
actual="$($NEXUS ticket list --available | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":"${NEXUS_AGENT}","roles":[]}
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# take a ticket (using another persona)
actual="$($NEXUS ticket --agent $another_agent take --id 2 | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":"${another_agent}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list all available tickets
actual="$($NEXUS ticket list --available | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":"${NEXUS_AGENT}","roles":[]}
EOF
assert_equal "$expect" "$actual"

# take a ticket that is owned by someone else, which is not allowed
assert_status 1 $NEXUS ticket take --id 2

# take a ticket by force
actual="$($NEXUS ticket take --id 2 --force | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":"${NEXUS_AGENT}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list all tickets belonging to us
actual="$($NEXUS ticket list --mine | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created","owner_id":"${NEXUS_AGENT}","roles":[]}
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":"${NEXUS_AGENT}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# update a ticket
actual="$($NEXUS ticket update --id 1 --role another --summary "This is the first ticket we've created, and then some" | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
EOF
assert_equal "$expect" "$actual"

# fetch our updated ticket
actual="$($NEXUS ticket get --id 1 | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
EOF
assert_equal "$expect" "$actual"

# fetch two at a time
actual="$($NEXUS ticket get --id 1 --id 2 | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":"${NEXUS_AGENT}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# put our status back
actual="$($NEXUS ticket update --id 1 --state available | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
EOF
assert_equal "$expect" "$actual"

# list all available tickets
actual="$($NEXUS ticket list --available | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"available","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":"${NEXUS_AGENT}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# start working on one
actual="$($NEXUS ticket update --id 1 --state in_progress | jq -scra '.[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"in_progress","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
EOF
assert_equal "$expect" "$actual"

# list all available tickets that already belong to us
actual="$($NEXUS ticket list --available --mine | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":"${NEXUS_AGENT}","roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# abandon one of our tickets
actual="$($NEXUS ticket abandon --id 2 | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":2,"state":"available","summary":"This is the second ticket we've created","owner_id":null,"roles":["reviewer"]}
EOF
assert_equal "$expect" "$actual"

# list all tickets belonging to us
actual="$($NEXUS ticket list --mine | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"in_progress","summary":"This is the first ticket we've created, and then some","owner_id":"${NEXUS_AGENT}","roles":["another"]}
EOF
assert_equal "$expect" "$actual"

# abandon all remaining tickets we own
actual="$($NEXUS ticket abandon | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
read_eof expect <<EOF
{"id":1,"state":"in_progress","summary":"This is the first ticket we've created, and then some","owner_id":null,"roles":["another"]}
EOF
assert_equal "$expect" "$actual"

# list all tickets belonging to us
actual="$($NEXUS ticket list --mine | jq -scra 'sort_by(.id) | .[] | {id, state, summary, owner_id, roles}')"
assert_equal "" "$actual"
