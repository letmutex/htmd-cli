#[cfg(test)]
mod tests {
    use std::{
        env, fs, io,
        path::{Path, PathBuf},
        process::Command,
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
    fn no_options() {
        let result = exec(vec![]);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("No input."));
    }

    #[test]
    fn unnamed_file_input() {
        let result = exec_with_temp_fs(vec!["hello.html"], |dir| {
            assert!(dir.join("hello.md").exists());
        });
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Converted"));
    }

    #[test]
    fn with_input_option() {
        let result = exec_with_temp_fs(vec!["--input", "hello.html"], |dir| {
            assert!(dir.join("hello.md").exists());
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn file_input_not_exists() {
        let result = exec_with_temp_fs(vec!["404.html"], |_| {});
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("File or directory does not exist"));
    }

    #[test]
    fn file_input_with_file_output() {
        let result = exec_with_temp_fs(
            vec!["hello.html", "--output", "hello_converted.md"],
            |dir| {
                assert!(dir.join("hello_converted.md").exists());
            },
        );
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn folder_input() {
        let result = exec_with_temp_fs(vec!["./sub-folder"], |dir| {
            let html_count = count_dir_file_count(&dir.join("sub-folder"), "html", true);
            let md_count = count_dir_file_count(&dir, "md", true);
            assert_eq!(html_count, md_count);
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn glob_input_hierarchy() {
        let result = exec_with_temp_fs(vec!["**/*.html"], |dir| {
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
    fn glob_input_flatten() {
        let result = exec_with_temp_fs(vec!["**/*.html", "--flatten-output"], |dir| {
            let html_count = count_dir_file_count(&dir, "html", true);
            let md_count = count_dir_file_count(&dir, "md", false);
            assert_eq!(html_count, md_count);
        });
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn read_options_toml() {
        let result = exec_with_temp_fs(vec!["hello.html", "--config", "cli-options.toml"], |dir| {
            let md_file = dir.join("hello.md");
            assert!(md_file.exists());
            let md = fs::read_to_string(md_file).unwrap();
            assert!(md.contains("Hello World!\n============"));
        });
        assert_eq!(result.exit_code, 0);
    }

    fn exec(args: Vec<&str>) -> ExecResult {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(args)
            .output()
            .unwrap();
        ExecResult {
            exit_code: output.status.code().unwrap(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    fn exec_with_temp_fs(args: Vec<&str>, verify: impl FnOnce(PathBuf) -> ()) -> ExecResult {
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

        let output = Command::new("cargo")
            .current_dir(temp_dir.clone())
            .arg("run")
            .arg("--")
            .args(args)
            .output()
            .unwrap();
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
