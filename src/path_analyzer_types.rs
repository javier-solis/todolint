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
    pub valids: Option<Vec<ValidTodoComment>>,
    pub invalids: Option<Vec<InvalidTodoComment>>,
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

impl<'a> AnalysisConfig<'a> {
    pub fn new_from_dir_config(dir_config: &'a DirAnalysisConfig<'a>) -> Self {
        Self {
            dir_analysis_config: DirAnalysisConfig::from_ref(dir_config),
            file_analysis_config: FileAnalysisConfig::from_ref(&dir_config.file_analysis_config),
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

/// Trait for creating a new instance of a type from a reference to itself.
pub trait FromRef<'a> {
    fn from_ref(t: &'a Self) -> Self;
}

impl<'a> Default for FileAnalysisConfig<'a> {
    fn default() -> Self {
        Self {
            repo: None,
            include_files: None,
        }
    }
}

impl<'a> FromRef<'a> for FileAnalysisConfig<'a> {
    fn from_ref(config: &'a Self) -> Self {
        Self {
            repo: config.repo,
            include_files: config.include_files,
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

impl<'a> FromRef<'a> for DirAnalysisConfig<'a> {
    fn from_ref(config: &'a Self) -> Self {
        Self {
            file_analysis_config: FileAnalysisConfig::from_ref(&config.file_analysis_config),
            exclude_dirs: config.exclude_dirs,
        }
    }
}
