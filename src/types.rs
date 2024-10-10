use crate::line_analyzer_types::{InvalidTodoComment, ValidTodoComment};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;


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
