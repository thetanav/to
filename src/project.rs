use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

const AGENT_TODO_SECTION: &str = "\
<!-- to:todo-instructions:start -->
## Project TODOs

This project uses the `to` CLI for project-scoped task tracking.

- Run `to ls` to inspect current tasks before starting work.
- Run `to tree <number>` to inspect a task and its subtasks.
- Run `to add \"task text\"` to add follow-up work.
- Run `to add \"task text\" --parent <number>` to add subtasks.
- Use `--priority <high|medium|low>` and `--label <label>` for task metadata.
- Run `to done <number>` when a task is complete.
- Prefer `to do <number> --create-branch` when starting agent-driven work.
<!-- to:todo-instructions:end -->
";

pub fn find_todo_file(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let candidate = current.join(".todo");
        if candidate.is_file() {
            return Ok(candidate);
        }

        if !current.pop() {
            return Err(AppError::TodoNotFound(start.to_path_buf()));
        }
    }
}

pub fn init_todo_file(dir: &Path) -> Result<PathBuf> {
    let path = dir.join(".todo");

    if !path.exists() {
        File::create(&path)?;
    }

    Ok(path)
}

pub fn ensure_agent_docs(dir: &Path) -> Result<Vec<PathBuf>> {
    ["CLAUDE.md", "AGENTS.md"]
        .into_iter()
        .map(|file_name| ensure_agent_doc(dir, file_name))
        .collect()
}

fn ensure_agent_doc(dir: &Path, file_name: &str) -> Result<PathBuf> {
    let path = dir.join(file_name);
    if !path.exists() {
        fs::write(&path, AGENT_TODO_SECTION)?;
        return Ok(path);
    }

    let existing = fs::read_to_string(&path)?;
    if existing.contains("<!-- to:todo-instructions:start -->") {
        return Ok(path);
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    if !updated.is_empty() {
        updated.push('\n');
    }
    updated.push_str(AGENT_TODO_SECTION);
    fs::write(&path, updated)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("to-{name}-{unique}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn finds_parent_todo_file() {
        let temp = TempDir::new("project-search");
        let project = temp.path.join("workspace");
        let nested = project.join("service").join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(project.join(".todo"), "[ ] parent task\n").unwrap();

        let found = find_todo_file(&nested).unwrap();
        assert_eq!(found, project.join(".todo"));
    }

    #[test]
    fn init_todo_file_is_idempotent() {
        let temp = TempDir::new("init-idempotent");
        let first = init_todo_file(&temp.path).unwrap();
        let second = init_todo_file(&temp.path).unwrap();
        assert_eq!(first, second);
        assert!(first.is_file());
    }

    #[test]
    fn ensure_agent_docs_creates_claude_and_agents_docs() {
        let temp = TempDir::new("agent-docs");
        let docs = ensure_agent_docs(&temp.path).unwrap();
        assert_eq!(
            docs,
            vec![temp.path.join("CLAUDE.md"), temp.path.join("AGENTS.md")]
        );

        let claude = fs::read_to_string(temp.path.join("CLAUDE.md")).unwrap();
        let agents = fs::read_to_string(temp.path.join("AGENTS.md")).unwrap();
        assert!(claude.contains("Run `to ls`"));
        assert!(agents.contains("to do <number> --create-branch"));
    }

    #[test]
    fn ensure_agent_docs_does_not_duplicate_section() {
        let temp = TempDir::new("agent-docs-existing");
        fs::write(temp.path.join("CLAUDE.md"), "# Claude\n").unwrap();

        ensure_agent_docs(&temp.path).unwrap();
        ensure_agent_docs(&temp.path).unwrap();

        let claude = fs::read_to_string(temp.path.join("CLAUDE.md")).unwrap();
        assert_eq!(
            claude
                .matches("<!-- to:todo-instructions:start -->")
                .count(),
            1
        );
        assert!(claude.starts_with("# Claude\n\n"));
    }
}
