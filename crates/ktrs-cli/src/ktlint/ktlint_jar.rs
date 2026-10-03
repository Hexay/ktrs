//! Runs the real ktlint release jar with the run's argv when it loads a rule set JAR ktrs can't run natively
//! (research/27-custom-rulesets-impl.md): stdout, stderr and exit code pass through untouched. The jar is the
//! release asset of the active [`KtlintVersion`], downloaded once into the user cache and checked against a
//! pinned SHA-256; `KTRS_KTLINT_JAR` overrides the path (not checked).

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ktrs_lint::editorconfig::KtlintVersion;

use crate::ktlint::console::Console;
use crate::ktlint::hand_off_args::hand_off_args;
use crate::ktlint::sha256::sha256;

pub const KTLINT_JAR_ENV: &str = "KTRS_KTLINT_JAR";

/// A ktlint release's self-executing jar (the `ktlint` asset: a `sh` launcher prepended to the jar).
struct Release {
    version: &'static str,
    url: &'static str,
    sha256: &'static str,
}

fn release_of(ktlint_version: KtlintVersion) -> Release {
    match ktlint_version {
        KtlintVersion::V1_8 => Release {
            version: "1.8.0",
            url: "https://github.com/pinterest/ktlint/releases/download/1.8.0/ktlint",
            sha256: "a3fd620207d5c40da6ca789b95e7f823c54e854b7fade7f613e91096a3706d75",
        },
        KtlintVersion::V2_0 => Release {
            version: "2.0.0-ALPHA-4",
            url: "https://github.com/ktlint/ktlint/releases/download/2.0.0-ALPHA-4/ktlint",
            sha256: "fb28b3cd57116d1de78867ebd8ce398ede91e91b280b33dc367e0108336b79f1",
        },
    }
}

/// Where the hand-off finds `java`, the jar and its cache: the process environment, or a test's stand-in.
#[derive(Clone, Debug, Default)]
pub struct JvmEnv {
    pub java_home: Option<PathBuf>,
    pub path: Option<OsString>,
    pub jar: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
}

impl JvmEnv {
    pub fn from_env() -> JvmEnv {
        let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
        JvmEnv {
            java_home: var("JAVA_HOME").map(PathBuf::from),
            path: var("PATH"),
            jar: var(KTLINT_JAR_ENV).map(PathBuf::from),
            cache_dir: default_cache_dir(var),
        }
    }
}

/// The per-user cache: `%LOCALAPPDATA%\ktrs`, `~/Library/Caches/ktrs`, else `$XDG_CACHE_HOME/ktrs` or `~/.cache/ktrs`.
fn default_cache_dir(var: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let base = if cfg!(windows) {
        var("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|home| Path::new(&home).join("Library/Caches"))
    } else {
        var("XDG_CACHE_HOME").map(PathBuf::from).or_else(|| var("HOME").map(|home| Path::new(&home).join(".cache")))
    };
    base.map(|base| base.join("ktrs"))
}

/// Runs ktlint `ktlint_version`'s jar with `args` (ktrs's `--ktlint-version` removed) in `working_dir`;
/// returns its exit code. `cause` names the JAR that made the hand-off necessary, for error messages.
pub fn run_ktlint_jar(env: &JvmEnv, ktlint_version: KtlintVersion, args: &[String], working_dir: &Path, console: &Console, cause: &str) -> i32 {
    match launch(env, ktlint_version, args, working_dir, console) {
        Ok(code) => code,
        Err(message) => {
            console.err(&format!(
                "ktrs: '{cause}' is a ktlint plugin JAR ktrs can not run natively, so this run is handed to ktlint {} on the JVM, \
                 but {message}\n",
                release_of(ktlint_version).version
            ));
            1
        }
    }
}

fn launch(env: &JvmEnv, ktlint_version: KtlintVersion, args: &[String], working_dir: &Path, console: &Console) -> Result<i32, String> {
    let java = find_java(env).ok_or("no `java` was found: install a JDK (17+) and set JAVA_HOME or put `java` on PATH.")?;
    let jar = ktlint_jar(env, ktlint_version, console)?;
    let hand_off = hand_off_args(args, working_dir)?;
    let mut command = Command::new(&java);
    command.args(jvm_options(java_major_version(&java))).arg("-jar").arg(&jar).args(&hand_off.args).current_dir(working_dir);
    if console.is_process_streams() {
        return if hand_off.has_temp_files() { status(command) } else { exec(command) };
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("`{}` could not be started: {e}", java.display()))?;
    let input = console.read_stdin();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let _ = writer.join();
    console.out(&String::from_utf8_lossy(&output.stdout));
    console.err(&String::from_utf8_lossy(&output.stderr));
    Ok(output.status.code().unwrap_or(1))
}

#[cfg(unix)]
fn exec(mut command: Command) -> Result<i32, String> {
    use std::os::unix::process::CommandExt;
    let program = command.get_program().to_string_lossy().into_owned();
    let error = command.exec();
    Err(format!("`{program}` could not be started: {error}"))
}

#[cfg(not(unix))]
fn exec(command: Command) -> Result<i32, String> {
    status(command)
}

fn status(mut command: Command) -> Result<i32, String> {
    let status = command.status().map_err(|e| format!("`{}` could not be started: {e}", command.get_program().to_string_lossy()))?;
    Ok(status.code().unwrap_or(1))
}

/// The launcher script's options: `-Xmx512m`, plus two warning switches on Java 24+.
fn jvm_options(java_major_version: Option<u32>) -> Vec<&'static str> {
    let mut options = Vec::new();
    if java_major_version.is_some_and(|v| v >= 24) {
        options.extend(["--sun-misc-unsafe-memory-access=allow", "--enable-native-access=ALL-UNNAMED"]);
    }
    options.push("-Xmx512m");
    options
}

