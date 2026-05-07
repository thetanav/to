# AI Coding Era TODO Tool Improvement Plan

> Strategic plan to make this tool competitive in the AI coding era (2025-2026+)

## Strategic Focus

**AI Workflow Orchestration** - Position as the bridge between task tracking and AI coding assistants (Claude Code, Aider, Cline).

**Target Users**: Both solo developers and development teams.

**Integration Priority**: CLI callable (priority over MCP).

---

## Executive Summary

The AI coding tool landscape in 2026 is dominated by three tools:

- **Cursor** ($20/mo) - Best inline completion, VS Code fork
- **Claude Code** ($20/mo) - Best autonomous agent, terminal-native
- **Windsurf** ($15/mo) - Best value, Cascade agent

All three share a critical gap: no standalone, offline-first CLI TODO tool that integrates natively with their workflows.

This tool fills that gap by being:

1. CLI-native - Works from terminal, no GUI required
2. MCP-ready - Exposes tasks to AI assistants
3. AI-optimized - Exports context optimized for LLM consumption
4. Offline-first - Works without internet
5. Git-integrated - Branch-per-task workflow built-in

---

## Phase 1: Core AI Workflow Features

### 1.1 MCP Server Implementation

**Why**: Claude Code, Cursor, Windsurf, and Cline all support MCP. This makes TODO globally accessible.

```json
// .mcp.json configuration
{
  "mcpServers": {
    "todo": {
      "command": "npx",
      "args": ["-y", "@the-tanav/to-mcp"],
      "env": { "TODO_PATH": ".todo" }
    }
  }
}
```

**Tools to expose**:
- `list_todos` - Return pending/completed items
- `create_todo` - Add new task with title, description, priority
- `update_todo` - Modify status, priority, description
- `delete_todo` - Remove completed tasks
- `get_next_task` - Return highest priority pending task

**Storage**: SQLite (offline-first, portable)

### 1.2 CLI Callable from AI Tools

Make the tool executable and callable from AI assistants:

```bash
# From Aider
/run todo add "Implement auth feature"
/run todo ls --pending
/run todo done 3

# From Claude Code
!to add "Refactor API layer"

# From Cline (internal)
/run to scan --git-only
```

**Requirements**:
- Single binary distribution (no installation steps)
- Environment-agnostic (works in any shell)
- Exit codes for automation

### 1.3 CLAUDE.md Integration

Add auto-generated export for AI context:

```bash
# Generate AI-optimized task summary
to export --ai

# Output:
## Current Tasks

### Priority: High
1. [ ] Fix auth bug - Blocking; affects login flow
2. [ ] Add API rate limiting - Performance concern

### Priority: Medium
3. [ ] Update documentation - Can defer

### Blocked Tasks
- Task 4: Refactor DB schema (waiting on #2)

## Notes
- Run tests before marking done: to test
- Use branch: feature/auth-fix
```

**Why**: Claude Code loads CLAUDE.md every session. Include `to export --ai` output automatically.

---

## Phase 2: Enhanced Task Management

### 2.1 Task Dependencies & Subtasks

```bash
# Add task with dependencies
to add "Fix auth bug" --blocks 3
to add "Review PR" --depends-on 5

# Add subtask
to add "Add unit tests" --parent 1

# View dependency tree
to tree 1

# Blocked tasks shown automatically
to ls --blocked
```

**Schema**:
```
[ ] Parent task
  ├── [ ] Subtask A
  └── [X] Subtask B
[ ] Task blocked by #3
```

### 2.2 Branch-Per-Task Workflow

Enhance `to do` to create branches automatically:

```bash
# Creates feature/task-description-from-todo
to do 1 --create-branch

# Full workflow:
to do 1 --create-branch --commit "Fix: task description"

# Branch template: feature/{task-slug}
```

### 2.3 Priority & Due Dates

```bash
# Add with metadata
to add "Critical fix" --priority high --due today
to add "Feature" --priority medium --due next-week
to add "Nice to have" --priority low --due someday

# Filters
to ls --priority high
to ls --overdue
to ls --due today
to ls --due this-week

# Automatic overdue highlighting
```

### 2.4 Labels & Categories

```bash
# Add labels
to add "API endpoint" --label backend --label api
to add "UI component" --label frontend

# Filter by label
to ls --label frontend
to ls --label backend,api
```

---

## Phase 3: Team & Multi-Tool Features

### 3.1 Git-Based Sync

Add git worktree support for parallel task work:

```bash
# Create worktree for task
to worktree add feature-a
# Result: new/feature-a/ (git worktree + branch)

# List active worktrees
to worktree ls

# Switch worktrees
to worktree switch feature-a
```

**Workflow**:
```
Main branch: .todo lists all tasks
Worktrees: Each feature in isolated directory
Sync: to sync pushes worktree changes
```

### 3.2 Export to External Trackers

Export tasks to Linear/Jira:

