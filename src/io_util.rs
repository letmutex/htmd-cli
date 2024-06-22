use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
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

        let files = get_html_files_from_input(&input_arg);

        if files.is_empty() {
            eprintln!("File or directory does not exists: {}", input_arg);
            exit(1);
        }

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

fn get_html_files_from_input(pattern: &str) -> Vec<PathBuf> {
    if pattern == "." || pattern == "./" {
        // Fast path for the current dir
        return read_dir_html_files(&std::env::current_dir().unwrap());
    }
    // Parse input as glob
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in glob(pattern).expect("Failed to read input") {
        match entry {
            Ok(path) => {
                let Some(ext) = path.extension() else {
                    continue;
                };
                let Some(etx) = ext.to_str() else {
                    continue;
                };
                let ext = etx.to_lowercase();
                if ext == "html" || ext == "htm" {
                    files.push(path);
                }
            }
            Err(e) => eprintln!("Error while matching file: {}", e),
        }
    }
    if !files.is_empty() {
        return files;
    }
    // Treat the input as a file or a directory
    let file = PathBuf::from(pattern);
    if !file.exists() {
        eprintln!("File or directory does not exist: {:?}", file);
        exit(1);
    }
    if file.is_dir() {
        read_dir_html_files(&file)
    } else {
        vec![file]
    }
}

fn read_dir_html_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        if let Ok(entry) = entry {
            let child = entry.path();
            if possible_html_file(&child) {
                files.push(child);
            }
        }
    }
    files
}

fn possible_html_file(path: &PathBuf) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };
    let Some(ext) = ext.to_str() else {
        return false;
    };
    let ext = ext.to_lowercase();
    ext == "html" || ext == "htm"
}
