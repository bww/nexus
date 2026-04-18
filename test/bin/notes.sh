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
actual="$($NEXUS note new --summary "Remember to read The Raven, by E.A. Poe" --commit fafafafafafa --detail - <<EOF | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}'
Once upon a midnight dreary, while I pondered, weak and weary,
Over many a quaint and curious volume of forgotten lore—
    While I nodded, nearly napping, suddenly there came a tapping,
As of some one gently rapping, rapping at my chamber door.
“’Tis some visitor,” I muttered, “tapping at my chamber door—
            Only this and nothing more.”
EOF
)"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe","detail":"Once upon a midnight dreary, while I pondered, weak and weary,\\nOver many a quaint and curious volume of forgotten lore\\u2014\\n    While I nodded, nearly napping, suddenly there came a tapping,\\nAs of some one gently rapping, rapping at my chamber door.\\n\\u201c\\u2019Tis some visitor,\\u201d I muttered, \\u201ctapping at my chamber door\\u2014\\n            Only this and nothing more.\\u201d\\n"}
EOF
assert_equal "$expect" "$actual"

# add another note
actual="$($NEXUS note new --summary "Remember to read Leaves of Grass, by Walt Whitman" --commit fbfbfbfbfbfb | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":2,"creator_id":"${NEXUS_AGENT}","commit_sha":"fbfbfbfbfbfb","summary":"Remember to read Leaves of Grass, by Walt Whitman","detail":null}
EOF
assert_equal "$expect" "$actual"

# add a third note
actual="$($NEXUS note --agent ${another_agent} new --summary "Remember to read Ode on a Grecian Urn, by John Keats" | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":3,"creator_id":"${another_agent}","commit_sha":null,"summary":"Remember to read Ode on a Grecian Urn, by John Keats","detail":null}
EOF
assert_equal "$expect" "$actual"

# fetch the note by identifier
actual="$($NEXUS note get --id 1 | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe","detail":"Once upon a midnight dreary, while I pondered, weak and weary,\\nOver many a quaint and curious volume of forgotten lore\\u2014\\n    While I nodded, nearly napping, suddenly there came a tapping,\\nAs of some one gently rapping, rapping at my chamber door.\\n\\u201c\\u2019Tis some visitor,\\u201d I muttered, \\u201ctapping at my chamber door\\u2014\\n            Only this and nothing more.\\u201d\\n"}
EOF
assert_equal "$expect" "$actual"

# list every note
actual="$($NEXUS note list | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe","detail":null}
{"id":2,"creator_id":"${NEXUS_AGENT}","commit_sha":"fbfbfbfbfbfb","summary":"Remember to read Leaves of Grass, by Walt Whitman","detail":null}
{"id":3,"creator_id":"${another_agent}","commit_sha":null,"summary":"Remember to read Ode on a Grecian Urn, by John Keats","detail":null}
EOF
assert_equal "$expect" "$actual"

# list every note after the first one
created_at="$($NEXUS note get --id 1 | jq -scr '.[] | .created_at')"
actual="$($NEXUS note list --created-after "${created_at}" | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":2,"creator_id":"${NEXUS_AGENT}","commit_sha":"fbfbfbfbfbfb","summary":"Remember to read Leaves of Grass, by Walt Whitman","detail":null}
{"id":3,"creator_id":"${another_agent}","commit_sha":null,"summary":"Remember to read Ode on a Grecian Urn, by John Keats","detail":null}
EOF
assert_equal "$expect" "$actual"

# list every note with detail
actual="$($NEXUS --verbose note list | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe","detail":"Once upon a midnight dreary, while I pondered, weak and weary,\\nOver many a quaint and curious volume of forgotten lore\\u2014\\n    While I nodded, nearly napping, suddenly there came a tapping,\\nAs of some one gently rapping, rapping at my chamber door.\\n\\u201c\\u2019Tis some visitor,\\u201d I muttered, \\u201ctapping at my chamber door\\u2014\\n            Only this and nothing more.\\u201d\\n"}
{"id":2,"creator_id":"${NEXUS_AGENT}","commit_sha":"fbfbfbfbfbfb","summary":"Remember to read Leaves of Grass, by Walt Whitman","detail":null}
{"id":3,"creator_id":"${another_agent}","commit_sha":null,"summary":"Remember to read Ode on a Grecian Urn, by John Keats","detail":null}
EOF
assert_equal "$expect" "$actual"

# list every note relating to a commit
actual="$($NEXUS note list --commit "fafafafafafa" | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":1,"creator_id":"${NEXUS_AGENT}","commit_sha":"fafafafafafa","summary":"Remember to read The Raven, by E.A. Poe","detail":null}
EOF
assert_equal "$expect" "$actual"

# list every note crated by another agent
actual="$($NEXUS note list --creator "${another_agent}" | jq -scra '.[] | {id, creator_id, commit_sha, summary, detail}')"
read_eof expect <<EOF
{"id":3,"creator_id":"${another_agent}","commit_sha":null,"summary":"Remember to read Ode on a Grecian Urn, by John Keats","detail":null}
EOF
assert_equal "$expect" "$actual"
