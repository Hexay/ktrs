"use strict";
const { spawnSync } = require("child_process");

// Runs `name` from the installed @ktrs/cli-<platform>-<arch> package with this process's arguments and stdio,
// then exits as it did (same code, or the same signal).
module.exports = function run(name) {
  const pkg = `@ktrs/cli-${process.platform}-${process.arch}`;
  const exe = process.platform === "win32" ? `${name}.exe` : name;
  let binary;
  try {
    binary = require.resolve(`${pkg}/bin/${exe}`);
  } catch {
    console.error(
      `ktrs: ${pkg} is not installed. ktrs ships binaries for win32, darwin and linux on x64 and arm64; ` +
        "reinstall without --no-optional / --omit=optional, or download a binary from " +
        "https://github.com/Hexay/ktrs/releases",
    );
    process.exit(1);
  }
  const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit", windowsHide: true });
  if (result.error) {
    console.error(`ktrs: could not run ${binary}: ${result.error.message}`);
    process.exit(1);
  }
  if (result.signal) process.kill(process.pid, result.signal);
  process.exit(result.status ?? 1);
};