/// `java -version`'s major version, the launcher's way (`1.8.0_x` is 1).
fn java_major_version(java: &Path) -> Option<u32> {
    let output = Command::new(java).arg("-version").stdin(Stdio::null()).output().ok()?;
    let text = String::from_utf8_lossy(&output.stderr);
    let version = text.split("version \"").nth(1)?;
    version.split(['.', '-', '"']).next()?.parse().ok()
}

fn find_java(env: &JvmEnv) -> Option<PathBuf> {
    let exe = if cfg!(windows) { "java.exe" } else { "java" };
    let from_home = env.java_home.as_ref().map(|home| home.join("bin").join(exe)).filter(|java| java.is_file());
    from_home.or_else(|| std::env::split_paths(env.path.as_ref()?).map(|dir| dir.join(exe)).find(|java| java.is_file()))
}

/// `KTRS_KTLINT_JAR`, else the cached release jar, downloaded on first use.
fn ktlint_jar(env: &JvmEnv, ktlint_version: KtlintVersion, console: &Console) -> Result<PathBuf, String> {
    if let Some(jar) = &env.jar {
        return if jar.is_file() { Ok(jar.clone()) } else { Err(format!("{KTLINT_JAR_ENV} names '{}', which does not exist.", jar.display())) };
    }
    let release = release_of(ktlint_version);
    let cache_dir = env.cache_dir.as_ref().ok_or(format!("there is no user cache directory to download it to: set {KTLINT_JAR_ENV}."))?;
    let jar = cache_dir.join(format!("ktlint-{}.jar", release.version));
    if !jar.is_file() {
        download(&release, &jar, console)?;
    }
    Ok(jar)
}

/// Downloads with `curl` into a per-process temporary file, checks the digest, then renames it into place.
fn download(release: &Release, jar: &Path, console: &Console) -> Result<(), String> {
    let manual = format!("download {} yourself and set {KTLINT_JAR_ENV} to its path.", release.url);
    std::fs::create_dir_all(jar.parent().unwrap()).map_err(|e| format!("'{}' can not be created ({e}): {manual}", jar.display()))?;
    console.err(&format!("ktrs: downloading ktlint {} (once) to {}\n", release.version, jar.display()));
    let part = jar.with_extension(format!("{}.part", std::process::id()));
    let status = Command::new("curl").args(["-fsSL", "--retry", "2", "-o"]).arg(&part).arg(release.url).stdin(Stdio::null()).status();
    let result = match status {
        Ok(status) if status.success() => verify(&part, release.sha256).map_err(|e| format!("{e}: {manual}")),
        Ok(status) => Err(format!("curl failed ({status}): {manual}")),
        Err(e) => Err(format!("`curl` could not be run ({e}): {manual}")),
    };
    let result = result.and_then(|()| std::fs::rename(&part, jar).map_err(|e| format!("'{}' can not be written ({e}): {manual}", jar.display())));
    let _ = std::fs::remove_file(&part);
    result
}

fn verify(file: &Path, expected: &str) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
    let actual: String = sha256(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    if actual == expected { Ok(()) } else { Err(format!("the download's SHA-256 is {actual}, not the pinned {expected}")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_options_follow_the_java_version() {
        assert_eq!(jvm_options(Some(21)), ["-Xmx512m"]);
        assert_eq!(jvm_options(None), ["-Xmx512m"]);
        assert_eq!(jvm_options(Some(25)), ["--sun-misc-unsafe-memory-access=allow", "--enable-native-access=ALL-UNNAMED", "-Xmx512m"]);
    }

    #[test]
    fn missing_java_is_an_error_naming_the_jar() {
        let env = JvmEnv { path: Some(OsString::new()), ..JvmEnv::default() };
        let (console, out, err) = Console::capture(b"");
        let code = run_ktlint_jar(&env, KtlintVersion::V1_8, &[], Path::new("."), &console, "custom.jar");
        assert_eq!(code, 1);
        assert_eq!(out.text(), "");
        assert!(err.text().starts_with("ktrs: 'custom.jar' is a ktlint plugin JAR"), "{}", err.text());
        assert!(err.text().contains("ktlint 1.8.0 on the JVM, but no `java` was found"), "{}", err.text());
    }
}
