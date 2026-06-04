use std::ffi::OsString;

use crate::error::{AppError, Result};
use crate::todo::{clean_label, Priority};

pub const HELP_TEXT: &str = "\
to - project TODOs

Usage:
  to [ls] [query] [--priority <high|medium|low>] [--label <label>]
      List tasks for the current project. Filters can be combined.

  to init
      Create a new .todo file in the current directory

  to add \"task text\" [--parent <number>] [--priority <high|medium|low>] [--label <label>]
      Add a new task or subtask

  to edit
      Open the project .todo in your editor

  to mv <from> <to>
      Move a task (and its subtasks) to a new position

  to done <number> [number ...]
      Mark one or more tasks completed

  to uncheck <number> [number ...]
      Mark one or more tasks as not completed

  to rm <number> [number ...]
      Remove one or more tasks

  to prune
      Remove completed tasks that have no open subtasks

  to do <number> [number ...] [-b <branch-name> | --create-branch]
      Launch `opencode` for one or more tasks; optionally switch/create a branch

  to next
      Show the first unfinished task

  to tree <number>
      Show a task and its subtasks

  to scan
      Scan git-tracked files for `TODO:` comments and add them to .todo

Notes:
  `to` looks for a `.todo` file in the current directory and then each parent directory.

Examples:
  to
  to add \"implement streaming responses\"
  to add \"write unit tests\" --parent 1
  to edit
  to mv 5 2
  to ls sqlite --priority high --label backend
  to done 1 2
  to do 1 --create-branch
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Init,
    List(ListOptions),
    Add {
        text: String,
        parent: Option<usize>,
        priority: Option<Priority>,
        labels: Vec<String>,
    },
    Edit,
    Move { from: usize, to: usize },
    Done(Vec<usize>),
    Do {
        indices: Vec<usize>,
        branch_name: Option<String>,
        create_branch: bool,
    },
    Uncheck(Vec<usize>),
    Scan,
    Remove(Vec<usize>),
    Prune,
    Next,
    Tree(usize),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ListOptions {
    pub query: Option<String>,
    pub priority: Option<Priority>,
    pub labels: Vec<String>,
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
        return Ok(Command::List(ListOptions::default()));
    };

    match command.as_str() {
        "help" | "-h" | "--help" => Ok(Command::Help),
        "init" => expect_no_extra_args(rest, Command::Init),
        "ls" => parse_list_command(rest),
        "next" => expect_no_extra_args(rest, Command::Next),
        "scan" => expect_no_extra_args(rest, Command::Scan),
        "edit" => expect_no_extra_args(rest, Command::Edit),
        "add" => parse_add_command(rest),
        "done" => parse_indices_command(rest, "done", Command::Done),
        "do" => parse_do_command(rest),
        "uncheck" => parse_indices_command(rest, "uncheck", Command::Uncheck),
        "rm" => parse_indices_command(rest, "rm", Command::Remove),
        "mv" => parse_move_command(rest),
        "prune" => expect_no_extra_args(rest, Command::Prune),
        "tree" => parse_tree_command(rest),
        other => Err(AppError::InvalidArgs(format!(
            "unknown command `{other}`: run `to --help` for usage"
        ))),
    }
}

fn parse_list_command(rest: &[String]) -> Result<Command> {
    let mut options = ListOptions::default();
    let mut expect_priority = false;
    let mut expect_label = false;
    let mut query_parts = Vec::new();

    for value in rest {
        if expect_priority {
            options.priority = Some(Priority::parse(value)?);
            expect_priority = false;
            continue;
        }

        if expect_label {
            append_labels(&mut options.labels, value)?;
            expect_label = false;
            continue;
        }

        match value.as_str() {
            "--priority" | "-P" => {
                if options.priority.is_some() {
                    return Err(list_usage_error());
                }
                expect_priority = true;
            }
            "--label" | "-l" => {
                expect_label = true;
            }
            _ => query_parts.push(value.clone()),
        }
    }

    if expect_priority {
        return Err(AppError::InvalidArgs(
            "missing priority: use `--priority <high|medium|low>`".to_string(),
        ));
    }

    if expect_label {
        return Err(AppError::InvalidArgs(
            "missing label: use `--label <label>`".to_string(),
        ));
    }

    let query = query_parts.join(" ");
    let query = query.trim();
    if query.is_empty() {
        options.query = None;
    } else {
        options.query = Some(query.to_string());
    }

    Ok(Command::List(options))
}

