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

  to ls [query]
      List tasks for the current project, optionally filtered by a search query

  to add \"task text\"
      Add a new task

  to done <number> [number ...]
      Mark one or more tasks completed

  to do [-b] <number>
      Launch `opencode` for a task, optionally switching to branch `task-<number>` first

  to uncheck <number> [number ...]
      Mark one or more tasks as not completed

  to scan
      Scan git-tracked files for `TODO:` comments and add them to .todo

  to rm <number> [number ...]
      Remove one or more tasks

  to next
      Show the first unfinished task
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Init,
    List(Option<String>),
    Add(String),
    Done(Vec<usize>),
    Do { index: usize, create_branch: bool },
    Uncheck(Vec<usize>),
    Scan,
    Remove(Vec<usize>),
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
        "ls" => parse_list_command(rest),
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
        "done" => parse_indices_command(rest, "done", Command::Done),
        "do" => parse_do_command(rest),
        "uncheck" => parse_indices_command(rest, "uncheck", Command::Uncheck),
        "rm" => parse_indices_command(rest, "rm", Command::Remove),
        other => Err(AppError::InvalidArgs(format!(
            "unknown command `{other}`: run `to` for usage"
        ))),
    }
}

fn parse_list_command(rest: &[String]) -> Result<Command> {
    let query = rest.join(" ");
    let query = query.trim();

    if query.is_empty() {
        Ok(Command::List(None))
    } else {
        Ok(Command::List(Some(query.to_string())))
    }
}

fn expect_no_extra_args(rest: &[String], command: Command) -> Result<Command> {
    if rest.is_empty() {
        Ok(command)
    } else {
        Err(AppError::InvalidArgs("too many arguments".to_string()))
    }
}

fn parse_indices_command(
    rest: &[String],
    name: &str,
    constructor: fn(Vec<usize>) -> Command,
) -> Result<Command> {
    if rest.is_empty() {
        return Err(AppError::InvalidArgs(format!(
            "usage: `to {name} <number> [number ...]`"
        )));
    }

    let indices = rest
        .iter()
        .map(|value| {
            value.parse::<usize>().map_err(|_| {
                AppError::InvalidArgs(format!(
                    "task number must be a positive integer for `{name}`"
                ))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(constructor(indices))
}

fn parse_do_command(rest: &[String]) -> Result<Command> {
    if rest.is_empty() {
        return Err(do_usage_error());
    }

    let mut create_branch = false;
    let mut index = None;

    for value in rest {
        match value.as_str() {
            "-b" | "--branch" => {
                if create_branch {
                    return Err(do_usage_error());
                }
                create_branch = true;
            }
            _ => {
                if index.is_some() {
                    return Err(do_usage_error());
                }

                index = Some(value.parse::<usize>().map_err(|_| {
                    AppError::InvalidArgs(
                        "task number must be a positive integer for `do`".to_string(),
                    )
                })?);
            }
        }
    }

    let index = index.ok_or_else(do_usage_error)?;
    Ok(Command::Do {
        index,
        create_branch,
    })
}

fn do_usage_error() -> AppError {
    AppError::InvalidArgs("usage: `to do [-b] <number>`".to_string())
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
            Command::Uncheck(vec![2])
        );
    }

    #[test]
    fn parses_do_command() {
        assert_eq!(
            parse_args(args(&["do", "3"])).unwrap(),
            Command::Do {
                index: 3,
                create_branch: false
            }
        );
    }

    #[test]
    fn parses_do_command_with_branch_flag() {
        assert_eq!(
            parse_args(args(&["do", "-b", "3"])).unwrap(),
            Command::Do {
                index: 3,
                create_branch: true
            }
        );
        assert_eq!(
            parse_args(args(&["do", "3", "-b"])).unwrap(),
            Command::Do {
                index: 3,
                create_branch: true
            }
        );
    }

    #[test]
    fn parses_list_query() {
        assert_eq!(
            parse_args(args(&["ls", "branch"])).unwrap(),
            Command::List(Some("branch".to_string()))
        );
    }

    #[test]
    fn parses_done_command_with_multiple_indices() {
        assert_eq!(
            parse_args(args(&["done", "1", "3"])).unwrap(),
            Command::Done(vec![1, 3])
        );
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
