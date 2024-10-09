use chrono::{DateTime, Utc};
use email_address::EmailAddress;
use serde::Serialize;
use std::path::PathBuf;
use strum_macros::{AsRefStr, Display, EnumIter};


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
    pub filepath: PathBuf,
    pub last_modified: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub enum TodoCommentResult {
    Valid(ValidTodoComment),
    Invalid(InvalidTodoComment),
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
    pub delimiters: Vec<DelimiterContent>,
}

#[derive(Serialize, Debug)]
pub struct InvalidContent {
    pub full_text: String,
}

#[derive(Serialize, Debug)]
pub struct DelimiterContent {
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

pub struct DelimiterChars {
    open: char,
    close: char,
}

impl DelimiterChars {
    /// For quick destructuring.
    pub fn to_tuple(&self) -> (char, char) {
        (self.open, self.close)
    }
}

#[derive(Debug, PartialEq, EnumIter)]
pub enum Delimiter {
    Parentheses,
    Braces,
    Brackets,
    Angles,
}

#[rustfmt::skip]
impl Delimiter {
    pub fn get_chars(&self) -> DelimiterChars {
        match self {
            Delimiter::Parentheses => DelimiterChars { open: '(', close: ')' },
            Delimiter::Braces => DelimiterChars { open: '{', close: '}' },
            Delimiter::Brackets => DelimiterChars { open: '[', close: ']' },
            Delimiter::Angles => DelimiterChars { open: '<', close: '>' },
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Delimiter::Parentheses => "parentheses",
            Delimiter::Braces => "braces",
            Delimiter::Brackets => "brackets",
            Delimiter::Angles => "angles",
        }
    }
}
