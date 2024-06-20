import * as fs from "fs/promises";
import * as path from "path";
import puppeteer from "puppeteer";

type Page = {
  title: string;
  html: string;
};

async function fetchPage(url: string): Promise<Page> {
  const browser = await puppeteer.launch({
    args: ["--lang=en-US"],
  });
  const page = await browser.newPage();
  await page.goto(url, {
    waitUntil: "domcontentloaded",
    timeout: 20000,
  });
  const title = await page.title();
  const html = await page.content();
  await browser.close();
  return {
    title,
    html,
  };
}

function sanitizePath(input: string): string {
  const illegalRe = /[<>:"/\\|?*]/g;
  const controlRe = /[\x00-\x1f\x80-\x9f]/g;
  const reservedRe = /^\.+$/;
  const windowsReservedRe = /^(con|prn|aux|nul|com\d|lpt\d)$/i;
  const windowsTrailingRe = /[. ]+$/;

  let sanitized = input
    .replace(illegalRe, "")
    .replace(controlRe, "")
    .replace(reservedRe, "")
    .replace(windowsReservedRe, "")
    .replace(windowsTrailingRe, "")
    .trim();

  return sanitized;
}

function findPageLinksInHtml(
  pageUrl: string,
  html: string,
  count: number
): string[] {
  const url = new URL(pageUrl);
  // Regex to match <a> tags and capture href values
  const linkRegex = /<a\s+(?:[^>]*?\s+)?href="([^"]*)"/g;
  // Extract all links
  let links = new Set<string>();
  let match;
  while ((match = linkRegex.exec(html)) !== null && links.size < count) {
    const link = match[1];
    if (link.startsWith("#")) {
      continue;
    }
    if (!link.startsWith("http://") && !link.startsWith("https://")) {
      links.add(url.origin + link);
    } else {
      links.add(link);
    }
  }
  return Array.from(links);
}

export async function fetchBenchPages(
  sourcePageUrl: string,
  targetDir: string,
  count: number
) {
  const sourcePage = await fetchPage(sourcePageUrl);

  const pageLinks = findPageLinksInHtml(sourcePageUrl, sourcePage.html, count);

  console.log(`Got ${pageLinks.length} links from ${sourcePageUrl}`);

  if (await fs.exists(targetDir)) {
    await fs.rm(targetDir, { recursive: true });
  }
  await fs.mkdir(targetDir);

  const batchSize = 10;
  let linkIndex = 0;

  const outputDir = path.isAbsolute(targetDir)
    ? targetDir
    : path.join(process.cwd(), targetDir);

  const savedFilenames = new Map<string, number>();

  while (linkIndex < pageLinks.length) {
    const promises: Promise<void>[] = [];
    const end = Math.min(pageLinks.length, linkIndex + batchSize);

    console.log(`Fetch pages from index ${linkIndex} to ${end}`);

    for (let i = linkIndex; i < end && i < pageLinks.length; i++) {
      const promise = fetchPage(pageLinks[i]).then(async (page) => {
        const filename = sanitizePath(page.title);
        const savedCount = savedFilenames.get(filename) ?? 0;
        const filenameToSave =
          savedCount > 0 ? filename + ` (${savedCount})` : filename;
        const file = path.join(outputDir, filenameToSave + ".html");
        savedFilenames.set(filename, savedCount + 1);
        await fs.writeFile(file, page.html);
      });
      promises.push(promise);
    }
    linkIndex = end;
    await Promise.all(promises).then(async (page) => {});
  }
}
