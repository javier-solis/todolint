use crate::line_analyzer_types::{InvalidTodoComment, ValidTodoComment};
use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::{Blame, BlameOptions, Repository};
use serde::Serialize;
use std::path::{Path, PathBuf};


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

// The lifetime `'repo` ensures that the FileBlameContext doesn't outlive the `Repository` its
// borrowing from (which is an arg in the `new` method).
pub struct FileBlameContext<'repo> {
    pub repo: &'repo Repository,
    pub blame: Blame<'repo>,
}

impl<'repo> FileBlameContext<'repo> {
    // todo: I don't like that Repository is a param, but its that way bc of ownership rules
    // refactor in the future
    pub fn new(repo: &'repo Repository, file_path: &Path) -> Result<Self> {
        let mut blame_opts = BlameOptions::new();
        let blame = repo.blame_file(file_path, Some(&mut blame_opts))?;

        Ok(FileBlameContext { repo, blame })
    }
}
