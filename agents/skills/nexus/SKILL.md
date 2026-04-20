---
name: nexus
description: Coordinate with other agents on a project via tickets and an immutable note log. Use Nexus when told to work autonomously, manage your work as tickets, leave notes about completed work, or read the history left by other agents or your prior self.
---

# Nexus
Nexus is used to operate on the following things:

**Tickets**
* Create new tickets that describe work that you or another agent will do later,
* Find tickets that you can work on, that you are assigned, or filtered by some other criteria,
* Update the status of tickets that are in progress or completed.

**Notes**
* Store free form notes about work you have performed for other agents or your future self,
* List the history of notes that you can use to understand more about how work was done,
* You can update or delete notes in exceptional cases, but generally speaking, the history of notes should be immutable and additive.

## Using Nexus
Nexus is a command line tool. Usually, you invoke it as `nexus`. In some cases, usually for testing Nexus itself, you may be asked to invoke it by a different command, for example: `cargo run --`. You can use Nexus in the same way in either case.

## Discovering Commands and Getting Documentation
Nexus can provide documentation about itself. **When you first load this skill, run `nexus docs` once and read the full output.** The documentation is effectively an extension of this skill and contains the complete reference for every command and flag.

```
$ nexus docs
```

For brief help information, you can use:
```
$ nexus help
```

You can get details about any command by using the `--help` flag:
```
$ nexus ticket new --help
```

## Command Output
By default, output is formatted as JSON. When more than one record is produced by a single command there is one record per line. Sometimes there is also a summary line at the very end.

A text output option is also supported via `--format text`.

## The Project Context
When you use Nexus, you are operating on a specific project context. This context is identified by a path on disk. By default, this path is the root of the Git repository you are in, or failing that, the current directory. If you are given a specific project context, provide it to Nexus. Otherwise, you may allow Nexus to auto-discover the default repo project context. If for some reason you suspect the default project context will not be what the user intends, collect the project context explicitly from the user.

You can provide the project context path via the `--project` flag or by setting the environment variable `NEXUS_PROJECT=$path`; for example:
```
$ nexus --project $PROJECT_ROOT # additional commands...
```

## Your Role
When you use Nexus you may have a role. Your role will be assigned to you in your prompt. Your role tells you the type of work you can do and you must only ever do work assigned to your role.

**If you were not given a role in your prompt or in subsequent instructions, disregard roles entirely when interacting with Nexus.** Do not pass `--role` to any command, and do not filter by role. You may take and work on any ticket regardless of the roles it specifies.

## Your Identity
When you use Nexus you have an identity. This identity is your Agent ID, which is tied to your role. Your Agent ID is a random identifier that uniquely identifies you among other agents.

A "session" for this purpose is the lifetime of a single Claude Code conversation, beginning at the `SessionStart` hook and ending at the `SessionEnd` hook. At session start, generate a new Agent ID and use it for the duration of the session. **At session end, clean up by abandoning every ticket you still own** (see `nexus ticket abandon`). Never reuse an Agent ID across sessions.

The `--agent` flag should be provided at the top level and is required for all commands except `agent`.
```
$ nexus --agent $AGENT_ID ticket list
```

The `NEXUS_AGENT` environment variable is also supported. Note that in Claude Code each Bash invocation runs in a fresh subshell, so an `export` in one call does not carry over to the next. Your practical options are:
* Pass `--agent <id>` on every command (recommended), or
* Inline the env var per command: `NEXUS_AGENT=$AGENT_ID nexus ticket list`.

### Creating an Agent ID
To create a new Agent ID, use the command `agent new`. Provide your role (omit `--role` if you have no role) and remember the result.
```
$ nexus agent new --role reviewer
role: agent-reviewer-WRYOL6bXHYsMeI89
```

# Tickets
A ticket describes a well-defined unit of work that is worked on by a single agent at a time.

The following is an example of a JSON ticket record:
```json
{
  "id": 3,
  "state": "available",
  "roles": [
    "reviewer"
  ],
  "summary": "This is the second ticket we've created",
  "detail": "More detailed information about this ticket may be found here",
  "data": null,
  "owner_id": "example-agent",
  "references": null,
  "fence": 4,
  "created_at": "2026-04-19T17:23:15.453048Z",
  "updated_at": "2026-04-19T17:23:15.453096Z"
}
```

## Ticket Ownership
Only the owner of a ticket may work on it. If another agent already is the owner of a ticket, that agent is already working on it and you must not work on it also. If a ticket has no owner an agent can _take_ the ticket and start working on it.

## Ticket Roles
A ticket may only be suitable for agents with certain roles to work on. If this is the case, the ticket should indicate these roles and only agents that have one of the indicated roles should attempt to take or work on the ticket.

## Ticket States and Workflow
Tickets have a set of well-defined states and a ticket exists in exactly one of those states at any given time.

| State | Meaning |
|-------|---------|
| `available` | The ticket is available and ready to be worked on. An agent may take the ticket and work on it. |
| `in_progress` | The ticket is being worked on by an agent. |
| `done` | The ticket has been completed. |

The normal workflow for tickets is:

1. ticket created → `available` state
2. agent takes ticket → `in_progress` state
3. agent works on ticket ...
4. agent finishes ticket → `done` state.

It is the agent's responsibility to update the state of the ticket as it moves through this process by using the command `nexus ticket update`.

## Concurrency and Editing
Take, abandon, and update operations are all atomic. If two agents race to take the same ticket, only one will succeed; the other will receive an error and a non-zero exit status. When a take fails because the ticket is already owned, do not retry — pick a different ticket instead.

`take` and `abandon` use the ticket's owner state for atomicity and do **not** require a fencing token: only one agent (the owner) can ever abandon a given ticket, and only one agent can win a race to take an unowned one. You only need to think about fencing for `update`.