fn parse_add_command(rest: &[String]) -> Result<Command> {
    if rest.is_empty() {
        return Err(AppError::InvalidArgs(
            "missing task text: use `to add \"task text\"`".to_string(),
        ));
    }

    let mut parent = None;
    let mut priority = None;
    let mut labels = Vec::new();
    let mut expect_parent = false;
    let mut expect_priority = false;
    let mut expect_label = false;
    let mut text_parts = Vec::new();

    for value in rest {
        if expect_parent {
            parent = Some(value.parse::<usize>().map_err(|_| {
                AppError::InvalidArgs("parent task number must be a positive integer".to_string())
            })?);
            expect_parent = false;
            continue;
        }

        if expect_priority {
            priority = Some(Priority::parse(value)?);
            expect_priority = false;
            continue;
        }

        if expect_label {
            append_labels(&mut labels, value)?;
            expect_label = false;
            continue;
        }

        if matches!(value.as_str(), "--parent" | "-p") {
            if parent.is_some() {
                return Err(add_usage_error());
            }
            expect_parent = true;
            continue;
        }

        if matches!(value.as_str(), "--priority" | "-P") {
            if priority.is_some() {
                return Err(add_usage_error());
            }
            expect_priority = true;
            continue;
        }

        if matches!(value.as_str(), "--label" | "-l") {
            expect_label = true;
            continue;
        }

        text_parts.push(value.clone());
    }

    if expect_parent {
        return Err(AppError::InvalidArgs(
            "missing parent task number: use `--parent <number>`".to_string(),
        ));
    }
    if expect_priority {
        return Err(AppError::InvalidArgs(
            "missing priority: use `--priority <high|medium|low>`".to_string(),
        ));
    }
    if expect_label {
        return Err(AppError::InvalidArgs(
            "missing label: use `--label <label>`".to_string(),
        ));
    }

    let text = text_parts.join(" ");
    if text.trim().is_empty() {
        return Err(AppError::EmptyTask);
    }

    Ok(Command::Add {
        text,
        parent,
        priority,
        labels,
    })
}

fn add_usage_error() -> AppError {
    AppError::InvalidArgs(
        "usage: `to add \"task text\" [--parent <number>] [--priority <high|medium|low>] [--label <label>]`"
            .to_string(),
    )
}

fn list_usage_error() -> AppError {
    AppError::InvalidArgs(
        "usage: `to ls [query] [--priority <high|medium|low>] [--label <label>]`".to_string(),
    )
}

