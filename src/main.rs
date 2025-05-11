mod cli_options;
mod config_util;
mod io_util;
mod path_util;

use std::{env::current_dir, fs, path::PathBuf, process::exit, sync::Arc, time::Instant, vec};

use clap::Command;
use cli_options::{cli_args, parse_cli_options, CliOptions};
use htmd::{options::Options, HtmlToMarkdown};

use io_util::{resolve_input, resolve_output, Input, Output};
use path_util::common_ancestor;
use tokio::task::JoinHandle;

const ABOUT: &'static str = r#"
The command line tool for htmd, an HTML to Markdown converter

Examples: 
  htmd # Read input from stdin
  htmd index.html
  htmd --input ./pages --output ./pages/md
  htmd -i *.html -o ./md"#;

fn main() {
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

    let CliOptions {
        converter_options: options,
        ignored_tags,
        flatten_output,
        scripting_enabled,
    } = parse_cli_options(&matches);

    let input = resolve_input(&matches);
    let output = resolve_output(&matches);

    let converter = new_converter(ignored_tags, options, scripting_enabled);

    match input {
        Input::Stdin(text) => convert_text(&converter, &text, &output),
        Input::Fs(files) => {
            convert_files(converter, &files, &output, flatten_output);
            if output != Output::Stdout {
                println!("Converted {} file(s) in {:?}.", files.len(), now.elapsed());
            }
        }
    }
}

fn convert_files(
    converter: HtmlToMarkdown,
    files: &Vec<PathBuf>,
    output: &Output,
    flatten_output: bool,
) {
    match output {
        Output::Stdout => {
            if files.len() > 1 {
                let cwd = current_dir().expect("Cannot get current dir.");
                let paths = files
                    .iter()
                    .map(|file| format!("  {:?}", file.strip_prefix(&cwd).unwrap_or(file)))
                    .collect::<Vec<String>>()
                    .join("\n");
                eprintln!(
                    "Output to stdout doesn't support multiple files as the input.\n\n\
                    Input files:\n{}\n\n\
                    Try to use a folder as the output:\n  --output converted",
                    paths
                );
                exit(1);
            } else {
                let file = &files[0];
                let text = fs::read_to_string(file)
                    .expect(format!("Failed to read file: {:?}", file).as_str());
                convert_text(&converter, &text, &Output::Stdout);
            }
        }
        Output::Fs(output) => {
            let len = files.len();
            if len == 0 {
                println!("Nothing to convert.");
                exit(0);
            } else if len == 1 {
                let file = &files[0];
                let output_as_dir = !output.extension().is_some();
                let base_dir = &file.parent().unwrap().to_path_buf();
                convert_file(
                    file,
                    &converter,
                    output_as_dir,
                    flatten_output,
                    base_dir,
                    output,
                );
            } else {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    convert_multiple_and_write(converter, files, output, flatten_output).await;
                });
            }
        }
    }
}

fn convert_text(converter: &HtmlToMarkdown, text: &str, output: &Output) {
    let md = converter
        .convert(text)
        .expect("Failed to parse html from text");
    match output {
        Output::Stdout => print!("{}", md),
        Output::Fs(file) => {
            if file.exists() && file.is_dir() {
                eprintln!("Output cannot be a directory.");
                exit(1);
            }
            fs::write(file, md).expect(format!("Failed to write to file: {:?}", file).as_str())
        }
    }
}

async fn convert_multiple_and_write(
    converter: HtmlToMarkdown,
    files: &Vec<PathBuf>,
    output: &PathBuf,
    flatten_output: bool,
) {
    if output.exists() && output.is_file() {
        eprintln!("Multiple input files with non-directory output is unsupported.");
        exit(1);
    }

    if !output.exists() {
        fs::create_dir_all(output).expect(format!("Cannot create dir: {:?}", output).as_str());
    }

    let converter = Arc::new(converter);

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
                &converter_clone,
                true,
                flatten_output,
                &base_dir_clone,
                &output_clone,
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

fn convert_file(
    file: &PathBuf,
    converter: &HtmlToMarkdown,
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

fn new_converter(
    ignored_tags: Option<Vec<String>>,
    options: Options,
    scripting_enabled: bool,
) -> HtmlToMarkdown {
    let mut builder = HtmlToMarkdown::builder()
        .options(options)
        .scripting_enabled(scripting_enabled);

    if let Some(ignored_tags) = ignored_tags {
        builder = builder.skip_tags(ignored_tags.iter().map(|tag| tag.as_str()).collect());
    }
    builder.build()
}
