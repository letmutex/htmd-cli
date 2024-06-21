#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        io::{self, Write},
        path::{Path, PathBuf},
        process::{Command, Stdio},
    };

    struct ExecResult {
        exit_code: i32,
        stdout: String,
        stderr: String,
    }

    #[test]
    fn print_version() {
        let result = exec(vec!["-v"]);
        assert_eq!(env!("CARGO_PKG_VERSION"), result.stdout.trim());
    }

    #[test]
    fn stdin_in_stdout_out() {
        let html = "<h1>Hello</h1>";
        let result = exec_with_input(Some(html), vec![]);
        assert_eq!(result.exit_code, 0);
        assert_eq!("# Hello", result.stdout);
    }

    #[test]
    fn stdin_in_stdout_out_with_explicit_input() {
        let html = "<h1>Hello</h1>";
        let result = exec_with_input(Some(html), vec!["--input", "-"]);
        assert_eq!(result.exit_code, 0);
        assert_eq!("# Hello", result.stdout);
    }

    #[test]
    fn stdin_in_file_out() {
        let html = "<h1>Hello</h1>";
        let result =
            exec_with_temp_fs_and_input(Some(html), vec!["--output", "output.md"], |dir| {
                let output = dir.join("output.md");
                assert!(output.exists());
                assert_eq!("# Hello", fs::read_to_string(output).unwrap());
            });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn stdin_in_folder_out() {
        let html = "<h1>Hello</h1>";
        let result = exec_with_input(Some(html), vec!["-", "--output", "./"]);
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("Output cannot be a directory."));
    }

    #[test]
    fn unnamed_file_in_folder_out() {
        let result = exec_with_temp_fs(vec!["hello.html", "--output", "./"], |dir| {
            assert!(dir.join("hello.md").exists());
        });
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Converted"));
    }

    #[test]
    fn file_in_with_input_option() {
        let result = exec_with_temp_fs(vec!["--input", "hello.html", "--output", "./"], |dir| {
            assert!(dir.join("hello.md").exists());
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn file_in_not_found() {
        let result = exec_with_temp_fs(vec!["404.html"], |_| {});
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("File or directory does not exist"));
    }

    #[test]
    fn file_in_file_out() {
        let result = exec_with_temp_fs(
            vec!["hello.html", "--output", "hello_converted.md"],
            |dir| {
                let file = dir.join("hello_converted.md");
                assert!(file.exists());
                assert!(file.is_file());
            },
        );
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn file_in_folder_out() {
        let result = exec_with_temp_fs(vec!["hello.html", "--output", "converted"], |dir| {
            let file = dir.join("converted").join("hello.md");
            assert!(file.exists());
            assert!(file.is_file());
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn folder_in_folder_out() {
        let result = exec_with_temp_fs(vec!["./sub-folder2", "--output", "./"], |dir| {
            let html_count = count_dir_file_count(&dir.join("./sub-folder2"), "html", true);
            let md_count = count_dir_file_count(&dir, "md", true);
            assert_eq!(html_count, md_count);
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn folder_in_default_out() {
        let result = exec_with_temp_fs(vec!["./**/*.html"], |_| {});
        assert_eq!(result.exit_code, 1);
        assert!(result
            .stderr
            .contains("Output to stdout doesn't support multiple files as the input."))
    }

    #[test]
    fn folder_in_stdout_out() {
        let result = exec_with_temp_fs(vec!["./**/*.html", "--output", "-"], |_| {});
        assert_eq!(result.exit_code, 1);
        assert!(result
            .stderr
            .contains("Output to stdout doesn't support multiple files as the input."))
    }

    #[test]
    fn glob_in_folder_out_hierarchy() {
        let result = exec_with_temp_fs(vec!["**/*.html", "--output", "./"], |dir| {
            let sub_folders = vec!["./", "sub-folder", "sub-folder2"];
            for sub_folder in sub_folders {
                let html_count = count_dir_file_count(&dir.join(sub_folder), "html", false);
                let md_count = count_dir_file_count(&dir.join(sub_folder), "md", false);
                assert_eq!(html_count, md_count);
            }
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn glob_in_folder_out_flatten() {
        let result = exec_with_temp_fs(
            vec!["**/*.html", "--flatten-output", "--output", "./"],
            |dir| {
                let html_count = count_dir_file_count(&dir, "html", true);
                let md_count = count_dir_file_count(&dir, "md", false);
                assert_eq!(html_count, md_count);
            },
        );
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn read_options_toml() {
        let result = exec_with_temp_fs(vec!["hello.html", "--config", "cli-options.toml"], |_| {});
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello World!\n============"))
    }

    fn exec(args: Vec<&str>) -> ExecResult {
        exec_with_input(None, args)
    }

    fn exec_with_input(input_text: Option<&str>, args: Vec<&str>) -> ExecResult {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        if let Some(input_text) = input_text {
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_text.as_bytes())
                    .expect("Failed to write to stdin");
            }
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        ExecResult {
            exit_code: output.status.code().unwrap(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    fn exec_with_temp_fs(args: Vec<&str>, verify: impl FnOnce(PathBuf) -> ()) -> ExecResult {
        exec_with_temp_fs_and_input(None, args, verify)
    }

    fn exec_with_temp_fs_and_input(
        input_text: Option<&str>,
        args: Vec<&str>,
        verify: impl FnOnce(PathBuf) -> (),
    ) -> ExecResult {
        let tests_dir = env::current_dir().unwrap().join("tests");
        let html_dir = tests_dir.join("html");
        let temp_dir = tests_dir
            .join("temp")
            .join(format!("{}", uuid::Uuid::new_v4().to_string()));
        // Copy html dir temp dir
        copy_dir(&html_dir, &temp_dir)
            .expect(format!("Cannot setup temp dir: {:?}", temp_dir).as_str());
        fs::copy(
            tests_dir.join("cli-options.toml"),
            &temp_dir.join("cli-options.toml"),
        )
        .unwrap();

        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(args)
            .current_dir(&temp_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        if let Some(input_text) = input_text {
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(input_text.as_bytes())
                    .expect("Failed to write to stdin");
            }
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        let result = ExecResult {
            exit_code: output.status.code().unwrap(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };

        verify(temp_dir.clone());

        fs::remove_dir_all(temp_dir.clone())
            .expect(format!("Cannot delete temp dir: {}", temp_dir.to_str().unwrap()).as_str());

        result
    }

    fn copy_dir(source: &Path, target: &Path) -> io::Result<()> {
        if !target.exists() {
            fs::create_dir_all(target)?;
        }

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_name = entry_path.file_name().unwrap();

            let target_path = target.join(file_name);

            if entry_path.is_dir() {
                copy_dir(&entry_path, &target_path)?;
            } else {
                fs::copy(&entry_path, &target_path)?;
            }
        }

        Ok(())
    }

    fn count_dir_file_count(dir: &PathBuf, ext: &str, recursively: bool) -> usize {
        let mut count: usize = 0;
        for entry in fs::read_dir(dir).unwrap() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if recursively {
                        count += count_dir_file_count(&path, ext, true);
                    }
                } else {
                    if path.extension().unwrap().to_str().unwrap() == ext {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}
