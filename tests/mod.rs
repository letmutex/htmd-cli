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
        let result = exec_with_temp_fs(
            vec!["hello.html", "--options-file", "cli-options.toml"],
            |_| {},
        );
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello World!\n============"))
    }

    #[test]
    fn test_ul_bullet_spacing() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let result = exec_with_input(Some(html), vec![]);
        assert_eq!(result.stdout, "*   Item 1\n*   Item 2");

        let result = exec_with_input(Some(html), vec!["--ul-bullet-spacing", "5"]);
        assert_eq!(result.stdout, "*     Item 1\n*     Item 2");
    }

    #[test]
    fn test_ol_number_spacing() {
        let html = "<ol><li>Item A</li><li>Item B</li></ol>";
        let result = exec_with_input(Some(html), vec![]);
        assert_eq!(result.stdout, "1.  Item A\n2.  Item B");

        let result = exec_with_input(Some(html), vec!["--ol-number-spacing", "4"]);
        assert_eq!(result.stdout, "1.    Item A\n2.    Item B");
    }

    #[test]
    fn test_scripting_enabled() {
        let html = "<noscript><div>Content</div></noscript>";
        let result = exec_with_input(Some(html), vec![]);
        assert_eq!(result.stdout.trim(), "\\<div>Content\\</div>");

        let html = "<noscript><div>Content</div></noscript>";
        let result = exec_with_input(Some(html), vec!["--scripting-enabled", "false"]);
        assert_eq!(result.stdout.trim(), "Content");
    }

    #[test]
    fn test_link_style_inlined_prefer_autolinks() {
        let html = r#"<a href="https://example.com">https://example.com</a>"#;
        let result = exec_with_input(Some(html), vec!["--link-style", "inlined-prefer-autolinks"]);
        assert_eq!(result.stdout.trim(), "<https://example.com>");
    }

    #[test]
    fn glob_in_xhtml_files() {
        let result = exec_with_temp_fs_and_setup(
            |dir| {
                fs::write(dir.join("page1.xhtml"), "<h1>Page 1</h1>").unwrap();
                fs::write(dir.join("page2.xhtml"), "<h2>Page 2</h2>").unwrap();
            },
            vec!["--ignored-tags", "title,style", "-i", "*.xhtml", "-o", "./"],
            |dir| {
                assert!(dir.join("page1.md").exists());
                assert!(dir.join("page2.md").exists());
                assert_eq!(
                    "# Page 1",
                    fs::read_to_string(dir.join("page1.md")).unwrap().trim()
                );
                assert_eq!(
                    "## Page 2",
                    fs::read_to_string(dir.join("page2.md")).unwrap().trim()
                );
            },
        );
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn single_xhtml_file_in_folder_out() {
        let result = exec_with_temp_fs_and_setup(
            |dir| {
                fs::write(dir.join("single.xhtml"), "<h1>Single XHTML</h1>").unwrap();
            },
            vec!["single.xhtml", "--output", "./"],
            |dir| {
                assert!(dir.join("single.md").exists());
                assert_eq!(
                    "# Single XHTML",
                    fs::read_to_string(dir.join("single.md")).unwrap().trim()
                );
            },
        );
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn folder_in_with_xhtml_files() {
        let result = exec_with_temp_fs_and_setup(
            |dir| {
                let xhtml_dir = dir.join("xhtml-folder");
                fs::create_dir_all(&xhtml_dir).unwrap();
                fs::write(xhtml_dir.join("doc.xhtml"), "<p>Doc</p>").unwrap();
            },
            vec!["-i", "./xhtml-folder", "--output", "./output"],
            |dir| {
                assert!(dir.join("output").join("doc.md").exists());
            },
        );
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn glob_no_matches_error() {
        let result = exec_with_temp_fs(vec!["-i", "*.xhtml", "--output", "./"], |_| {});
        assert_eq!(result.exit_code, 1);
        assert!(result
            .stderr
            .contains("No matching HTML files found for pattern: *.xhtml"));
    }

    #[test]
    fn empty_dir_error() {
        let result = exec_with_temp_fs_and_setup(
            |dir| {
                fs::create_dir_all(dir.join("empty-folder")).unwrap();
            },
            vec!["-i", "./empty-folder", "--output", "./"],
            |_| {},
        );
        assert_eq!(result.exit_code, 1);
        assert!(result
            .stderr
            .contains("No HTML files found in directory: ./empty-folder"));
    }

    fn exec(args: Vec<&str>) -> ExecResult {
        exec_with_input(None, args)
    }

    fn exec_with_input(input_text: Option<&str>, args: Vec<&str>) -> ExecResult {
        let mut child = Command::new(env!("CARGO_BIN_EXE_htmd"))
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
        } else {
            drop(child.stdin.take());
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        ExecResult {
            exit_code: output.status.code().unwrap(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    fn exec_with_temp_fs(args: Vec<&str>, verify: impl FnOnce(PathBuf)) -> ExecResult {
        exec_with_temp_fs_and_setup(|_| {}, args, verify)
    }

    fn exec_with_temp_fs_and_setup(
        setup: impl FnOnce(&Path),
        args: Vec<&str>,
        verify: impl FnOnce(PathBuf),
    ) -> ExecResult {
        exec_with_temp_fs_and_input_and_setup(None, setup, args, verify)
    }

    fn exec_with_temp_fs_and_input(
        input_text: Option<&str>,
        args: Vec<&str>,
        verify: impl FnOnce(PathBuf),
    ) -> ExecResult {
        exec_with_temp_fs_and_input_and_setup(input_text, |_| {}, args, verify)
    }

    fn exec_with_temp_fs_and_input_and_setup(
        input_text: Option<&str>,
        setup: impl FnOnce(&Path),
        args: Vec<&str>,
        verify: impl FnOnce(PathBuf),
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

        setup(&temp_dir);

        let mut child = Command::new(env!("CARGO_BIN_EXE_htmd"))
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
        } else {
            drop(child.stdin.take());
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
