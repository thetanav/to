mod cli;
mod error;
mod project;
mod todo;

use std::env;
use std::io::{self, Write};

use cli::Command;
pub use error::{AppError, Result};
use project::{find_todo_file, init_todo_file};
use todo::TodoList;

pub fn run() -> Result<()> {
    let command = cli::parse_args(env::args_os().skip(1))?;
    execute(command, &mut io::stdout())
}

fn execute<W: Write>(command: Command, writer: &mut W) -> Result<()> {
    match command {
        Command::Help => {
            writer.write_all(cli::HELP_TEXT.as_bytes())?;
        }
        Command::Init => {
            let cwd = env::current_dir()?;
            let path = init_todo_file(&cwd)?;
            writeln!(writer, "Initialized {}", path.display())?;
        }
        other => {
            let cwd = env::current_dir()?;
            let todo_path = find_todo_file(&cwd)?;
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
                Command::Uncheck(index) => {
                    let task = todos.mark_undone(index)?.text.clone();
                    todos.save(&todo_path)?;
                    writeln!(writer, "Unchecked task {index}: {task}")?;
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

    #[test]
    fn help_command_writes_usage() {
        let mut output = Vec::new();
        execute(Command::Help, &mut output).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("to ls"));
        assert!(rendered.contains("to init"));
    }
}
