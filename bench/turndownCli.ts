import * as fs from "fs/promises";
import * as path from "path";
import TurndownService from "turndown";

const htmlDir = process.argv[2];

if (htmlDir == null) {
  throw new Error("No html dir provided");
}

if (!(await fs.exists(htmlDir))) {
  throw new Error(`Html dir does not exist: ${htmlDir}`);
}

const start = Date.now();

const turndownService = new TurndownService();

async function convert(filepath: string, outputDir: string) {
  const html = (await fs.readFile(filepath)).toString();
  const md = turndownService.turndown(html);
  await fs.writeFile(
    path.join(outputDir, path.parse(path.basename(filepath)).name + ".md"),
    md
  );
}

const outputDir = path.join(path.dirname(__filename), "bench-out", "turndown");

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

const promises: Promise<void>[] = [];
for (const file of htmlFiles) {
  promises.push(convert(file, outputDir));
}

await Promise.all(promises);

console.log(`Converted ${htmlFiles.length} files in ${Date.now() - start}ms.`);
