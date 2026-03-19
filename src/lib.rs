mod cli;
mod error;
mod project;
mod scan;
mod todo;

use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
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
    execute(command, &cwd, &mut io::stdout(), launch_opencode)
}

fn execute<W, F>(command: Command, cwd: &Path, writer: &mut W, mut run_opencode: F) -> Result<()>
where
    W: Write,
    F: FnMut(&Path, &str) -> Result<()>,
{
    match command {
        Command::Help => {
            writer.write_all(cli::HELP_TEXT.as_bytes())?;
        }
        Command::Init => {
            let path = init_todo_file(cwd)?;
            writeln!(writer, "Initialized {}", path.display())?;
        }
        other => {
            let todo_path = find_todo_file(cwd)?;
            let mut todos = TodoList::load(&todo_path)?;

            match other {
                Command::List => write_task_list(writer, &todo_path, &todos)?,
                Command::Add(text) => {
                    let index = todos.add(text)?;
                    let task = &todos.tasks()[index - 1];
                    todos.save(&todo_path)?;
                    writeln!(writer, "Added task {index}: {}", task.text)?;
                }
                Command::Done(index) => {
                    let task = todos.mark_done(index)?.text.clone();
                    todos.save(&todo_path)?;
                    writeln!(writer, "Completed task {index}: {task}")?;
                }
                Command::Do(index) => {
                    let task = todos.task(index)?;
                    let prompt = build_opencode_prompt(index, &task.text);
                    let project_root = todo_path.parent().unwrap_or(cwd);
                    run_opencode(project_root, &prompt)?;
                    writeln!(writer, "Spawned agent for task {index}: {}", task.text)?;
                }
                Command::Uncheck(index) => {
                    let task = todos.mark_undone(index)?.text.clone();
                    todos.save(&todo_path)?;
                    writeln!(writer, "Unchecked task {index}: {task}")?;
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
                            "Added {added} task{} from git-tracked TODO comments.",
                            if added == 1 { "" } else { "s" }
                        )?;
                    }
                }
                Command::Remove(index) => {
                    let task = todos.remove(index)?;
                    todos.save(&todo_path)?;
                    writeln!(writer, "Removed task {index}: {}", task.text)?;
                }
                Command::Next => {
                    if let Some((index, task)) = todos.next_open_task() {
                        writeln!(writer, "Next task: {index}. {}", task.text)?;
                    } else {
                        writeln!(writer, "All tasks are complete.")?;
                    }
                }
                Command::Help | Command::Init => unreachable!("handled above"),
            }
        }
    }

    Ok(())
}

fn build_opencode_prompt(index: usize, task: &str) -> String {
    format!("Task #{index}: {task}\n\n{OPENCODE_AGENT_PROMPT}")
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

fn write_task_list<W: Write>(
    writer: &mut W,
    todo_path: &std::path::Path,
    todos: &TodoList,
) -> Result<()> {
    writeln!(writer, "Tasks from {}", todo_path.display())?;

    if todos.tasks().is_empty() {
        writeln!(writer, "No tasks yet.")?;
        return Ok(());
    }

    for (index, task) in todos.tasks().iter().enumerate() {
        let marker = if task.done { "[x]" } else { "[ ]" };
        writeln!(writer, "{}. {} {}", index + 1, marker, task.text)?;
    }

    writeln!(
        writer,
        "Open: {}  Done: {}",
        todos.open_count(),
        todos.done_count()
    )?;
    Ok(())
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

    #[test]
    fn help_command_writes_usage() {
        let mut output = Vec::new();
        execute(
            Command::Help,
            Path::new("."),
            &mut output,
            |_root, _prompt| Ok(()),
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("to ls"));
        assert!(rendered.contains("to init"));
    }

    #[test]
    fn builds_opencode_prompt_from_task() {
        let prompt = build_opencode_prompt(4, "implement agent runner");
        assert!(prompt.contains("Task #4: implement agent runner"));
        assert!(prompt.contains("Inspect the codebase before making changes"));
    }

    #[test]
    fn do_command_runs_opencode_from_project_root() {
        let temp = TempDir::new("do-command");
        let project = temp.path.join("workspace");
        let nested = project.join("service").join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(project.join(".todo"), "[ ] implement agent runner\n").unwrap();

        let mut output = Vec::new();
        let mut observed_call = None;

        execute(
            Command::Do(1),
            &nested,
            &mut output,
            |project_root, prompt| {
                observed_call = Some((project_root.to_path_buf(), prompt.to_string()));
                Ok(())
            },
        )
        .unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("Spawned agent for task 1: implement agent runner"));

        let (project_root, prompt) = observed_call.expect("expected opencode to be invoked");
        assert_eq!(project_root, project);
        assert!(prompt.contains("Task #1: implement agent runner"));
    }
}
