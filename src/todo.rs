use std::fs;
use std::path::Path;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "high" | "h" => Ok(Self::High),
            "medium" | "med" | "m" => Ok(Self::Medium),
            "low" | "l" => Ok(Self::Low),
            _ => Err(AppError::InvalidArgs(format!(
                "priority must be high, medium, or low; got `{value}`"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::High => "@high",
            Self::Medium => "@medium",
            Self::Low => "@low",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub done: bool,
    pub text: String,
    pub indent: usize,
    pub priority: Option<Priority>,
    pub labels: Vec<String>,
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
            let trimmed_end = line.trim_end();
            if trimmed_end.is_empty() {
                continue;
            }

            let leading_spaces = trimmed_end.chars().take_while(|ch| *ch == ' ').count();
            if leading_spaces % 2 != 0 {
                return Err(AppError::MalformedTodoLine {
                    line: line_number + 1,
                    content: trimmed_end.to_string(),
                });
            }

            let indent = leading_spaces / 2;
            let trimmed = &trimmed_end[leading_spaces..];
            let (done, text) = if let Some(text) = trimmed.strip_prefix("[ ] ") {
                (false, text)
            } else if let Some(text) = trimmed.strip_prefix("[x] ") {
                (true, text)
            } else if let Some(text) = trimmed.strip_prefix("[X] ") {
                (true, text)
            } else {
                return Err(AppError::MalformedTodoLine {
                    line: line_number + 1,
                    content: trimmed_end.to_string(),
                });
            };

            let (text, priority, labels) = parse_task_metadata(text.trim())?;
            if text.is_empty() {
                return Err(AppError::MalformedTodoLine {
                    line: line_number + 1,
                    content: trimmed_end.to_string(),
                });
            }

            tasks.push(Task {
                done,
                text,
                indent,
                priority,
                labels,
            });
        }

        Ok(Self { tasks })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut contents = String::new();
        for task in &self.tasks {
            let marker = if task.done { "[x]" } else { "[ ]" };
            for _ in 0..task.indent {
                contents.push_str("  ");
            }
            contents.push_str(marker);
            contents.push(' ');
            contents.push_str(&task.render_text());
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
        self.add_with_metadata(text, None, Vec::new())
    }

    pub fn add_with_metadata(
        &mut self,
        text: String,
        priority: Option<Priority>,
        labels: Vec<String>,
    ) -> Result<usize> {
        self.add_with_metadata_at_indent(text, 0, priority, labels)
    }

    pub fn add_child_with_metadata(
        &mut self,
        parent_index: usize,
        text: String,
        priority: Option<Priority>,
        labels: Vec<String>,
    ) -> Result<usize> {
        let parent = self.checked_index(parent_index)?;
        let indent = self.tasks[parent].indent + 1;
        let insert_at = self.subtree_end(parent);
        let text = clean_task_text(text)?;
        let labels = clean_labels(labels)?;

        self.tasks.insert(
            insert_at,
            Task {
                done: false,
                text,
                indent,
                priority,
                labels,
            },
        );
        Ok(insert_at + 1)
    }

    fn add_with_metadata_at_indent(
        &mut self,
        text: String,
        indent: usize,
        priority: Option<Priority>,
        labels: Vec<String>,
    ) -> Result<usize> {
        let text = clean_task_text(text)?;
        let labels = clean_labels(labels)?;
        self.tasks.push(Task {
            done: false,
            text,
            indent,
            priority,
            labels,
        });
        Ok(self.tasks.len())
    }

    pub fn subtree(&self, index: usize) -> Result<&[Task]> {
        let index = self.checked_index(index)?;
        Ok(&self.tasks[index..self.subtree_end(index)])
    }

    fn subtree_end(&self, index: usize) -> usize {
        let indent = self.tasks[index].indent;
        self.tasks
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, task)| task.indent <= indent)
            .map(|(index, _)| index)
            .unwrap_or(self.tasks.len())
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

    pub fn move_task(&mut self, from: usize, to: usize) -> Result<()> {
        let from_index = self.checked_index(from)?;
        let list_len = self.tasks.len();

        if to == from {
            return Ok(());
        }

        let to_index = if to > list_len + 1 {
            return Err(AppError::InvalidTaskIndex {
                index: to,
                len: list_len,
            });
        } else if to == list_len + 1 {
            list_len
        } else {
            self.checked_index(to)?
        };

        let moved_indent = self.tasks[from_index].indent;
        let subtree_end = self.subtree_end(from_index);
        if to_index >= from_index && to_index < subtree_end {
            return Err(AppError::InvalidArgs(
                "cannot move a task into its own subtree".to_string(),
            ));
        }

        let mut segment: Vec<Task> = self.tasks.drain(from_index..subtree_end).collect();

        let mut insert_at = to_index;
        if from_index < to_index {
            insert_at = insert_at.saturating_sub(segment.len());
        }

        if insert_at > self.tasks.len() {
            insert_at = self.tasks.len();
        }

        let target_indent = self.tasks.get(insert_at).map(|task| task.indent);
        if let Some(target_indent) = target_indent {
            let indent_delta = target_indent as isize - moved_indent as isize;
            for task in &mut segment {
                if indent_delta > 0 {
                    task.indent = task.indent.saturating_add(indent_delta as usize);
                } else if indent_delta < 0 {
                    task.indent = task.indent.saturating_sub((-indent_delta) as usize);
                }
            }
        }

        self.tasks.splice(insert_at..insert_at, segment);
        Ok(())
    }

    pub fn prune_completed(&mut self) -> usize {
        if self.tasks.is_empty() {
            return 0;
        }

        let mut pruned = 0usize;
        let mut index = 0usize;
        while index < self.tasks.len() {
            if !self.tasks[index].done {
                index += 1;
                continue;
            }

            let subtree_end = self.subtree_end(index);
            let all_done = self.tasks[index..subtree_end].iter().all(|task| task.done);
            if !all_done {
                index += 1;
                continue;
            }

            pruned += subtree_end - index;
            self.tasks.drain(index..subtree_end);
        }

        pruned
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
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

impl Task {
    pub fn render_text(&self) -> String {
        let mut text = self.text.clone();

        if let Some(priority) = self.priority {
            text.push(' ');
            text.push_str(priority.tag());
        }

        for label in &self.labels {
            text.push(' ');
            text.push('#');
            text.push_str(label);
        }

        text
    }
}

fn clean_task_text(text: String) -> Result<String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(AppError::EmptyTask);
    }

    Ok(text)
}

