# `to`

`to` is a small Rust CLI for project-scoped TODO lists. It walks up from your current working directory, finds the nearest `.todo` file, and operates on that task list automatically.

## Install

```bash
cargo install thetanav-to
```

That installs the executable as `to`, so you run it like:

```bash
to init
```

## Usage

```bash
to
to init
to ls
to ls branch
to add "implement streaming responses"
to done 1 2
to do 1 2
to do 1 2 -b feature/batch-work
to uncheck 1 2
to scan
to rm 2 3
to next
```

## `.todo` format

```text
[ ] implement streaming responses
[ ] add sqlite persistence
[x] setup CLI parser
```

## Scan TODO comments

`to scan` looks through git-tracked files in the current project and imports lines containing `TODO:` into `.todo`.

```rust
// TODO: add sqlite persistence
// TODO: implement streaming responses
```

## Example session

```text
$ to init
Initialized /projects/api/.todo

$ to add "implement streaming responses"
Added task 1: implement streaming responses

$ to add "add sqlite persistence"
Added task 2: add sqlite persistence

$ to ls
Tasks from /projects/api/.todo
1. [ ] implement streaming responses
2. [ ] add sqlite persistence
Open: 2  Done: 0

$ to ls sqlite
Tasks from /projects/api/.todo
Filter: "sqlite"
2. [ ] add sqlite persistence
Matches: 1  Open: 1  Done: 0

$ to next
Next task: 1. implement streaming responses

$ to do 1 2
# launches opencode with a prompt telling it to do tasks 1 and 2 and use `to` to inspect the todo list

$ to do 1 2 -b feature/batch-work
# switches to branch `feature/batch-work`, then launches opencode

$ to done 1 2
Completed task 1: implement streaming responses
Completed task 2: add sqlite persistence

$ to uncheck 1 2
Unchecked task 1: implement streaming responses
Unchecked task 2: add sqlite persistence

$ to scan
Added 2 tasks from git-tracked TODO comments.
```

## Notes

- `to` searches for `.todo` starting in the current directory and then each parent directory.
- If no `.todo` file is found, the command tells you to run `to init` in the project root.
- `to scan` only reads git version-controlled files.
- `to ls [query]` filters task text while preserving the original task numbers.
- `to do <number> [number ...] [-b <branch-name>]` launches `opencode --prompt ...` from the `.todo` project root with the selected task numbers and a built-in agent prompt that tells the agent to use `to` to inspect the todo list. With `-b`, it switches to the named branch first, creating it when needed.
- `to done`, `to uncheck`, and `to rm` accept multiple task numbers.
- Task numbers are 1-based.