If a ticket has an owner, only that owner may update it. If it has no owner, any agent may update it, but updates from concurrent agents must be serialized. In either case this is accomplished using a **fencing token** that ensures the update is applied to the intended state. When updating a ticket, you must provide the current fencing token via the `--fence` flag. The current token is found in the `fence` property of the ticket.

You will always already have the fencing token when you go to update a ticket, because you obtained it when you read the ticket. Every `nexus ticket get` and `nexus ticket list` response includes the `fence` value for each ticket; you should never be in a position where you are updating a ticket you have not first seen.

The token advances with every successful state-changing operation on the ticket — `take`, `abandon`, and `update` all increment it. So if you read a ticket, then perform a `take`, the `fence` value you read is already stale; re-read the ticket before updating.

The update will fail if **any** of the following is true:

* The ticket has an owner and the agent attempting the update is not that owner,
* No fencing token was provided,
* A fencing token was provided but does not match the current value.

If an update fails because of an invalid fencing token, the ticket has been changed by another agent (or by a take/abandon you forgot about) since you last read it. You should fetch the ticket again to read the new `fence` value, decide whether your edit is still appropriate in light of the new state, and then either retry the update with the new token or abandon the change. The decision to continue at all is yours; do not assume a retry is always the right move.

## Review Workflow
Whether the project uses a review workflow should be communicated to you by the user. If it is not explicitly stated, infer it as follows:
* If you were given a role, assume a review workflow **is** in use.
* If you have no role, assume there is **no** review workflow.

When a review workflow is used, the agent that completes a ticket must create a _new_ ticket to review the work it has done. It must mention the identifier of the ticket that needs review, include any relevant information about the work in the ticket description, and assign the ticket to a reviewer role. A reviewer agent should take the ticket, perform its review, and open new tickets for any cleanup work that is required as a result.

Tickets never move back into an `available` or `in_progress` state as a result of a review. Instead, new tickets are created which mention any previous tickets they relate to. In this way you can follow the chain back to the original work.

## Example: Ticket Workflow
A typical end-to-end flow for a worker agent. Replace `$AGENT_ID` with the value returned by `nexus agent new`.

```bash
# Find an available ticket that matches my role
nexus --agent $AGENT_ID ticket list --available --role reviewer

# Take a ticket (no --fence needed; owner state provides atomicity)
nexus --agent $AGENT_ID ticket take --id 1

# Re-read the ticket to obtain the current fencing token
FENCE=$(nexus --agent $AGENT_ID ticket get --id 1 | jq -r '.fence')

# Move it into progress when you start work, passing the fence
nexus --agent $AGENT_ID ticket update --id 1 --state in_progress --fence $FENCE

# A successful update advances the fence; re-read it before the next update
FENCE=$(nexus --agent $AGENT_ID ticket get --id 1 | jq -r '.fence')

# Mark it done when complete
nexus --agent $AGENT_ID ticket update --id 1 --state done --fence $FENCE

# If you cannot finish, abandon it so another agent may take it (no --fence needed)
nexus --agent $AGENT_ID ticket abandon --id 1

# Create a follow-up ticket (for example, a review ticket referencing #1)
nexus --agent $AGENT_ID ticket new \
  --role reviewer \
  --summary "Review work completed for ticket #1" \
  --detail - <<EOF
Ticket #1 has been completed; please review the changes.
EOF
```

# Notes
A note is a free form artifact that allows agents to record valuable information for other agents or their future selves. Notes should be preferred over writing markdown files to communicate this sort of information because notes are better organized, searchable, and don't need to be cleaned up.

A note can refer to a specific commit in the repo (use `--commit $COMMIT_SHA`) or it can refer to the project in general. In any case, the creator of the note is recorded.

The following is an example of a JSON note record:
```json
{
  "id": 2,
  "creator_id": "example-agent",
  "commit_sha": "0d44171577d4d48bfff65b604f90412364653e67",
  "summary": "Remember to read The Raven, by E.A. Poe",
  "detail": "Once upon a midnight dreary, while I pondered, weak and weary,\nOver many a quaint and curious volume of forgotten lore—\n    While I nodded, nearly napping, suddenly there came a tapping,\nAs of some one gently rapping, rapping at my chamber door.\n“’Tis some visitor,” I muttered, “tapping at my chamber door—\n            Only this and nothing more.”\n",
  "data": null,
  "created_at": "2026-04-19T17:25:29.924629Z",
  "updated_at": "2026-04-19T17:25:29.924705Z"
}
```

## Example: Note Workflow
```bash
# Leave a note tied to the current commit, with the body coming from STDIN
nexus --agent $AGENT_ID note new \
  --summary "Refactored auth middleware to meet new compliance requirements" \
  --commit $(git rev-parse HEAD) \
  --detail - <<EOF
Replaced session token storage. The legacy middleware in src/auth/legacy.rs
has been removed; downstream callers now use src/auth/session.rs.
EOF

# Leave a project-level note (no commit)
nexus --agent $AGENT_ID note new \
  --summary "Investigation: ingest pipeline drops events under load"

# List every note (summary only by default; add --verbose for full detail)
nexus --agent $AGENT_ID note list
nexus --agent $AGENT_ID --verbose note list

# Filter by commit, by creator, or by time window
nexus --agent $AGENT_ID note list --commit $(git rev-parse HEAD)
nexus --agent $AGENT_ID note list --creator agent-reviewer-WRYOL6bXHYsMeI89
nexus --agent $AGENT_ID note list --created-after 2026-04-01T00:00:00Z

# Fetch a specific note by id
nexus --agent $AGENT_ID note get --id 2
```
