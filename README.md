<div align="center">
  <h1>htmd-cli</h1>
  <p>The command line tool for <a href="https://github.com/letmutex/htmd">htmd</a></p>
</div>

### Features

- Minimal, A 2MB+ binary is all you need
- Batch, both file, directory and glob pattern are supported
- Fast, it takes only ~2s to convert 200 html files (~70MB in total size, see the benchmark [README](./bench/README.md))

# Usages

### Basic

```bash
# Will write output to stdout
htmd test.html

# Explicit input option
htmd --input test.html

# Write output to test.md by shell
htmd test.html > test.md

# Write output to test.md internally
htmd test.html --output ./

# Read html files from a directory
htmd ./pages -o converted
```

### Inputs

Stdin (`-` as the filename), file, directory, and glob pattern are supported.

This default input is `stdin`, so after you type only `htmd` it will wait for input, to submit typing, press <kbd>Ctrl</kbd> + <kbd>D</kbd> (<kbd>Ctrl</kbd> + <kbd>Z</kbd> on Windows).

Example inputs:

- Stdin: `-`, `< page.html`
- File: `page.html`, `index.html`
- Directory: `pages`, `./folder`
- Glob pattern: `pages/\*\*/\*.html`, `./\*.html`

### Output

Stdout (`-` as the filename), file, and directory are supported. Defaults to stdout.

You cannot set output as stdout when you have multiple input files.

Example outputs:

- Stdout: `-`
- File: `output.md`,
- Directory: `output`, `./converted`

### With conversion options

```bash
htmd test.html --ignored-tags "head,script,style" --heading-style setex
```

### Flatten output

By default, when converting files using glob patterns such as `pages/**/*.html`, output files will follow the original folder hierarchy, to flatten output files, use `--flatten-output`.

```bash
htmd pages/**/*.html --output converted
```

### Load options form toml file

You can save your options to a toml config file

```toml
# htmd-options.toml
[options]
ignored-tags =["head", "script", "style"]
heading-style = "setex"
```

Then load them using `--config`

```
htmd test.html --config htmd-options.toml
```

# Install

### Cargo

```bash
cargo install htmd-cli
```

### Binaries

You can download binaries from [GitHub - Releases](https://github.com/letmutex/htmd-cli/releases)

# License

```
Copyright 2024 letmutex

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
