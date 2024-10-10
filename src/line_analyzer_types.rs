use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use email_address::EmailAddress;
use git2::{BlameOptions, Repository};
use serde::Serialize;
use std::{path::Path, str::FromStr};
use strum_macros::{AsRefStr, Display, EnumIter};

// == Types ==

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
    // #[serde(flatten)]
    // pub blame_info: Option<BlameInfo>,
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
    pub delimiter_type: Delimiter,
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

#[derive(Serialize, Debug, PartialEq, EnumIter)]
pub enum Delimiter {
    Parentheses,
    Braces,
    Brackets,
    Angles,
}

// == Impl's ==

impl DelimiterChars {
    /// For quick destructuring.
    pub fn to_tuple(&self) -> (char, char) {
        (self.open, self.close)
    }
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

impl BlameInfo {
    fn new(file_path: &Path, line_number: usize) -> Result<BlameInfo> {
        let repo = Repository::discover(file_path)
            .with_context(|| format!("Failed to discover repository for file {:?}", file_path))?;

        // Get blame information for the specified file and line, then retrieve the corresponding commit
        let mut blame_opts = BlameOptions::new();
        let blame = repo
            .blame_file(file_path, Some(&mut blame_opts))
            .with_context(|| format!("Failed to get blame for file {:?}", file_path))?;
        let hunk = blame
            .get_line(line_number)
            .with_context(|| format!("No blame information found for line {}", line_number))?;
        let commit_id = hunk.final_commit_id();
        let commit = repo
            .find_commit(commit_id)
            .with_context(|| format!("Failed to find commit {}", commit_id))?;

        // From commit, extract author email and timestamp
        let author_email = commit
            .author()
            .email()
            .and_then(|email| EmailAddress::from_str(email).ok())
            .unwrap_or_else(|| EmailAddress::new_unchecked("unknown@example.com"));
        let timestamp = Utc
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .with_context(|| format!("Invalid timestamp: {}", commit.time().seconds()))?;

        Ok(BlameInfo {
            email: author_email,
            timestamp,
        })
    }
}
