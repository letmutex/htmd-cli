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
htmd test.html # Will generate test.md
htmd --input test.html --output converted.md
```

### Folders

```bash
htmd pages --output converted
```

### Glob patterns

```bash
htmd pages/**/*.htm
```

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

Install the package

```bash
cargo install htmd-cli
```

Use `cargo-htmd` or `cargo htmd`

```bash
cargo-htmd hello.html
cargo htmd -i hello.html # Require explicit input option
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
