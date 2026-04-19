---
name: nexus
description: Nexus is a tool for agents to coordinate with each other on a project and to record a history of their work. Use Nexus when you need to find tickets to work, record notes about the work you have done, or read the history of notes left by other agents or your prior self.
---

Nexus is used to operate on the following things:

## Tickets
* Create new tickets that describe work that you or another agent will do later,
* Find tickets that you can work on, that you are assigned, or filtered by some other criteria,
* Update the status of tickets that are in progress or completed.

## Notes
* Store free form notes about work you have performed for other agents or your future self,
* List the history of notes that you can use to understand more about how work was done,
* You can update or delete notes in exceptional cases, but generally speaking, the history of notes should be immutable and additive.

# Using Nexus
Nexus is a command line tool. Usually, you invoke it as `nexus`. In some cases, usually for testing Nexus itself, you may be asked to invoke it by a different command, for example: `cargo run --`. You can use Nexus in the same way in either case.

## Discovering Commands and Getting Documentation
Nexus can provide documentation about itself. To read comprehensive documentation about all of Nexus' commands, run:
```
$ nexus docs
```

For brief help information about commands, you can use:
```
$ nexus help
```

You can get details about any command by using the `--help` flag:
```
$ nexus ticket new --help
```

## Command Output
By default, output is formatted as JSON. When more than one record is produced by a single command there is one record per line. Sometimes there is also a summary line at the very end.

A text output option is also supported.

## The Project Context
When you use Nexus, you are operating on a specific project context. This context is identified by a path on disk. By default, this path is the root of the Git repository you are in, or failing that, the current directory. If you are given a specific project context, provide it to Nexus. Otherwise, you may allow Nexus to auto-discover the default repo project context. If for some reason you suspect the default project context will not be what the user intends, collect the project context explicitly from the user.

You can provide the project context path via the `--project` flag or by setting the environment variable `NEXUS_PROJECT=$path`; for example:
```
$ nexus --project $PROJECT_ROOT # additional commands...
```

## Your Role
When you use Nexus to manage tickets or notes you will have a role. Your role will be assigned to you in the prompt. Your role tells you the type of work you can do and you must only ever do work assigned to your role.

If you were not explicitly given a role in your prompt or in subsequent instructions, you have the special role `*`. Whether you have an explicitly provided role or the special default role `*` you should provide your role to Nexus anywhere it accepts a role as a parameter.

## Your Identity
When you use Nexus you have an identity. This identity is your Agent ID, which is tied to your role. Your Agent ID is a random identifier that uniquely identifies you among other agents. When you first start a session you will generate a new Agent ID and you will use it for the duration of the session. When the session ends, you will cleanup after this Agent ID and never use it again.

You can provide your agent identity via the `--agent` flag for commands that support this; or by setting the environment variable `NEXUS_AGENT=$agent_id`, for example:
```
$ nexus note --agent $AGENT_ID # additional commands...
```

### Creating an Agent ID
To create a new Agent ID, use the command `agent new`. Provide your role and remember the result.
```
$ nexus agent new --role reviewer
role: agent-reviewer-WRYOL6bXHYsMeI89
```

When you interact with future commands, provide your identifier to every command that accepts it; for example:
```
$ nexus ticket --agent agent-reviewer-WRYOL6bXHYsMeI89 list
```

You may also set your Agent ID in the environment, but make sure that it does not live longer than the session:
```
$ NEXUS_AGENT=$agent_id nexus ticket list
```

# Tickets
A ticket describes a well-defined unit of work that is worked on by a single agent at a time.

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

It is the agent's responsibility to update the state of the ticket as it move through this process by using the command `nexus ticket update`.

### Review Workflow
It is common for a reviewer role to validate that tickets in the `done` state meet the project's standard. When a review workflow is used, the agent that completes a ticket must create a _new_ ticket to review the work it has done. It must mention the identifier of the ticket that needs review, include any relevant information about the work in the ticket description, and assign the ticket to a reviewer role. A reviewer agent should take the ticket, perform its review, and open new tickets for any cleanup work that is required as a result.

Tickets never move back into an `available` or `in_progress` state as a result of a review. Instead, new tickets are created which mention any previous tickets they relate on. In this way you can follow the chain back to the original work.

# Notes
A note is a free form artifact that allows agents to record valuable information for other agents or their future selves. Notes should be preferred over leaving markdown files for a similar purpose because notes are better organized, searchable, and don't need to be cleaned up.

A note can refer to a specific commit in the repo (use `--commit $COMMIT_SHA`) or it can refer to the project in general. In any case, the creator of the note is recorded.