fn clean_labels(labels: Vec<String>) -> Result<Vec<String>> {
    let mut cleaned = Vec::new();
    for label in labels {
        let label = clean_label(&label)?;
        if !cleaned.contains(&label) {
            cleaned.push(label);
        }
    }

    Ok(cleaned)
}

pub fn clean_label(label: &str) -> Result<String> {
    let label = label.trim().trim_start_matches('#');
    if label.is_empty() {
        return Err(AppError::InvalidArgs("label cannot be empty".to_string()));
    }

    if !label
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(AppError::InvalidArgs(format!(
            "label `{label}` can only contain letters, numbers, hyphen, or underscore"
        )));
    }

    Ok(label.to_ascii_lowercase())
}

fn parse_task_metadata(text: &str) -> Result<(String, Option<Priority>, Vec<String>)> {
    let mut words = text.split_whitespace().collect::<Vec<_>>();
    let mut priority = None;
    let mut labels = Vec::new();

    loop {
        let Some(last) = words.last().copied() else {
            break;
        };

        if let Some(label) = last.strip_prefix('#') {
            labels.push(clean_label(label)?);
            words.pop();
            continue;
        }

        if let Some(value) = last.strip_prefix('@') {
            let parsed = Priority::parse(value)?;
            if priority.is_some() {
                return Err(AppError::InvalidArgs(
                    "task can only have one priority tag".to_string(),
                ));
            }
            priority = Some(parsed);
            words.pop();
            continue;
        }

        break;
    }

    labels.reverse();
    Ok((words.join(" "), priority, labels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_todo_contents() {
        let list = TodoList::parse("[ ] ship feature @high #backend\n  [x] write docs\n").unwrap();
        assert_eq!(list.tasks.len(), 2);
        assert!(!list.tasks[0].done);
        assert!(list.tasks[1].done);
        assert_eq!(list.tasks[0].indent, 0);
        assert_eq!(list.tasks[1].indent, 1);
        assert_eq!(list.tasks[0].text, "ship feature");
        assert_eq!(list.tasks[0].priority, Some(Priority::High));
        assert_eq!(list.tasks[0].labels, vec!["backend"]);
        assert_eq!(list.tasks[0].render_text(), "ship feature @high #backend");
    }

    #[test]
    fn adds_child_after_parent_subtree() {
        let mut list = TodoList::parse("[ ] parent\n  [ ] existing child\n[ ] sibling\n").unwrap();
        let index = list
            .add_child_with_metadata(1, "new child".to_string(), None, Vec::new())
            .unwrap();
        assert_eq!(index, 3);
        assert_eq!(list.tasks[2].text, "new child");
        assert_eq!(list.tasks[2].indent, 1);
        assert_eq!(list.tasks[3].text, "sibling");
    }

    #[test]
    fn adds_task_metadata() {
        let mut list = TodoList::default();
        list.add_with_metadata(
            "api fix".to_string(),
            Some(Priority::Medium),
            vec!["backend".to_string(), "api".to_string()],
        )
        .unwrap();

        assert_eq!(list.tasks[0].render_text(), "api fix @medium #backend #api");
    }

    #[test]
    fn moves_task_with_subtree() {
        let mut list = TodoList::parse(
            "[ ] first\n[ ] second\n  [ ] second child\n[ ] third\n",
        )
        .unwrap();
        list.move_task(4, 2).unwrap();
        assert_eq!(list.tasks[0].text, "first");
        assert_eq!(list.tasks[1].text, "third");
        assert_eq!(list.tasks[2].text, "second");
        assert_eq!(list.tasks[3].text, "second child");
    }

    #[test]
    fn prunes_completed_tasks_without_open_descendants() {
        let mut list = TodoList::parse(
            "[x] done\n  [x] done child\n[ ] open\n[x] mixed\n  [ ] open child\n",
        )
        .unwrap();
        let pruned = list.prune_completed();
        assert_eq!(pruned, 2);
        assert_eq!(list.tasks.len(), 3);
        assert_eq!(list.tasks[0].text, "open");
        assert_eq!(list.tasks[1].text, "mixed");
        assert_eq!(list.tasks[2].text, "open child");
    }

    #[test]
    fn returns_task_subtree() {
        let list =
            TodoList::parse("[ ] parent\n  [ ] child\n    [ ] grandchild\n[ ] sibling\n").unwrap();
        let subtree = list.subtree(1).unwrap();
        assert_eq!(subtree.len(), 3);
        assert_eq!(subtree[2].text, "grandchild");
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
