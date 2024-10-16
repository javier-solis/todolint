use anyhow::{Context, Result};
use chrono::Utc;
use git2::Repository;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use utils::print_json;
mod line_analyzer;
mod path_analyzer;
mod path_analyzer_types;
use path_analyzer_types::{
    AnalysisConfig, AnalysisResult, DirAnalysisConfig, DirectoryAnalysis, FileAnalysis,
    FileAnalysisConfig, FileBlameContext, FileMetadata,
};
mod line_analyzer_types;
use line_analyzer_types::TodoCommentResult;
mod utils;
use line_analyzer::LineAnalyzer;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let path = Path::new("test/one_valid.txt");
    let repo = Repository::discover(path).unwrap();

    let file_analysis_config = FileAnalysisConfig {
        repo: Some(&repo),
        include_files: None,
    };

    let dir_analysis_config = DirAnalysisConfig {
        file_analysis_config: file_analysis_config,
        exclude_dirs: None,
    };

    let analysis_config = AnalysisConfig::new_from_dir_config(&dir_analysis_config);
    // let analysis_config = AnalysisConfig::default();

    let analysis = analyze_path(&path, &analysis_config)?;
    print_json(&analysis);

    Ok(())
}

/// Entry point function.
fn analyze_path(path: &Path, analysis_config: &AnalysisConfig) -> Result<AnalysisResult> {
    if path.is_dir() {
        let dir_analysis_config = &analysis_config.dir_analysis_config;

        Ok(AnalysisResult::Directory(analyze_dir(
            path,
            &dir_analysis_config,
        )))
    } else if path.is_file() {
        let file_analysis_config = &analysis_config.file_analysis_config;

        Ok(AnalysisResult::File(analyze_file(
            path,
            &file_analysis_config,
        )?))
    } else {
        Err(anyhow::anyhow!("Path is neither a file nor a directory"))
    }
}

fn analyze_dir(dirpath: &Path, dir_analysis_config: &DirAnalysisConfig) -> DirectoryAnalysis {
    let file_analyses: Vec<FileAnalysis> = WalkDir::new(dirpath)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            analyze_file(path, &dir_analysis_config.file_analysis_config).ok()
        })
        .collect();

    let total_files_scanned = file_analyses.len();

    DirectoryAnalysis {
        total_files_scanned,
        last_scan_on: Utc::now(),
        file_analyses,
    }
}

fn analyze_file(filepath: &Path, config: &FileAnalysisConfig) -> Result<FileAnalysis> {
    let file = File::open(filepath).context("Failed to open file")?;
    let metadata = file.metadata().context("Failed to get file metadata")?;
    let reader = BufReader::new(file);

    // todo: create with a FileAnalysis::new?
    let mut file_analysis = FileAnalysis {
        metadata: FileMetadata {
            filepath: filepath.to_path_buf(),
            last_modified: metadata.modified()?.into(),
        },
        valids: None,
        invalids: None,
    };

    let file_blame_context = config
        .repo
        .and_then(|repo| FileBlameContext::new(repo, filepath).ok());

    let line_analyzer_obj = LineAnalyzer::new(file_blame_context.as_ref())?;

    for (line_number, line) in reader.lines().enumerate() {
        let line = line.context("Failed to read line")?;

        if let Some(processed_line) = line_analyzer_obj.process(&line, line_number)? {
            match processed_line {
                TodoCommentResult::Valid(comment) => {
                    file_analysis
                        .valids
                        .get_or_insert_with(Vec::new)
                        .push(comment);
                }
                TodoCommentResult::Invalid(comment) => {
                    file_analysis
                        .invalids
                        .get_or_insert_with(Vec::new)
                        .push(comment);
                }
            }
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
        let dir_analysis_config = DirAnalysisConfig::default();
        let result = analyze_dir(test_dir, &dir_analysis_config);

        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);

        Ok(())
    }

    #[test]
    fn test_analyze_file_valid() -> Result<()> {
        let filename = Path::new("test/valid.txt");
        let file_analysis_config = FileAnalysisConfig::default();
        let analysis = analyze_file(filename, &file_analysis_config)?;

        assert!(analysis.invalids.is_none(), "Expected no invalid todos");
        Ok(())
    }

    #[test]
    fn test_analyze_file_invalid() -> Result<()> {
        let filename = Path::new("test/invalid.txt");
        let file_analysis_config = FileAnalysisConfig::default();
        let analysis = analyze_file(filename, &file_analysis_config)?;

        assert!(analysis.valids.is_none(), "Expected no valid todos");
        Ok(())
    }
}