```bash
# Export to Linear
to export linear
# Creates issues in Linear, updates local .todo with external IDs

# Import from Linear
to import linear
# Creates TODO items from Linear issues
```

**Mappings**:
| TODO | Linear |
|------|--------|
| title | title |
| priority:high | priority:urgent |
| priority:medium | priority:high |
| priority:low | priority:low |
| --due | dueDate |

### 3.3 Shared TODO Format

JSON/YAML format for portability:

```bash
# Export as JSON
to export --json

# Export as YAML
to export --yaml

# Import
to import tasks.json
```

---

## Phase 4: Developer Experience

### 4.1 Terminal UI Enhancements

```bash
# Interactive selection
to next
# Opens fuzzy picker: select task to work on

# Interactive task preview
to preview 1
# Shows full task context, file references, git history

# Keyboard navigation (vim-style)
j/k - Next/prev task
Enter - Select task
q - Quit
```

### 4.2 Hooks & Automation

```bash
# .todo/hooks/pre-add
# Validation: require --label for tasks with --priority high

# .todo/hooks/post-done
# Auto-commit with "Done: task title"
# Notify Slack/Discord webhook

# .todo/hooks/pre-do
# Run tests before starting task
# Check branch is clean
```

**Hook directory structure**:
```
.todo/
  hooks/
    pre-add
    post-done
    pre-do
    post-do
  config.toml
```

### 4.3 Statistics & Reporting

```bash
# Basic stats
to stats
# Output:
# Completed: 23/45 (51%)
# This week: 8
# Average per day: 1.6
# Current streak: 5 days

# Weekly report
to report --weekly
# Markdown for standups

# Productivity chart
to chart --month
```

---

## Implementation Priority

| Priority | Feature | Complexity | Impact |
|----------|---------|-----------|---------|
| P0 | MCP Server | Medium | Universal AI tool access |
| P0 | CLI Callable | Low | Aider/Claude Code integration |
| P0 | `--ai` export | Low | CLAUDE.md workflow |
| P1 | Task dependencies | Medium | Complex task support |
| P1 | Git worktrees | High | Parallel workflows |
| P1 | Labels | Low | Categorization |
| P2 | Linear/Jira sync | High | Enterprise ready |
| P2 | Interactive TUI | Medium | UX polish |
| P2 | Hooks | Medium | Automation |

---

## Technical Decisions

### Language

**Current**: Rust (minimal std, no dependencies)

**Recommendation**: Keep Rust for CLI. MCP server can be:

1. **Rust-only**: Embed MCP server in same binary
   - Pros: Single binary, same language
   - Cons: More complex async handling

2. **Python sidecar**: `to mcp` runs Python server
   - Pros: Rich MCP Python ecosystem
   - Cons: Python dependency

**Decision**: Keep Rust-only initially. Add MCP in-process.

### Storage

**Current**: Plain text `.todo` file

**Options**:

1. **Keep text**: Simple, portable, git-mergeable
   - Keep for user-facing output

2. **SQLite**: Better query, concurrent access
   - Use for MCP server backing

**Decision**: Dual storage. Text for users, SQLite for MCP.

---

## Feature Comparison

| Feature | This Tool | Claude Code Tasks | Linear MCP |
|---------|-----------|------------------|------------|
| Offline-first | Yes | Partial | No |
| MCP integration | Planned | N/A | Yes |
| CLI-native | Yes | Yes | Via MCP |
| Branch-per-task | Yes | No | No |
| Git worktrees | Planned | No | No |
| Export for AI | Planned | Yes | Via CLAUDE.md |
| Team sync | Planned | No | Via API |

---

## Success Metrics

1. **MCP adoption**: Used by Claude Code, Cline, Aider without configuration
2. **CLI integration**: `to` callable from any AI assistant
3. **User growth**: X developers using for AI workflow
4. **Task volume**: X tasks managed via tool
5. **Integration count**: X external tools connected (Linear, Jira)

---

## Timeline

### Month 1: Core AI Features
- [ ] Add `--ai` export for CLAUDE.md
- [ ] Make CLI callable from Aider/Claude Code
- [ ] Basic MCP server implementation
- [ ] Priority and labels

### Month 2: Task Dependencies
- [ ] Task `--blocks` and `--depends-on`
- [ ] Subtask / parent-child
- [ ] `to tree` command
- [ ] `to ls --blocked`

### Month 3: Team Features
- [ ] Git worktrees
- [ ] Linear/Jira export
- [ ] JSON/YAML import/export
- [ ] Hooks system

### Month 4: Polish
- [ ] Interactive TUI
- [ ] Statistics
- [ ] Report generation
- [ ] Documentation

---

## Open Questions

1. Merge into OpenCode or keep separate CLI?
2. MCP first or CLI callable first?
3. Target team size (solo vs team)?
4. Enterprise features (SSO, hosted)?