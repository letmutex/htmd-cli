use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::exit,
};

use clap::ArgMatches;
use glob::glob;

#[derive(PartialEq)]
pub(crate) enum Input {
    Stdin(String),
    Fs(Vec<PathBuf>),
}

#[derive(PartialEq)]
pub(crate) enum Output {
    Stdout,
    Fs(PathBuf),
}

pub(crate) fn resolve_input(matches: &ArgMatches) -> Input {
    let read_stdin = || -> Input {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .expect("Cannot read text from stdin.");
        Input::Stdin(text)
    };

    let input_from_arg = |id: &str| -> Option<Input> {
        let Some(input_arg) = matches.get_one::<String>(id) else {
            return None;
        };

        if input_arg == "-" {
            return Some(read_stdin());
        }

        let files = match get_html_files_from_input(&input_arg) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };

        Some(Input::Fs(files))
    };

    if let Some(input) = input_from_arg("input") {
        return input;
    }

    if let Some(input) = input_from_arg("input-unnamed") {
        return input;
    }

    read_stdin()
}

pub(crate) fn resolve_output(matches: &ArgMatches) -> Output {
    let Some(output) = matches.get_one::<String>("output") else {
        return Output::Stdout;
    };
    if output == "-" {
        return Output::Stdout;
    }
    Output::Fs(PathBuf::from(output))
}

fn is_glob_pattern(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
}

fn get_html_files_from_input(input: &str) -> Result<Vec<PathBuf>, String> {
    let path = if input == "." || input == "./" {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(input))
    } else {
        PathBuf::from(input)
    };

    // 1. Literal file: trust explicit file inputs directly
    if path.is_file() {
        return Ok(vec![path]);
    }

    // 2. Literal directory: scan for all HTML/XHTML files
    if path.is_dir() {
        let files = read_dir_html_files(&path)
            .map_err(|e| format!("Cannot read directory '{}': {}", input, e))?;
        if files.is_empty() {
            return Err(format!("No HTML files found in directory: {}", input));
        }
        return Ok(files);
    }

    // 3. Glob pattern
    if is_glob_pattern(input) {
        let mut files: Vec<PathBuf> = Vec::new();
        let entries = glob(input).map_err(|e| format!("Invalid glob pattern '{}': {}", input, e))?;
        for entry in entries {
            match entry {
                Ok(p) => {
                    if possible_html_file(&p) {
                        files.push(p);
                    }
                }
                Err(e) => eprintln!("Error while matching file: {}", e),
            }
        }
        if files.is_empty() {
            return Err(format!("No matching HTML files found for pattern: {}", input));
        }
        files.sort();
        return Ok(files);
    }

    Err(format!("File or directory does not exist: {}", input))
}

fn read_dir_html_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let child = entry.path();
        if possible_html_file(&child) {
            files.push(child);
        }
    }
    files.sort();
    Ok(files)
}

fn possible_html_file(path: &Path) -> bool {
    path.is_file() && is_html_extension(path)
}

fn is_html_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "html" | "htm" | "xhtml" | "xhtm"
    )
}