fn append_labels(labels: &mut Vec<String>, value: &str) -> Result<()> {
    for label in value.split(',') {
        let label = clean_label(label)?;
        if !labels.contains(&label) {
            labels.push(label);
        }
    }

    Ok(())
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

fn parse_tree_command(rest: &[String]) -> Result<Command> {
    let [index] = rest else {
        return Err(AppError::InvalidArgs(
            "usage: `to tree <number>`".to_string(),
        ));
    };

    Ok(Command::Tree(index.parse::<usize>().map_err(|_| {
        AppError::InvalidArgs("task number must be a positive integer for `tree`".to_string())
    })?))
}

fn parse_move_command(rest: &[String]) -> Result<Command> {
    let [from, to] = rest else {
        return Err(AppError::InvalidArgs(
            "usage: `to mv <from> <to>`".to_string(),
        ));
    };

    let from = from.parse::<usize>().map_err(|_| {
        AppError::InvalidArgs("task number must be a positive integer for `mv`".to_string())
    })?;
    let to = to.parse::<usize>().map_err(|_| {
        AppError::InvalidArgs("task number must be a positive integer for `mv`".to_string())
    })?;

    Ok(Command::Move { from, to })
}

fn parse_do_command(rest: &[String]) -> Result<Command> {
    if rest.is_empty() {
        return Err(do_usage_error());
    }

    let mut branch_name = None;
    let mut create_branch = false;
    let mut expect_branch_name = false;
    let mut indices = Vec::new();

    for value in rest {
        if expect_branch_name {
            if value.trim().is_empty() {
                return Err(AppError::InvalidArgs(
                    "branch name for `to do -b` cannot be empty".to_string(),
                ));
            }
            branch_name = Some(value.clone());
            expect_branch_name = false;
            continue;
        }

        if matches!(value.as_str(), "-b" | "--branch") {
            if branch_name.is_some() || create_branch {
                return Err(do_usage_error());
            }
            expect_branch_name = true;
            continue;
        }

        if value == "--create-branch" {
            if branch_name.is_some() || create_branch {
                return Err(do_usage_error());
            }
            create_branch = true;
            continue;
        }

        indices.push(value.parse::<usize>().map_err(|_| {
            AppError::InvalidArgs("task number must be a positive integer for `do`".to_string())
        })?);
    }

    if indices.is_empty() {
        return Err(do_usage_error());
    }

    if expect_branch_name {
        return Err(AppError::InvalidArgs(
            "missing branch name for `to do -b`: use `-b <branch-name>`".to_string(),
        ));
    }

    Ok(Command::Do {
        indices,
        branch_name,
        create_branch,
    })
}

fn do_usage_error() -> AppError {
    AppError::InvalidArgs(
        "usage: `to do <number> [number ...] [-b <branch-name>|--create-branch]`".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_list_by_default() {
        assert_eq!(
            parse_args(Vec::<OsString>::new()).unwrap(),
            Command::List(ListOptions::default())
        );
    }

    #[test]
    fn parses_add_command() {
        assert_eq!(
            parse_args(args(&["add", "write", "tests"])).unwrap(),
            Command::Add {
                text: "write tests".to_string(),
                parent: None,
                priority: None,
                labels: Vec::new()
            }
        );
    }

    #[test]
    fn parses_add_subtask_command() {
        assert_eq!(
            parse_args(args(&["add", "write", "tests", "--parent", "2"])).unwrap(),
            Command::Add {
                text: "write tests".to_string(),
                parent: Some(2),
                priority: None,
                labels: Vec::new()
            }
        );
    }

    #[test]
    fn parses_add_metadata_command() {
        assert_eq!(
            parse_args(args(&[
                "add",
                "fix",
                "api",
                "--priority",
                "high",
                "--label",
                "backend",
                "--label",
                "api"
            ]))
            .unwrap(),
            Command::Add {
                text: "fix api".to_string(),
                parent: None,
                priority: Some(Priority::High),
                labels: vec!["backend".to_string(), "api".to_string()]
            }
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
                indices: vec![3],
                branch_name: None,
                create_branch: false
            }
        );
    }

    #[test]
    fn parses_do_command_with_branch_flag() {
        assert_eq!(
            parse_args(args(&["do", "-b", "feature/work", "3"])).unwrap(),
            Command::Do {
                indices: vec![3],
                branch_name: Some("feature/work".to_string()),
                create_branch: false
            }
        );
        assert_eq!(
            parse_args(args(&["do", "3", "-b", "feature/work"])).unwrap(),
            Command::Do {
                indices: vec![3],
                branch_name: Some("feature/work".to_string()),
                create_branch: false
            }
        );
    }

    #[test]
    fn parses_do_command_with_multiple_indices() {
        assert_eq!(
            parse_args(args(&["do", "3", "5"])).unwrap(),
            Command::Do {
                indices: vec![3, 5],
                branch_name: None,
                create_branch: false
            }
        );
    }

    #[test]
    fn parses_do_command_with_multiple_indices_and_branch_name() {
        assert_eq!(
            parse_args(args(&["do", "3", "5", "-b", "batch-work"])).unwrap(),
            Command::Do {
                indices: vec![3, 5],
                branch_name: Some("batch-work".to_string()),
                create_branch: false
            }
        );
    }

    #[test]
    fn parses_do_command_with_auto_branch() {
        assert_eq!(
            parse_args(args(&["do", "3", "--create-branch"])).unwrap(),
            Command::Do {
                indices: vec![3],
                branch_name: None,
                create_branch: true
            }
        );
    }

    #[test]
    fn parses_move_command() {
        assert_eq!(
            parse_args(args(&["mv", "4", "2"])).unwrap(),
            Command::Move { from: 4, to: 2 }
        );
    }

    #[test]
    fn parses_edit_command() {
        assert_eq!(parse_args(args(&["edit"])).unwrap(), Command::Edit);
    }

    #[test]
    fn parses_prune_command() {
        assert_eq!(parse_args(args(&["prune"])).unwrap(), Command::Prune);
    }

    #[test]
    fn rejects_do_command_without_branch_name() {
        let error = parse_args(args(&["do", "3", "-b"])).unwrap_err();
        assert_eq!(
            error.to_string(),
            "missing branch name for `to do -b`: use `-b <branch-name>`"
        );
    }

    #[test]
    fn parses_list_query() {
        assert_eq!(
            parse_args(args(&["ls", "branch"])).unwrap(),
            Command::List(ListOptions {
                query: Some("branch".to_string()),
                priority: None,
                labels: Vec::new()
            })
        );
    }

    #[test]
    fn parses_list_metadata_filters() {
        assert_eq!(
            parse_args(args(&["ls", "--priority", "low", "--label", "frontend,ui"])).unwrap(),
            Command::List(ListOptions {
                query: None,
                priority: Some(Priority::Low),
                labels: vec!["frontend".to_string(), "ui".to_string()]
            })
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
    fn parses_tree_command() {
        assert_eq!(parse_args(args(&["tree", "2"])).unwrap(), Command::Tree(2));
    }

    #[test]
    fn rejects_unknown_commands() {
        let error = parse_args(args(&["wat"])).unwrap_err();
        assert!(error.to_string().contains("unknown command"));
    }
}
