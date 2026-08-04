import { cp, mkdir, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const docsRoot = fileURLToPath(new URL("..", import.meta.url));
const repositoryRoot = path.resolve(docsRoot, "..");
const rustdocRoot = path.join(repositoryRoot, "target", "doc");
const publishedRoot = path.join(docsRoot, ".vitepress", "dist", "api", "rust");

const process = Bun.spawn(
  ["cargo", "doc", "-p", "vektra", "--all-features", "--no-deps"],
  {
    cwd: repositoryRoot,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit"
  }
);

const exitCode = await process.exited;
if (exitCode !== 0) {
  throw new Error(`cargo doc failed with exit code ${exitCode}`);
}

await rm(publishedRoot, { recursive: true, force: true });
await mkdir(path.dirname(publishedRoot), { recursive: true });
await cp(rustdocRoot, publishedRoot, { recursive: true });

console.log(`Published Vektra rustdoc to ${publishedRoot}`);
