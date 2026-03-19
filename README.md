# `to`

`to` is a small Rust CLI for project-scoped TODO lists. It walks up from your current working directory, finds the nearest `.todo` file, and operates on that task list automatically.

## Usage

```bash
to
to init
to ls
to add "implement streaming responses"
to done 1
to uncheck 1
to scan
to rm 2
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

$ to next
Next task: 1. implement streaming responses

$ to done 1
Completed task 1: implement streaming responses

$ to uncheck 1
Unchecked task 1: implement streaming responses

$ to scan
Added 2 tasks from git-tracked TODO comments.
```

## Notes

- `to` searches for `.todo` starting in the current directory and then each parent directory.
- If no `.todo` file is found, the command tells you to run `to init` in the project root.
- `to scan` only reads git version-controlled files.
- Task numbers are 1-based.
