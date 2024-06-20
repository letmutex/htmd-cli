import { $ } from "bun";
import * as fs from "fs/promises";
import * as path from "path";

const htmlDir = process.argv[2];

if (htmlDir == null) {
  throw new Error("No html dir provided");
}

if (!(await fs.exists(htmlDir))) {
  throw new Error(`Html dir does not exist: ${htmlDir}`);
}

const start = Date.now();

async function convert(filepath: string, outputDir: string) {
  const outFile = path.join(
    outputDir,
    path.parse(path.basename(filepath)).name + ".md"
  );
  await $`pandoc ${filepath} --from html --to markdown --output ${outFile}`;
}

async function promisePool<T>(
  tasks: (() => Promise<T>)[],
  poolSize: number
): Promise<T[]> {
  const results: Promise<T>[] = [];
  const executing: Promise<void>[] = [];

  for (const task of tasks) {
    const p = Promise.resolve().then(() => task());
    results.push(p);

    if (poolSize <= tasks.length) {
      const e = p.then(() => {
        executing.splice(executing.indexOf(e), 1);
      });
      executing.push(e);
      if (executing.length >= poolSize) {
        await Promise.race(executing);
      }
    }
  }
  return Promise.all(results);
}

const outputDir = path.join(path.dirname(__filename), "bench-out", "pandoc");

if (!(await fs.exists(outputDir))) {
  await fs.mkdir(outputDir, { recursive: true });
}

const isHtmlDirAbsolutePath = path.isAbsolute(htmlDir);

const htmlFiles = (await fs.readdir(htmlDir))
  .map((name) => {
    if (isHtmlDirAbsolutePath) {
      return path.join(htmlDir, name);
    } else {
      return path.join(process.cwd(), htmlDir, name);
    }
  })
  .filter(async (filepath) => {
    return filepath.endsWith(".html");
  });

const promises: (() => Promise<void>)[] = [];
for (const file of htmlFiles) {
  promises.push(() => convert(file, outputDir));
}

await promisePool(promises, 50);

console.log(`Converted ${htmlFiles.length} files in ${Date.now() - start}ms.`);
