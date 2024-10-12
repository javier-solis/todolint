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

/// Options for file analysis, wrapped in Option to indicate availability and user-enabled status
/// as determined by the parent entity.
pub struct FileAnalysisConfig<'a> {
    pub repo: Option<&'a Repository>,
    /// Additional file extensions to include. Do not include the leading dot.
    pub include_files: Option<&'static [&'static str]>,
}

/// Options for directory analysis, wrapped in Option to indicate availability and user-enabled
/// status as determined by the parent entity.
pub struct DirAnalysisConfig<'a> {
    pub file_analysis_config: FileAnalysisConfig<'a>,
    // Directories to exclude, relative to the root of the project.
    pub exclude_dirs: Option<&'static [&'static str]>,
}

/// Options for project analysis.
pub struct AnalysisConfig<'a> {
    pub dir_analysis_config: DirAnalysisConfig<'a>,
    pub file_analysis_config: FileAnalysisConfig<'a>,
}

impl<'a> Default for AnalysisConfig<'a> {
    fn default() -> Self {
        Self {
            dir_analysis_config: DirAnalysisConfig::default(),
            file_analysis_config: FileAnalysisConfig::default(),
        }
    }
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

impl<'a> Default for FileAnalysisConfig<'a> {
    fn default() -> Self {
        Self {
            repo: None,
            include_files: None,
        }
    }
}

impl<'a> Default for DirAnalysisConfig<'a> {
    fn default() -> Self {
        Self {
            file_analysis_config: FileAnalysisConfig::default(),
            exclude_dirs: None,
        }
    }
}
