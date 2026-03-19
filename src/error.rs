use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    InvalidArgs(String),
    TodoNotFound(PathBuf),
    AlreadyInitialized(PathBuf),
    InvalidTaskIndex { index: usize, len: usize },
    MalformedTodoLine { line: usize, content: String },
    EmptyTask,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::InvalidArgs(message) => write!(f, "{message}"),
            Self::TodoNotFound(path) => write!(
                f,
                "no `.todo` file found from {} upward; run `to init` in your project root",
                path.display()
            ),
            Self::AlreadyInitialized(path) => {
                write!(f, "a `.todo` file already exists at {}", path.display())
            }
            Self::InvalidTaskIndex { index, len } => {
                write!(f, "task {index} is out of range; expected a number between 1 and {len}")
            }
            Self::MalformedTodoLine { line, content } => write!(
                f,
                "invalid `.todo` format on line {line}: expected `[ ] task` or `[x] task`, got `{content}`"
            ),
            Self::EmptyTask => write!(f, "task text cannot be empty"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
