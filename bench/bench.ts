import { $ } from "bun";
import * as path from "path";
import * as fs from "fs/promises";
import * as os from "os";
import { fetchBenchPages } from "./fetchBenchmarkPages";
import toml from "toml";

const SOURCE_PAGE_URL =
  "https://en.wikipedia.org/wiki/Rust_(programming_language)";

const BENCH_PAGE_COUNT = 200;

type InputFilesInfo = {
  fileCount: number;
  totalSize: number;
};

async function checkCommand(command: string) {
  try {
    await $`${command} --version > /dev/null`;
  } catch (e) {
    throw new Error(`${command} is not installed.`);
  }
}

await checkCommand("hyperfine");
await checkCommand("pandoc");

const sourceFileDir = path.dirname(__filename);
if (process.cwd() != path.join(sourceFileDir, "..")) {
  throw new Error("Please run this script in project's root directory.");
}

const benchPagesDir = path.join(process.cwd(), "bench/bench-pages");
if (!(await fs.exists(benchPagesDir))) {
  console.log("Fetching bench pages...");
  await fetchBenchPages(SOURCE_PAGE_URL, benchPagesDir, BENCH_PAGE_COUNT);
}

console.log("Building project (--release)...");

await $`cargo build --release`;

const htmdCmd =
  "cargo run --release -- ./bench/bench-pages -o ./bench/bench-out/htmd";
const turndownCmd = "bun bench/turndownCli.ts ./bench/bench-pages";
const pandocCmd = "bun bench/pandocBatchCli.ts ./bench/bench-pages";

console.log("Benchmarking...");

const output =
  await $`hyperfine --warmup 3 --runs 5 '${htmdCmd}' '${turndownCmd}' '${pandocCmd}'`.text();

console.log(`Output:\n${output}`);

await writeBenchmarkResultReadme(output);

async function getInputFilesInfo(): Promise<InputFilesInfo> {
  const filenames = (await fs.readdir(benchPagesDir)).filter((name) =>
    name.endsWith(".html")
  );
  let totalSize = 0;
  for (const filename of filenames) {
    totalSize += (await fs.stat(path.join(benchPagesDir, filename))).size;
  }
  return {
    fileCount: filenames.length,
    totalSize: totalSize,
  };
}

function osCpus(): string {
  const cpus = new Map<string, number>();
  for (const cpu of os.cpus()) {
    cpus.set(cpu.model, (cpus.get(cpu.model) ?? 0) + 1);
  }
  return [...cpus.entries()]
    .map(([cpu, count]) => `${cpu} x ${count}`)
    .join("\n");
}

function osMemoryGB(): string {
  return (os.totalmem() / (1024 * 1024 * 1024)).toFixed(1) + " GB";
}

async function htmdCliVersion(): Promise<string> {
  const tomlString = (await fs.readFile("Cargo.toml")).toString();
  const cargo = toml.parse(tomlString);
  return cargo.package.version;
}

async function turndownVersion(): Promise<string> {
  const packageJson = (await fs.readFile("bench/package.json")).toString();
  return JSON.parse(packageJson).devDependencies.turndown;
}

async function writeBenchmarkResultReadme(result: string) {
  const benchCommand = `hyperfine --warmup 3 --runs 5 \\\n'${htmdCmd}' \\\n'${turndownCmd}' \\\n'${pandocCmd}'`;
  const inputFilesInfo = await getInputFilesInfo();
  const md = `# Benchmark

An HTML-to-Markdown benchmark for htmd-cli, [Turndown.js](https://github.com/mixmark-io/turndown) and [Pandoc](https://github.com/jgm/pandoc).

### What does it do?

- Fetch ${BENCH_PAGE_COUNT} page links from [Wikipedia - Rust](<${SOURCE_PAGE_URL}>)
- Fetch all ${BENCH_PAGE_COUNT} pages and save them as HTML files
- Bench using [hyperfine](https://github.com/sharkdp/hyperfine) and following command:
  \`\`\`
${benchCommand
  .split("\n")
  .map((line) => "  " + line)
  .join("\n")}
  \`\`\`

# Environment

System: ${os.platform()} ${os.arch()} ${os.version()}

CPUs: ${osCpus()}

Memory: ${osMemoryGB()}

# Versions

Bun: ${(await $`bun -v`.text()).trim()}

Hyperfine: ${(await $`hyperfine --version`.text()).split(" ")[1].trim()}

htmd-cli: ${await htmdCliVersion()}

Turndown.js: ${await turndownVersion()}

Pandoc: ${(await $`pandoc -v`.text()).split(" ")[1].split("\n")[0]}

# Inputs

File count: ${inputFilesInfo.fileCount}

Total size: ${(inputFilesInfo.totalSize / 1024 / 1024).toFixed(2)} MB

# Results

\`\`\`
${result}
\`\`\`

*Updated at ${new Date().toUTCString()}*

*Generated by [bench.ts](bench.ts)*
`;
  await fs.writeFile("bench/README.md", md);
}
