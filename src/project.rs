use std::fs::File;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

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

    if path.exists() {
        return Err(AppError::AlreadyInitialized(path));
    }

    File::create(&path)?;
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
}
