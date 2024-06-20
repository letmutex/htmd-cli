mod cli_options;
mod config_util;
mod path_util;

use std::{fs, path::PathBuf, process::exit, sync::Arc, time::Instant, vec};

use clap::Command;
use cli_options::{cli_args, parse_cli_options, CliOptions};
use htmd::{options::Options, HtmlToMarkdown};

use glob::glob;
use path_util::common_ancestor;
use tokio::task::JoinHandle;

const ABOUT: &'static str = r#"
The command line tool for htmd, an HTML to Markdown converter

Examples: 
  htmd-cli index.html
  htmd-cli --input ./pages --output ./pages/md
  htmd-cli -i *.html -o ./md"#;

#[tokio::main]
async fn main() {
    let now = Instant::now();

    let matches = Command::new("htmd-cli")
        .about(ABOUT)
        .args(cli_args())
        .get_matches();

    if matches
        .get_one::<bool>("version")
        .is_some_and(|is_version| *is_version)
    {
        println!("{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    let input = matches.get_one::<String>("input").unwrap_or_else(|| {
        match matches.get_one::<String>("input-unnamed") {
            Some(input) => input,
            None => {
                println!("No input.\nUse -h or --help to print help messages.");
                exit(0);
            }
        }
    });

    let files = get_html_files_from_input(input);

    if files.is_empty() {
        eprintln!("File or directory does not exists: {}", input);
        exit(1);
    }

    let output = matches
        .get_one::<String>("output")
        .map(|path: &String| PathBuf::from(path))
        .unwrap_or(std::env::current_dir().unwrap());

    let CliOptions {
        converter_options: options,
        ignored_tags,
        flatten_output,
    } = parse_cli_options(&matches);

    convert(&files, &output, flatten_output, ignored_tags, options).await;

    println!("Converted {} file(s) in {:?}.", files.len(), now.elapsed());
}

async fn convert(
    files: &Vec<PathBuf>,
    output: &PathBuf,
    flatten_output: bool,
    ignored_tags: Option<Vec<String>>,
    options: Options,
) {
    if files.is_empty() {
        println!("Nothing to convert.");
        exit(0);
    }

    let output_as_dir = files.len() > 0;

    if output_as_dir && output.exists() && output.is_file() {
        eprintln!("Multiple input files with non-directory output is unsupported.");
        exit(1);
    }

    if output_as_dir && !output.exists() {
        fs::create_dir_all(output).expect(format!("Cannot create dir: {:?}", output).as_str());
    }

    let mut builder = HtmlToMarkdown::builder().options(options);

    if let Some(ignored_tags) = ignored_tags {
        builder = builder.skip_tags(ignored_tags.iter().map(|tag| tag.as_str()).collect());
    }

    let converter = Arc::new(builder.build());

    let base_dir = &common_ancestor(&files).unwrap();

    let mut handles: Vec<JoinHandle<()>> = vec![];

    for file in files {
        let file_clone = file.clone();
        let converter_clone = converter.clone();
        let base_dir_clone = base_dir.clone();
        let output_clone = output.clone();
        let handle = tokio::spawn(async move {
            convert_file(
                &file_clone,
                converter_clone,
                output_as_dir,
                flatten_output,
                &base_dir_clone,
                &output_clone,
            )
            .await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

async fn convert_file(
    file: &PathBuf,
    converter: Arc<HtmlToMarkdown>,
    output_as_dir: bool,
    flatten_output: bool,
    base_dir: &PathBuf,
    output: &PathBuf,
) {
    if !file.exists() {
        eprintln!("File does not exist: {}", file.to_str().unwrap());
        exit(1);
    }

    let html =
        fs::read_to_string(file).expect(format!("Cannot read file as text: {:?}", file).as_str());

    let md = converter
        .convert(&html)
        .expect(format!("Failed to parse html from file: {:?}", file).as_str());

    if output_as_dir {
        let filename_with_ext = file.file_stem().unwrap().to_str().unwrap();
        let output_filename = format!("{}.md", filename_with_ext);
        let output_file = if !flatten_output {
            let parent = file.parent().unwrap();
            if parent != base_dir {
                let relative = parent.strip_prefix(base_dir).unwrap();
                output.join(relative).join(output_filename)
            } else {
                output.join(output_filename)
            }
        } else {
            output.join(output_filename)
        };

        let output_dir = output_file.parent().unwrap();
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)
                .expect(format!("Cannot create output dir: {:?}", output_dir).as_str());
        }

        fs::write(output_file.clone(), &md)
            .expect(format!("Cannot write file: {:?}", output_file).as_str());
    } else {
        fs::write(output, &md).expect(format!("Cannot write file: {:?}", output).as_str());
    }
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
