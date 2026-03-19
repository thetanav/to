use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, Result};

pub fn scan_project(project_root: &Path) -> Result<Vec<String>> {
    let git_root = git_root(project_root)?;
    let scope = project_root
        .strip_prefix(&git_root)
        .unwrap_or_else(|_| Path::new(""));
    let tracked_files = tracked_files(&git_root, scope)?;
    let mut tasks = Vec::new();

    for relative_path in tracked_files {
        let absolute_path = git_root.join(&relative_path);
        if !absolute_path.starts_with(project_root) || !absolute_path.is_file() {
            continue;
        }

        if absolute_path.file_name().and_then(|name| name.to_str()) == Some(".todo") {
            continue;
        }

        tasks.extend(scan_file(&absolute_path, project_root)?);
    }

    Ok(tasks)
}

fn scan_file(path: &Path, project_root: &Path) -> Result<Vec<String>> {
    let bytes = fs::read(path)?;
    if bytes.contains(&0) {
        return Ok(Vec::new());
    }

    let contents = String::from_utf8_lossy(&bytes);
    let relative_path = path.strip_prefix(project_root).unwrap_or(path);
    let mut tasks = Vec::new();

    for (line_number, line) in contents.lines().enumerate() {
        if let Some((_, todo)) = line.split_once("TODO:") {
            let todo = todo.trim();
            if !todo.is_empty() {
                tasks.push(format!(
                    "{} ({}:{})",
                    todo,
                    relative_path.display(),
                    line_number + 1
                ));
            }
        }
    }

    Ok(tasks)
}

fn git_root(project_root: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(AppError::NotGitRepository(project_root.to_path_buf()));
        }

        return Err(AppError::GitCommandFailed(format!(
            "failed to resolve git root: {}",
            stderr.trim()
        )));
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return Err(AppError::GitCommandFailed(
            "git returned an empty repository root".to_string(),
        ));
    }

    Ok(PathBuf::from(root))
}

fn tracked_files(git_root: &Path, scope: &Path) -> Result<Vec<PathBuf>> {
    let mut command = Command::new("git");
    command.arg("-C").arg(git_root).arg("ls-files").arg("-z");

    if !scope.as_os_str().is_empty() {
        command.arg("--").arg(scope);
    }

    let output = command.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::GitCommandFailed(format!(
            "failed to list tracked files: {}",
            stderr.trim()
        )));
    }

    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| PathBuf::from(String::from_utf8_lossy(entry).into_owned()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let path = std::env::temp_dir().join(format!("to-scan-{name}-{unique}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn run_git(path: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .unwrap();
        assert!(status.success(), "git command failed: {:?}", args);
    }

    #[test]
    fn scans_only_git_tracked_files() {
        let temp = TempDir::new("tracked-only");
        fs::write(temp.path.join(".todo"), "").unwrap();
        fs::create_dir_all(temp.path.join("src")).unwrap();

        run_git(&temp.path, &["init"]);
        fs::write(
            temp.path.join("src/lib.rs"),
            "// TODO: tracked task\nfn main() {}\n",
        )
        .unwrap();
        fs::write(
            temp.path.join("notes.txt"),
            "TODO: should not be scanned because it is untracked\n",
        )
        .unwrap();

        run_git(&temp.path, &["add", ".todo", "src/lib.rs"]);

        let tasks = scan_project(&temp.path).unwrap();
        assert_eq!(tasks, vec!["tracked task (src/lib.rs:1)".to_string()]);
    }

    #[test]
    fn returns_clear_error_outside_git_repo() {
        let temp = TempDir::new("not-git");
        fs::write(temp.path.join(".todo"), "").unwrap();

        let error = scan_project(&temp.path).unwrap_err();
        assert!(matches!(error, AppError::NotGitRepository(_)));
    }
}
