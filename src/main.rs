use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use strum::IntoEnumIterator;
use utils::{get_blame_info, print_json, print_todo_result};
mod line_analyzer;
mod types;
use types::{
    AnalysisResult, CaptureGroupNames, CommentMarker, Delimiter, DelimiterContent,
    DirectoryAnalysis, FileAnalysis, FileMetadata, InvalidContent, InvalidTodoComment,
    TodoCommentResult, ValidContent, ValidTodoComment,
};
mod utils;
use line_analyzer::LineAnalyzer;
use std::path::Path;
use walkdir::WalkDir;

extern crate chrono;

fn main() -> Result<()> {
    Ok(())
}
fn analyze_path(path: &Path) -> Result<AnalysisResult> {
    if path.is_dir() {
        Ok(AnalysisResult::Directory(analyze_dir(path)))
    } else if path.is_file() {
        Ok(AnalysisResult::File(analyze_file(path.to_str().unwrap())?))
    } else {
        Err(anyhow::anyhow!("Path is neither a file nor a directory"))
    }
}

fn analyze_dir(dir: &Path) -> DirectoryAnalysis {
    let file_analyses: Vec<FileAnalysis> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            analyze_file(path.to_str()?).ok()
        })
        .collect();

    let total_files_scanned = file_analyses.len();

    DirectoryAnalysis {
        total_files_scanned,
        last_scan_on: Utc::now(),
        file_analyses,
    }
}

fn analyze_file(filename: &str) -> Result<FileAnalysis> {
    let file = File::open(filename).context("Failed to open file")?;
    let metadata = file.metadata().context("Failed to get file metadata")?;
    let reader = BufReader::new(file);

    let mut file_analysis = FileAnalysis {
        metadata: FileMetadata {
            filepath: filename.to_string(),
            last_modified: metadata.modified()?.into(),
        },
        valids: Vec::new(),
        invalids: Vec::new(),
    };

    let line_analyzer_obj = LineAnalyzer::new()?;

    for (line_number, line) in reader.lines().enumerate() {
        let line = line.context("Failed to read line")?;

        let processed_line = line_analyzer_obj.process(&line, line_number)?;
        match processed_line {
            TodoCommentResult::Valid(comment) => {
                file_analysis.valids.push(comment);
            }
            TodoCommentResult::Invalid(comment) => {
                file_analysis.invalids.push(comment);
            }
            _ => {}
        }
    }

    Ok(file_analysis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_dir() -> Result<()> {
        let test_dir = Path::new("test");
        let result = analyze_dir(test_dir);

        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);

        Ok(())
    }

    #[test]
    fn test_analyze_file_valid() -> Result<()> {
        let filename = "test/valid.txt";
        let analysis = analyze_file(filename)?;

        assert_eq!(analysis.invalids.len(), 0, "Expected no invalid todos");
        Ok(())
    }

    #[test]
    fn test_analyze_file_invalid() -> Result<()> {
        let filename = "test/invalid.txt";
        let analysis = analyze_file(filename)?;

        assert_eq!(analysis.valids.len(), 0, "Expected no valid todos");
        Ok(())
    }
}
