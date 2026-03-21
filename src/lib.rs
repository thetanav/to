mod cli;
mod error;
mod project;
mod scan;
mod todo;

use std::collections::HashSet;
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command as ProcessCommand;

use cli::Command;
pub use error::{AppError, Result};
use project::{find_todo_file, init_todo_file};
use scan::scan_project;
use todo::TodoList;

const OPENCODE_AGENT_PROMPT: &str = "\
You are working from the project's `.todo` list. Inspect the codebase before making changes, \
complete this task end-to-end in the current repository, run relevant checks when practical, \
and report what changed plus any follow-up work.";

pub fn run() -> Result<()> {
    let command = cli::parse_args(env::args_os().skip(1))?;
    let cwd = env::current_dir()?;
    let mut stdout = io::stdout();
    let use_color = stdout.is_terminal() && env::var_os("NO_COLOR").is_none();
    execute(
        command,
        &cwd,
        &mut stdout,
        use_color,
        launch_opencode,
        switch_to_task_branch,
    )
}

fn execute<W, F, G>(
    command: Command,
    cwd: &Path,
    writer: &mut W,
    use_color: bool,
    mut run_opencode: F,
    mut switch_branch: G,
) -> Result<()>
where
    W: Write,
    F: FnMut(&Path, &str) -> Result<()>,
    G: FnMut(&Path, &str) -> Result<String>,
{
    match command {
        Command::Help => {
            writer.write_all(cli::HELP_TEXT.as_bytes())?;
        }
        Command::Init => {
            let path = init_todo_file(cwd)?;
            writeln!(
                writer,
                "{} {}",
                styled(use_color, "1;34", "Initialized"),
                path.display()
            )?;
        }
        other => {
            let todo_path = find_todo_file(cwd)?;
            let mut todos = TodoList::load(&todo_path)?;

            match other {
                Command::List(query) => {
                    write_task_list(writer, &todo_path, &todos, query.as_deref(), use_color)?
                }
                Command::Add(text) => {
                    let index = todos.add(text)?;
                    let task = &todos.tasks()[index - 1];
                    todos.save(&todo_path)?;
                    writeln!(
                        writer,
                        "{} task {index}: {}",
                        styled(use_color, "36", "Added"),
                        task.text
                    )?;
                }
                Command::Done(indices) => {
                    let indices = validate_task_indices(&todos, &indices)?;
                    let mut completed = Vec::new();

                    for index in indices {
                        let task = todos.mark_done(index)?.text.clone();
                        completed.push((index, task));
                    }

                    todos.save(&todo_path)?;
                    for (index, task) in completed {
                        writeln!(
                            writer,
                            "{} task {index}: {task}",
                            styled(use_color, "32", "Completed")
                        )?;
                    }
                }
                Command::Do {
                    indices,
                    branch_name,
                } => {
                    let indices = validate_task_indices(&todos, &indices)?;
                    let prompt = build_opencode_prompt(&indices);
                    let project_root = todo_path.parent().unwrap_or(cwd);

                    if let Some(branch_name) = branch_name.as_deref() {
                        let branch_name = switch_branch(project_root, branch_name)?;
                        writeln!(
                            writer,
                            "{} {}",
                            styled(use_color, "35", "Switched to branch"),
                            branch_name
                        )?;
                    }

                    run_opencode(project_root, &prompt)?;
                    for index in indices {
                        let task = todos.task(index)?.text.clone();
                        writeln!(
                            writer,
                            "{} task {index}: {task}",
                            styled(use_color, "34", "Spawned agent for")
                        )?;
                    }
                }
                Command::Uncheck(indices) => {
                    let indices = validate_task_indices(&todos, &indices)?;
                    let mut unchecked = Vec::new();

                    for index in indices {
                        let task = todos.mark_undone(index)?.text.clone();
                        unchecked.push((index, task));
                    }

                    todos.save(&todo_path)?;
                    for (index, task) in unchecked {
                        writeln!(
                            writer,
                            "{} task {index}: {task}",
                            styled(use_color, "33", "Unchecked")
                        )?;
                    }
                }
                Command::Scan => {
                    let project_root = todo_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new("."));
                    let scanned_tasks = scan_project(project_root)?;
                    let mut existing = todos
                        .tasks()
                        .iter()
                        .map(|task| task.text.clone())
                        .collect::<HashSet<_>>();
                    let mut added = 0usize;

                    for task in scanned_tasks {
                        if existing.insert(task.clone()) {
                            todos.add(task)?;
                            added += 1;
                        }
                    }

                    if added == 0 {
                        writeln!(writer, "No new TODO comments found in git-tracked files.")?;
                    } else {
                        todos.save(&todo_path)?;
                        writeln!(
                            writer,
                            "{} {added} task{} from git-tracked TODO comments.",
                            styled(use_color, "36", "Added"),
                            if added == 1 { "" } else { "s" }
                        )?;
                    }
                }
                Command::Remove(indices) => {
                    let indices = validate_task_indices(&todos, &indices)?;
                    let mut removal_order = indices.clone();
                    removal_order.sort_unstable_by(|left, right| right.cmp(left));

                    let mut removed = Vec::new();
                    for index in removal_order {
                        let task = todos.remove(index)?;
                        removed.push((index, task.text));
                    }

                    todos.save(&todo_path)?;
                    for index in indices {
                        let (_, task) = removed
                            .iter()
                            .find(|(removed_index, _)| *removed_index == index)
                            .expect("validated task should have been removed");
                        writeln!(
                            writer,
                            "{} task {index}: {task}",
                            styled(use_color, "31", "Removed")
                        )?;
                    }
                }
                Command::Next => {
                    if let Some((index, task)) = todos.next_open_task() {
                        writeln!(
                            writer,
                            "{} {index}. {}",
                            styled(use_color, "33", "Next task:"),
                            task.text
                        )?;
                    } else {
                        writeln!(
                            writer,
                            "{}",
                            styled(use_color, "32", "All tasks are complete.")
                        )?;
                    }
                }
                Command::Help | Command::Init => unreachable!("handled above"),
            }
        }
    }

    Ok(())
}

