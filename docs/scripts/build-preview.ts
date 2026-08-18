import { spawnSync } from "node:child_process";
import { cp, mkdir, rm } from "node:fs/promises";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const docsDir = resolve(scriptDir, "..");
const previewDir = resolve(docsDir, "preview");
const previewDistDir = resolve(previewDir, "dist");
const publicDir = resolve(docsDir, "public");
const publicPreviewsDir = resolve(publicDir, "previews");

function assertInside(child: string, parent: string): void {
  const relativePath = relative(parent, child);
  const outsideParent =
    relativePath === ".." ||
    relativePath.startsWith(`..${sep}`) ||
    isAbsolute(relativePath);
  if (outsideParent) {
    throw new Error(`路径 ${child} 不在允许目录 ${parent} 内`);
  }
}

const trunk = spawnSync(
  "trunk",
  ["build", "--release", "--public-url", "./"],
  {
    cwd: previewDir,
    env: {
      ...process.env,
      NO_COLOR: process.env.NO_COLOR === "1" ? "true" : process.env.NO_COLOR
    },
    stdio: "inherit"
  }
);

if (trunk.status !== 0) {
  process.exit(trunk.status ?? 1);
}

assertInside(publicPreviewsDir, docsDir);
await rm(publicPreviewsDir, { recursive: true, force: true });
await mkdir(publicDir, { recursive: true });
await cp(previewDistDir, publicPreviewsDir, { recursive: true });

console.log(`已构建并同步 GPUI WASM 预览：${publicPreviewsDir}`);
