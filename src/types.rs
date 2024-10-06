use chrono::{DateTime, Utc};
use email_address::EmailAddress;
use serde::Serialize;
use strum_macros::{AsRefStr, Display};

#[derive(Serialize, Debug)]
pub enum AnalysisResult {
    Directory(DirectoryAnalysis),
    File(FileAnalysis),
}

#[derive(Serialize, Debug)]
pub struct DirectoryAnalysis {
    pub total_files_scanned: usize,
    pub last_scan_on: DateTime<Utc>,
    pub file_analyses: Vec<FileAnalysis>,
}

#[derive(Serialize, Debug)]
pub struct FileAnalysis {
    pub metadata: FileMetadata,
    pub valids: Vec<ValidTodoComment>,
    pub invalids: Vec<InvalidTodoComment>,
}

#[derive(Serialize, Debug)]
pub struct FileMetadata {
    pub filepath: String,
    pub last_modified: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub enum TodoCommentResult {
    Valid(ValidTodoComment),
    Invalid(InvalidTodoComment),
    NotApplicable,
}

pub type ValidTodoComment = TodoCommentBase<ValidContent>;
pub type InvalidTodoComment = TodoCommentBase<InvalidContent>;

#[derive(Serialize, Debug)]
pub struct TodoCommentBase<T> {
    pub line: usize,
    #[serde(flatten)]
    pub line_info: T,
}

#[derive(Serialize, Debug)]
pub struct ValidContent {
    pub comment: String,
    pub delimiters: Vec<Delimiter>,
}

#[derive(Serialize, Debug)]
pub struct InvalidContent {
    pub full_text: String,
}

#[derive(Serialize, Debug)]
pub struct Delimiter {
    pub delimiter_type: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct BlameInfo {
    pub email: EmailAddress,
    pub timestamp: DateTime<Utc>,
}

#[derive(Display, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum CommentMarker {
    Todo,
}

#[derive(Display, AsRefStr)]
pub enum CaptureGroupNames {
    MarkerContent,
    CommentContent,
}