fn build_opencode_prompt(indices: &[usize]) -> String {
    let task_numbers = indices
        .iter()
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        "do all the tasks that are numbered {task_numbers} use `to` for seeing todo\n\n{OPENCODE_AGENT_PROMPT}"
    )
}

fn launch_opencode(project_root: &Path, prompt: &str) -> Result<()> {
    let status = ProcessCommand::new("opencode")
        .arg("--prompt")
        .arg(prompt)
        .current_dir(project_root)
        .status()
        .map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => {
                AppError::CommandFailed("`opencode` was not found in PATH".to_string())
            }
            _ => AppError::CommandFailed(format!("failed to launch `opencode`: {error}")),
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::CommandFailed(format!(
            "`opencode --prompt ...` exited with status {status}"
        )))
    }
}

fn switch_to_task_branch(project_root: &Path, branch_name: &str) -> Result<String> {
    if git_branch_exists(project_root, &branch_name)? {
        run_git_command(
            project_root,
            &["switch", branch_name],
            &format!("failed to switch to branch `{branch_name}`"),
        )?;
    } else {
        run_git_command(
            project_root,
            &["switch", "-c", branch_name],
            &format!("failed to create branch `{branch_name}`"),
        )?;
    }

    Ok(branch_name.to_string())
}

fn git_branch_exists(project_root: &Path, branch_name: &str) -> Result<bool> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--list")
        .arg("--format=%(refname:short)")
        .arg(branch_name)
        .output()?;

    if !output.status.success() {
        return Err(AppError::GitCommandFailed(format!(
            "failed to inspect git branches: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == branch_name))
}

fn run_git_command(project_root: &Path, args: &[&str], failure_message: &str) -> Result<()> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(project_root)
        .args(args)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(AppError::GitCommandFailed(format!(
            "{failure_message}: git exited with status {}",
            output.status
        )))
    } else {
        Err(AppError::GitCommandFailed(format!(
            "{failure_message}: {stderr}"
        )))
    }
}

fn write_task_list<W: Write>(
    writer: &mut W,
    todo_path: &Path,
    todos: &TodoList,
    query: Option<&str>,
    use_color: bool,
) -> Result<()> {
    writeln!(
        writer,
        "{}",
        styled(
            use_color,
            "1;34",
            &format!("Tasks from {}", todo_path.display())
        )
    )?;

    if todos.tasks().is_empty() {
        writeln!(writer, "No tasks yet.")?;
        return Ok(());
    }

    if let Some(query) = query {
        writeln!(writer, "{} \"{query}\"", styled(use_color, "36", "Filter:"))?;
    }

    let query = query.map(|value| value.to_lowercase());
    let mut matches = 0usize;
    let mut open = 0usize;
    let mut done = 0usize;

    for (index, task) in todos.tasks().iter().enumerate() {
        let matches_query = query
            .as_ref()
            .map(|value| task.text.to_lowercase().contains(value))
            .unwrap_or(true);

        if !matches_query {
            continue;
        }

        matches += 1;
        if task.done {
            done += 1;
        } else {
            open += 1;
        }

        writeln!(
            writer,
            "{}. {} {}",
            index + 1,
            task_marker(task.done, use_color),
            task.text
        )?;
    }

    if let Some(query) = query {
        if matches == 0 {
            writeln!(writer, "No tasks matching \"{query}\".")?;
        }
        writeln!(
            writer,
            "{} {}  {} {}  {} {}",
            styled(use_color, "36", "Matches:"),
            matches,
            styled(use_color, "33", "Open:"),
            open,
            styled(use_color, "32", "Done:"),
            done
        )?;
    } else {
        writeln!(
            writer,
            "{} {}  {} {}",
            styled(use_color, "33", "Open:"),
            open,
            styled(use_color, "32", "Done:"),
            done
        )?;
    }

    Ok(())
}

fn validate_task_indices(todos: &TodoList, indices: &[usize]) -> Result<Vec<usize>> {
    let indices = unique_indices(indices);
    for &index in &indices {
        let _ = todos.task(index)?;
    }
    Ok(indices)
}

fn unique_indices(indices: &[usize]) -> Vec<usize> {
    let mut unique = Vec::new();
    for &index in indices {
        if !unique.contains(&index) {
            unique.push(index);
        }
    }
    unique
}

fn task_marker(done: bool, use_color: bool) -> String {
    if done {
        styled(use_color, "32", "[x]")
    } else {
        styled(use_color, "33", "[ ]")
    }
}

fn styled(use_color: bool, code: &str, text: &str) -> String {
    if use_color {
        format!("\u{1b}[{code}m{text}\u{1b}[0m")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
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
            let path = std::env::temp_dir().join(format!("to-lib-{name}-{unique}"));
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
        let status = ProcessCommand::new("git")
            .args(args)
            .current_dir(path)
            .status()
            .unwrap();
        assert!(status.success(), "git command failed: {:?}", args);
    }

    #[test]
    fn help_command_writes_usage() {
        let mut output = Vec::new();
        execute(
            Command::Help,
            Path::new("."),
            &mut output,
            false,
            |_root, _prompt| Ok(()),
            |_root, branch_name| Ok(branch_name.to_string()),
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("to ls [query]"));
        assert!(rendered.contains("to init"));
        assert!(rendered.contains("to do <number> [number ...] [-b <branch-name>]"));
    }

    #[test]
    fn builds_opencode_prompt_from_indices() {
        let prompt = build_opencode_prompt(&[4, 7]);
        assert!(prompt.contains("do all the tasks that are numbered 4 7 use `to` for seeing todo"));
        assert!(prompt.contains("Inspect the codebase before making changes"));
    }

    #[test]
    fn do_command_runs_opencode_from_project_root() {
        let temp = TempDir::new("do-command");
        let project = temp.path.join("workspace");
        let nested = project.join("service").join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            project.join(".todo"),
            "[ ] implement agent runner\n[ ] update CLI parser\n",
        )
        .unwrap();

        let mut output = Vec::new();
        let mut observed_call = None;

        execute(
            Command::Do {
                indices: vec![1, 2],
                branch_name: None,
            },
            &nested,
            &mut output,
            false,
            |project_root, prompt| {
                observed_call = Some((project_root.to_path_buf(), prompt.to_string()));
                Ok(())
            },
            |_root, branch_name| Ok(branch_name.to_string()),
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Spawned agent for task 1: implement agent runner"));
        assert!(rendered.contains("Spawned agent for task 2: update CLI parser"));

        let (project_root, prompt) = observed_call.expect("expected opencode to be invoked");
        assert_eq!(project_root, project);
        assert!(prompt.contains("do all the tasks that are numbered 1 2 use `to` for seeing todo"));
    }

    #[test]
    fn list_command_filters_tasks_by_query() {
        let temp = TempDir::new("list-filter");
        fs::write(
            temp.path.join(".todo"),
            "[ ] branch work\n[x] docs cleanup\n[ ] branch follow-up\n",
        )
        .unwrap();

        let mut output = Vec::new();
        execute(
            Command::List(Some("branch".to_string())),
            &temp.path,
            &mut output,
            false,
            |_root, _prompt| Ok(()),
            |_root, branch_name| Ok(branch_name.to_string()),
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Filter: \"branch\""));
        assert!(rendered.contains("1. [ ] branch work"));
        assert!(rendered.contains("3. [ ] branch follow-up"));
        assert!(!rendered.contains("docs cleanup"));
        assert!(rendered.contains("Matches: 2  Open: 2  Done: 0"));
    }

    #[test]
    fn done_command_supports_multiple_indices() {
        let temp = TempDir::new("done-many");
        fs::write(
            temp.path.join(".todo"),
            "[ ] first\n[ ] second\n[ ] third\n",
        )
        .unwrap();

        let mut output = Vec::new();
        execute(
            Command::Done(vec![1, 3]),
            &temp.path,
            &mut output,
            false,
            |_root, _prompt| Ok(()),
            |_root, branch_name| Ok(branch_name.to_string()),
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Completed task 1: first"));
        assert!(rendered.contains("Completed task 3: third"));

        let saved = fs::read_to_string(temp.path.join(".todo")).unwrap();
        assert_eq!(saved, "[x] first\n[ ] second\n[x] third\n");
    }

    #[test]
    fn remove_command_supports_multiple_indices() {
        let temp = TempDir::new("remove-many");
        fs::write(
            temp.path.join(".todo"),
            "[ ] first\n[ ] second\n[ ] third\n",
        )
        .unwrap();

        let mut output = Vec::new();
        execute(
            Command::Remove(vec![1, 3]),
            &temp.path,
            &mut output,
            false,
            |_root, _prompt| Ok(()),
            |_root, branch_name| Ok(branch_name.to_string()),
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Removed task 1: first"));
        assert!(rendered.contains("Removed task 3: third"));

        let saved = fs::read_to_string(temp.path.join(".todo")).unwrap();
        assert_eq!(saved, "[ ] second\n");
    }

    #[test]
    fn do_command_can_switch_to_named_branch() {
        let temp = TempDir::new("do-branch");
        let project = temp.path.join("workspace");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join(".todo"), "[ ] branch task\n[ ] second task\n").unwrap();

        run_git(&project, &["init", "-b", "main"]);
        run_git(&project, &["config", "user.email", "test@example.com"]);
        run_git(&project, &["config", "user.name", "Test User"]);
        run_git(&project, &["add", ".todo"]);
        run_git(&project, &["commit", "-m", "initial"]);

        let mut output = Vec::new();
        execute(
            Command::Do {
                indices: vec![1, 2],
                branch_name: Some("feature/batch-work".to_string()),
            },
            &project,
            &mut output,
            false,
            |_root, _prompt| Ok(()),
            switch_to_task_branch,
        )
        .unwrap();

        let branch = ProcessCommand::new("git")
            .arg("-C")
            .arg(&project)
            .arg("branch")
            .arg("--show-current")
            .output()
            .unwrap();
        assert!(branch.status.success());
        assert_eq!(
            String::from_utf8_lossy(&branch.stdout).trim(),
            "feature/batch-work"
        );

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Switched to branch feature/batch-work"));
        assert!(rendered.contains("Spawned agent for task 1: branch task"));
        assert!(rendered.contains("Spawned agent for task 2: second task"));
    }
}
