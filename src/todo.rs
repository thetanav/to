use std::fs;
use std::path::Path;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub done: bool,
    pub text: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TodoList {
    tasks: Vec<Task>,
}

impl TodoList {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Self::parse(&contents)
    }

    pub fn parse(contents: &str) -> Result<Self> {
        let mut tasks = Vec::new();

        for (line_number, line) in contents.lines().enumerate() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                continue;
            }

            let (done, text) = if let Some(text) = trimmed.strip_prefix("[ ] ") {
                (false, text)
            } else if let Some(text) = trimmed.strip_prefix("[x] ") {
                (true, text)
            } else if let Some(text) = trimmed.strip_prefix("[X] ") {
                (true, text)
            } else {
                return Err(AppError::MalformedTodoLine {
                    line: line_number + 1,
                    content: trimmed.to_string(),
                });
            };

            let text = text.trim();
            if text.is_empty() {
                return Err(AppError::MalformedTodoLine {
                    line: line_number + 1,
                    content: trimmed.to_string(),
                });
            }

            tasks.push(Task {
                done,
                text: text.to_string(),
            });
        }

        Ok(Self { tasks })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut contents = String::new();
        for task in &self.tasks {
            let marker = if task.done { "[x]" } else { "[ ]" };
            contents.push_str(marker);
            contents.push(' ');
            contents.push_str(&task.text);
            contents.push('\n');
        }

        let temp_path = path.with_file_name(".todo.tmp");
        fs::write(&temp_path, &contents)?;

        if let Err(_) = fs::rename(&temp_path, path) {
            fs::write(path, contents)?;
            let _ = fs::remove_file(temp_path);
        }

        Ok(())
    }

    pub fn add(&mut self, text: String) -> Result<usize> {
        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(AppError::EmptyTask);
        }

        self.tasks.push(Task { done: false, text });
        Ok(self.tasks.len())
    }

    pub fn mark_done(&mut self, index: usize) -> Result<&Task> {
        let task = self.task_mut(index)?;
        task.done = true;
        Ok(task)
    }

    pub fn mark_undone(&mut self, index: usize) -> Result<&Task> {
        let task = self.task_mut(index)?;
        task.done = false;
        Ok(task)
    }

    pub fn remove(&mut self, index: usize) -> Result<Task> {
        let index = self.checked_index(index)?;
        Ok(self.tasks.remove(index))
    }

    pub fn next_open_task(&self) -> Option<(usize, &Task)> {
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, task)| !task.done)
            .map(|(index, task)| (index + 1, task))
    }

    pub fn task(&self, index: usize) -> Result<&Task> {
        let index = self.checked_index(index)?;
        Ok(&self.tasks[index])
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn open_count(&self) -> usize {
        self.tasks.iter().filter(|task| !task.done).count()
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|task| task.done).count()
    }

    fn task_mut(&mut self, index: usize) -> Result<&mut Task> {
        let index = self.checked_index(index)?;
        Ok(&mut self.tasks[index])
    }

    fn checked_index(&self, index: usize) -> Result<usize> {
        if index == 0 || index > self.tasks.len() {
            return Err(AppError::InvalidTaskIndex {
                index,
                len: self.tasks.len(),
            });
        }

        Ok(index - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_todo_contents() {
        let list = TodoList::parse("[ ] ship feature\n[x] write docs\n").unwrap();
        assert_eq!(list.tasks.len(), 2);
        assert!(!list.tasks[0].done);
        assert!(list.tasks[1].done);
    }

    #[test]
    fn finds_next_open_task() {
        let list = TodoList::parse("[x] ship feature\n[ ] write docs\n").unwrap();
        let next = list.next_open_task().unwrap();
        assert_eq!(next.0, 2);
        assert_eq!(next.1.text, "write docs");
    }

    #[test]
    fn can_uncheck_a_completed_task() {
        let mut list = TodoList::parse("[x] ship feature\n").unwrap();
        list.mark_undone(1).unwrap();
        assert!(!list.tasks[0].done);
    }

    #[test]
    fn rejects_malformed_lines() {
        let error = TodoList::parse("- invalid").unwrap_err();
        assert!(matches!(error, AppError::MalformedTodoLine { .. }));
    }
}
