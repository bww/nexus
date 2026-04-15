#!/usr/bin/env bash

# where am i?
me="$0"
me_home=$(dirname "$0")
me_home=$(cd "$me_home" && pwd)

# import test suite setup
. "$me_home/_suite.sh"

# empty state to start
assert_equal $($NEXUS ticket list | jq -sc 'map(.id) | sort') '[]'

# add one note
actual="$($NEXUS note new --summary "Remember to read The Raven, by E.A. Poe" --commit fafafafafafa <<EOF | jq -scr '.[] | {id, creator_id, commit_sha, summary}'
Once upon a midnight dreary, while I pondered, weak and weary,
Over many a quaint and curious volume of forgotten lore—
    While I nodded, nearly napping, suddenly there came a tapping,
As of some one gently rapping, rapping at my chamber door.
“’Tis some visitor,” I muttered, “tapping at my chamber door—
            Only this and nothing more.”
EOF
)"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe"}
EOF
assert_equal "$expect" "$actual"

# fetch the note by identifier
actual="$($NEXUS note get --id 1 | jq -scr '.[] | {id, creator_id, commit_sha, summary}')"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe"}
EOF
assert_equal "$expect" "$actual"
