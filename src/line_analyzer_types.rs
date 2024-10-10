use chrono::{DateTime, Utc};
use email_address::EmailAddress;
use serde::Serialize;
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
