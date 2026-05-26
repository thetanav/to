# `to`

Release 0.1.5.

`to` is a small Rust CLI for project-scoped TODO lists. It walks up from your current working directory, finds the nearest `.todo` file, and operates on that task list automatically. `to init` also prepares `CLAUDE.md` and `AGENTS.md` so AI coding agents know to use the project TODO list.

## Install

```bash
cargo install tto
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
to ls --priority high
to ls --label backend
to add "implement streaming responses"
to add "critical API fix" --priority high --label backend --label api
to add "write unit tests" --parent 1
to done 1 2
to do 1 2
to do 1 2 -b feature/batch-work
to do 1 --create-branch
to uncheck 1 2
to scan
to rm 2 3
to next
to tree 1
```

## `.todo` format

```text
[ ] implement streaming responses
  [ ] write unit tests
[ ] add sqlite persistence @medium #storage
[x] setup CLI parser
```

Subtasks are stored as indented tasks using two spaces per nesting level. Priority and labels are stored as plain trailing tags: `@high`, `@medium`, `@low`, and `#label`.

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
Updated agent doc /projects/api/CLAUDE.md
Updated agent doc /projects/api/AGENTS.md

$ to add "implement streaming responses"
Added task 1: implement streaming responses

$ to add "add sqlite persistence" --priority medium --label storage
Added task 2: add sqlite persistence @medium #storage

$ to add "write unit tests" --parent 1
Added task 2: write unit tests

$ to add "critical API fix" --priority high --label backend --label api
Added task 4: critical API fix @high #backend #api

$ to ls
Tasks from /projects/api/.todo
1. [ ] implement streaming responses
2. [ ]   write unit tests
3. [ ] add sqlite persistence @medium #storage
4. [ ] critical API fix @high #backend #api
Open: 4  Done: 0

$ to tree 1
1. [ ] implement streaming responses
2. [ ]   write unit tests

$ to ls sqlite
Tasks from /projects/api/.todo
Filter: "sqlite"
3. [ ] add sqlite persistence @medium #storage
Matches: 1  Open: 1  Done: 0

$ to ls --priority high --label backend
Tasks from /projects/api/.todo
Priority: high
Labels: #backend
4. [ ] critical API fix @high #backend #api
Matches: 1  Open: 1  Done: 0

$ to next
Next task: 1. implement streaming responses

$ to do 1 2
# launches opencode with a prompt telling it to do tasks 1 and 2 and use `to` to inspect the todo list

$ to do 1 2 -b feature/batch-work
# switches to branch `feature/batch-work`, then launches opencode

$ to do 1 --create-branch
# creates or switches to `feature/implement-streaming-responses`, then launches opencode

$ to done 1 2
Completed task 1: implement streaming responses
Completed task 2: write unit tests

$ to uncheck 1 2
Unchecked task 1: implement streaming responses
Unchecked task 2: write unit tests

$ to scan
Added 2 tasks from git-tracked TODO comments.
```

## Notes

- `to` searches for `.todo` starting in the current directory and then each parent directory.
- If no `.todo` file is found, the command tells you to run `to init` in the project root.
- `to init` creates `.todo` when needed and adds an idempotent `to` usage section to `CLAUDE.md` and `AGENTS.md`.
- `to scan` only reads git version-controlled files.
- `to add "task text" --priority <high|medium|low> --label <label>` stores task metadata as `@priority` and `#label` tags.
- `to ls [query] --priority <high|medium|low> --label <label>` filters task text and metadata while preserving the original task numbers. Repeat `--label` or pass comma-separated labels to require multiple labels.
- `to add "task text" --parent <number>` inserts a subtask under the selected task, after any existing subtasks.
- `to tree <number>` shows a task and its nested subtasks without unrelated tasks.
- `to do <number> [number ...] [-b <branch-name>]` launches `opencode --prompt ...` from the `.todo` project root with the selected task numbers and a built-in agent prompt that tells the agent to use `to` to inspect the todo list. With `-b`, it switches to the named branch first, creating it when needed.
- `to do <number> [number ...] --create-branch` creates or switches to a generated task branch before launching `opencode`. Single-task branches use `feature/<task-slug>`, for example `feature/implement-streaming-responses`; multi-task branches use `feature/tasks-1-2`.
- `to done`, `to uncheck`, and `to rm` accept multiple task numbers.
- Task numbers are 1-based.
