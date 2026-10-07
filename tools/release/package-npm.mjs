#!/usr/bin/env node
// Usage: node package-npm.mjs <dir with the release archives> <out dir> <version, e.g. v0.4.0>
// Writes <out>/cli (@ktrs/cli, from npm/cli) and <out>/cli-<os>-<cpu> per archive found in <dir>, all stamped with
// <version>; `npm publish` each directory, the platform packages first.
import { execFileSync } from "node:child_process";
import { chmodSync, cpSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const [dist, out, tag] = process.argv.slice(2);
if (!tag) throw new Error("usage: package-npm.mjs <dist> <out> <version>");
const version = tag.replace(/^v/, "");
const root = resolve(import.meta.dirname, "../..");
const legal = ["LICENSE-MIT", "LICENSE-APACHE", "NOTICE"];
const targets = {
  "x86_64-pc-windows-msvc": ["win32", "x64"],
  "aarch64-pc-windows-msvc": ["win32", "arm64"],
  "x86_64-apple-darwin": ["darwin", "x64"],
  "aarch64-apple-darwin": ["darwin", "arm64"],
  "x86_64-unknown-linux-musl": ["linux", "x64"],
  "aarch64-unknown-linux-musl": ["linux", "arm64"],
};

const main = JSON.parse(readFileSync(join(root, "npm/cli/package.json"), "utf8"));
const writeJson = (file, value) => writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const copyLegal = (dir, extra = []) => [...legal, ...extra].forEach((f) => cpSync(join(root, f), join(dir, f)));
rmSync(out, { recursive: true, force: true });

for (const [target, [os, cpu]] of Object.entries(targets)) {
  const archive = readdirSync(dist).find((f) => f.startsWith("ktrs-") && /\.(tar\.gz|zip)$/.test(f) && f.includes(`-${target}.`));
  if (!archive) {
    console.error(`skipping ${os}-${cpu}: no ${target} archive in ${dist}`);
    continue;
  }
  const dir = join(out, `cli-${os}-${cpu}`);
  const bin = join(dir, "bin");
  mkdirSync(bin, { recursive: true });
  const exe = os === "win32" ? ".exe" : "";
  const names = ["ktrs", "ktlint", "ktfmt"].map((n) => n + exe);
  const patterns = names.map((n) => `*/${n}`);
  if (archive.endsWith(".zip")) {
    execFileSync("unzip", ["-j", "-q", "-o", join(dist, archive), ...patterns, "-d", bin]);
  } else {
    execFileSync("tar", ["-xzf", join(dist, archive), "-C", bin, "--strip-components=1", "--wildcards", ...patterns]);
  }
  names.forEach((n) => chmodSync(join(bin, n), 0o755));
  writeJson(join(dir, "package.json"), {
    name: `@ktrs/cli-${os}-${cpu}`,
    version,
    description: `The ${os}-${cpu} binaries of @ktrs/cli`,
    homepage: main.homepage,
    repository: main.repository,
    license: main.license,
    os: [os],
    cpu: [cpu],
    files: ["bin"],
    preferUnplugged: true,
  });
  copyLegal(dir);
}

const dir = join(out, "cli");
cpSync(join(root, "npm/cli"), dir, { recursive: true });
main.version = version;
for (const dep of Object.keys(main.optionalDependencies)) main.optionalDependencies[dep] = version;
writeJson(join(dir, "package.json"), main);
copyLegal(dir, ["README.md"]);
console.log(readdirSync(out).map((d) => join(out, d)).join("\n"));
