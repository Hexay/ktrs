import * as fs from "node:fs";
import * as path from "node:path";

const exe = process.platform === "win32" ? "ktrs.exe" : "ktrs";

/** `ktrs.path` if set, else the binary bundled in a platform-specific VSIX, else `ktrs` on `PATH`; undefined if none. */
export function resolveBinary(configured: string, extensionPath: string): string | undefined {
  if (configured) {
    return configured;
  }
  const bundled = path.join(extensionPath, "bundled", exe);
  if (fs.existsSync(bundled)) {
    ensureExecutable(bundled);
    return bundled;
  }
  return findOnPath();
}

// Guards against a VSIX extractor that drops the zip entry's mode bits.
function ensureExecutable(file: string): void {
  if (process.platform === "win32") {
    return;
  }
  try {
    fs.accessSync(file, fs.constants.X_OK);
  } catch {
    fs.chmodSync(file, 0o755);
  }
}

function findOnPath(): string | undefined {
  const dirs = (process.env.PATH ?? "").split(path.delimiter).filter(Boolean);
  for (const dir of dirs) {
    const candidate = path.join(dir, exe);
    if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
      return candidate;
    }
  }
  return undefined;
}
