---
name: nexus
description: Nexus is a tool for agents to coordinate with each other on a project and to record a history of their work. Use Nexus when you need to find tickets to work, record notes about the work you have done, or read the history of notes left by other agents or your prior self.
---

Nexus id used to operate on the following things:

## Tickets
* Create new tickets that describe work that you or another agent will do later,
* Find tickets that you can work on, that you are assigned, or filtered by some other critera,
* Update the status of tickets that are in progress or completed.

## Notes
* Store free form notes about work you have performed for other agents or your future self,
* List the history of notes that you can use to understand more about how work was done,
* You can update or delete notes in exceptional cases, but generally speaking, the history of notes should be immutable and additive.

# Using Nexus
Nexus is a command line tool. Usually, you invoke it as `nexus`. In some cases, usually for testing Nexus itself, you may be asked to invoke it by a different command, for example: `cargo run --`. You can use Nexus in the same way in either case.

## Discovring Commands and Getting Documentation
Nexus can tell you about the commands it provides and the arguments they take. You can get help by running:
```
$ nexus help
```
You can get details about any command by adding `help` to the end:
```
$ nexus ticket help
```

## The Project Context
When you use Nexus, you are operating on a specific project context. This context is identifie by a path on disk. Unless you are asked to use a different project context, this path is the root of the project or repository you are working on. Nexus will manage a hidden folder in the project directory to store it's data.

Always provide the project context when you invoke Nexus. It is provided via the `--project` flag; for example:
```
$ nexus --project $PROJECT_ROOT # additional commands...
```

The project context should be generally clear, it's the root of the project. If for some reason you are not certain what the project context should be, collect it from the user.

## Your Role
When you use Nexus to manage tickets or notes you will have a role. Your role will be assigned to you in the prompt. Your role tells you the type of work you can do and you must only ever do work assigned to your role or you will cause problems for other agents that have different roles. If you were not explicitly given a role in your prompt or in subsequent instructions, you have the special role `*`. Whether you have an explicitly provided role or the special default role `*` you should provide your role to Nexus anywhere it accepts a role as a parameter.

## Your Identity
When you use Nexus you also have an identity. This identity is your Agent ID, which is tied to your role. Your Agent ID is a random identifier that uniquely identifies you among other agents. When you first start a session you will generate a new Agent ID and you will use it for the duration of the session. When the session ends, you will cleanup after this Agent ID and never use it again.

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
