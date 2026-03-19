use std::ffi::OsString;

use crate::error::{AppError, Result};

pub const HELP_TEXT: &str = "\
to - project-based TODO manager

Usage:
  to
      This is fast and no bs cli for humans and agents to store project level todos.

      Example - [ ] implement streaming responses
                [ ] add sqlite persistence
                [x] setup CLI parser

      `to` looks for a `.todo` file in the current directory and then each parent directory.

  to init
      Create a new .todo file in the current directory

  to ls
      List tasks for the current project

  to add \"task text\"
      Add a new task

  to done <number>
      Mark a task completed

  to do <number>
      Launch `opencode` for a task using the built-in agent prompt

  to uncheck <number>
      Mark a task as not completed

  to scan
      Scan git-tracked files for `TODO:` comments and add them to .todo

  to rm <number>
      Remove a task

  to next
      Show the first unfinished task
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Init,
    List,
    Add(String),
    Done(usize),
    Do(usize),
    Uncheck(usize),
    Scan,
    Remove(usize),
    Next,
}

pub fn parse_args<I>(args: I) -> Result<Command>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|_| AppError::InvalidArgs("arguments must be valid UTF-8".to_string()))
        })
        .collect::<Result<Vec<_>>>()?;

    let Some((command, rest)) = args.split_first() else {
        return Ok(Command::Help);
    };

    match command.as_str() {
        "help" | "-h" | "--help" => Ok(Command::Help),
        "init" => expect_no_extra_args(rest, Command::Init),
        "ls" => expect_no_extra_args(rest, Command::List),
        "next" => expect_no_extra_args(rest, Command::Next),
        "scan" => expect_no_extra_args(rest, Command::Scan),
        "add" => {
            if rest.is_empty() {
                return Err(AppError::InvalidArgs(
                    "missing task text: use `to add \"task text\"`".to_string(),
                ));
            }

            let text = rest.join(" ");
            if text.trim().is_empty() {
                return Err(AppError::EmptyTask);
            }

            Ok(Command::Add(text))
        }
        "done" => parse_index_command(rest, "done", Command::Done),
        "do" => parse_index_command(rest, "do", Command::Do),
        "uncheck" => parse_index_command(rest, "uncheck", Command::Uncheck),
        "rm" => parse_index_command(rest, "rm", Command::Remove),
        other => Err(AppError::InvalidArgs(format!(
            "unknown command `{other}`: run `to` for usage"
        ))),
    }
}

fn expect_no_extra_args(rest: &[String], command: Command) -> Result<Command> {
    if rest.is_empty() {
        Ok(command)
    } else {
        Err(AppError::InvalidArgs("too many arguments".to_string()))
    }
}

fn parse_index_command(
    rest: &[String],
    name: &str,
    constructor: fn(usize) -> Command,
) -> Result<Command> {
    if rest.len() != 1 {
        return Err(AppError::InvalidArgs(format!(
            "usage: `to {name} <number>`"
        )));
    }

    let index = rest[0].parse::<usize>().map_err(|_| {
        AppError::InvalidArgs(format!(
            "task number must be a positive integer for `{name}`"
        ))
    })?;

    Ok(constructor(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_help_by_default() {
        assert_eq!(parse_args(Vec::<OsString>::new()).unwrap(), Command::Help);
    }

    #[test]
    fn parses_add_command() {
        assert_eq!(
            parse_args(args(&["add", "write", "tests"])).unwrap(),
            Command::Add("write tests".to_string())
        );
    }

    #[test]
    fn parses_uncheck_command() {
        assert_eq!(
            parse_args(args(&["uncheck", "2"])).unwrap(),
            Command::Uncheck(2)
        );
    }

    #[test]
    fn parses_do_command() {
        assert_eq!(parse_args(args(&["do", "3"])).unwrap(), Command::Do(3));
    }

    #[test]
    fn parses_scan_command() {
        assert_eq!(parse_args(args(&["scan"])).unwrap(), Command::Scan);
    }

    #[test]
    fn rejects_unknown_commands() {
        let error = parse_args(args(&["wat"])).unwrap_err();
        assert!(error.to_string().contains("unknown command"));
    }
}
