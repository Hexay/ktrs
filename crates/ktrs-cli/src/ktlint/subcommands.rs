//! Ports of `GenerateEditorConfigSubCommand`, `GitHookCliktCommand` and the two git hook subcommands.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ktrs_editorconfig::EnumValue;
use ktrs_lint::editorconfig::{CODE_STYLE_PROPERTY, CodeStyleValue, KtlintVersion, PropertyRef};
use ktrs_lint::rule_provider::{RuleV2Provider, rule_providers_in};
use ktrs_lint::{EditorConfigDefaults, EditorConfigOverride, KtLintRuleEngine};

use crate::ktlint::command_line::{Exit, ExitCode, KtlintCli};
use crate::ktlint::legacy_rule_set::require_singular_identities;
use crate::ktlint::logger::{GENERATE_EDITOR_CONFIG_SUB_COMMAND, Logger};
use crate::ktlint::sha256::sha256;
use crate::ktlint::version::{repository, with_ktlint_version};

pub fn generate_editor_config(
    cli: &KtlintCli,
    rule_providers: Vec<RuleV2Provider>,
    code_style: CodeStyleValue,
    logger: &Logger,
    ktlint_version: KtlintVersion,
) -> Result<(), Exit> {
    if !ktlint_version.is_1_8() {
        require_singular_identities(&rule_providers)?;
    }
    let code_style = EditorConfigOverride::from(vec![(PropertyRef::from(&*CODE_STYLE_PROPERTY), Some(code_style.name().to_owned()))]);
    let engine = KtLintRuleEngine::with_editor_config(
        rule_providers_in(&rule_providers, ktlint_version),
        EditorConfigDefaults::empty(),
        with_ktlint_version(code_style, ktlint_version),
    );
    let generated_editor_config = engine
        .generate_kotlin_editor_config_section(&cli.working_dir.to_path_buf())
        .map_err(|e| Exit::Crash(format!("org.ec4j.core.parser.ParseException: {e}")))?;
    if generated_editor_config.trim().is_empty() {
        logger.info(GENERATE_EDITOR_CONFIG_SUB_COMMAND, || "Nothing to add to .editorconfig file".to_owned());
    } else {
        // Printed, not logged: it is to be copied into '.editorconfig'.
        cli.console.println_out(&format!("[*.{{kt,kts}}]\n{generated_editor_config}"));
    }
    Ok(())
}

pub struct GitHook {
    name: &'static str,
    content: String,
}

pub fn pre_commit(ktlint_version: KtlintVersion) -> GitHook {
    GitHook {
        name: "pre-commit",
        content: format!(
            "#!/bin/sh\n\n# <{}> pre-commit hook\n\n\
             git diff --name-only -z --cached --relative -- '*.kt' '*.kts' | ktlint --relative --patterns-from-stdin=''",
            repository(ktlint_version)
        ),
    }
}

pub fn pre_push(ktlint_version: KtlintVersion) -> GitHook {
    GitHook {
        name: "pre-push",
        content: format!(
            "#!/bin/sh\n\n# <{}> pre-push hook\n\n\
             git diff --name-only -z HEAD \"origin/$(git rev-parse --abbrev-ref HEAD)\" -- '*.kt' '*.kts' | ktlint --relative --patterns-from-stdin=''",
            repository(ktlint_version)
        ),
    }
}

/// `installGitHook`: writes the hook into the repository's hooks directory, backing up a different one.
pub fn install_git_hook(cli: &KtlintCli, hook: GitHook) -> Result<(), Exit> {
    let working_dir = cli.working_dir.to_path_buf();
    let git_hooks_dir = match resolve_git_hooks_dir(&working_dir) {
        Ok(dir) => dir,
        Err(message) => {
            cli.console.println_err(&message);
            return Err(Exit::Code(ExitCode::IoException));
        }
    };
    let git_hook_file = git_hooks_dir.join(hook.name);
    let hook_content = hook.content.as_bytes();
    if working_dir.join(&git_hook_file).exists() {
        backup_existing_hook(cli, &git_hooks_dir, &git_hook_file, hook_content, hook.name)?;
    }
    let absolute = working_dir.join(&git_hook_file);
    fs::write(&absolute, hook_content).map_err(|e| Exit::Crash(format!("java.io.IOException: {e}")))?;
    set_executable(&absolute);
    cli.console.println_out(&format!(
        "{} is installed. Be aware that this hook assumes to find ktlint on the PATH. Either ensure that ktlint is actually \
         added to the path or expand the ktlint command in the hook with the path.",
        java_path(&git_hook_file)
    ));
    Ok(())
}

fn resolve_git_hooks_dir(working_dir: &Path) -> Result<PathBuf, String> {
    let git_dir = get_git_dir(working_dir)?;
    let hooks_dir = git_dir.join(get_hooks_dir_name(working_dir));
    let absolute = working_dir.join(&hooks_dir);
    if !absolute.exists() && fs::create_dir(&absolute).is_err() {
        return Err(format!("Failed to create {} folder", java_path(&hooks_dir)));
    }
    Ok(hooks_dir)
}

/// The `.git` directory of `git rev-parse --show-toplevel` (absolute), else `.git/.git` (relative, as
/// upstream's `File(rootDir ?: ".git").resolve(".git")`).
fn get_git_dir(working_dir: &Path) -> Result<PathBuf, String> {
    let root_dir = first_line_of(working_dir, &["rev-parse", "--show-toplevel"]);
    let git_dir = PathBuf::from(root_dir.unwrap_or_else(|| ".git".to_owned())).join(".git");
    if !working_dir.join(&git_dir).is_dir() {
        return Err(".git directory not found. Are you sure you are inside project directory?".to_owned());
    }
    Ok(git_dir)
}

fn get_hooks_dir_name(working_dir: &Path) -> String {
    first_line_of(working_dir, &["config", "--get", "core.hooksPath"])
        .map(|dir| dir.trim().to_owned())
        .filter(|dir| !dir.is_empty())
        .unwrap_or_else(|| "hooks".to_owned())
}

fn first_line_of(working_dir: &Path, git_args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(git_args).current_dir(working_dir).output().ok()?;
    String::from_utf8_lossy(&output.stdout).lines().next().map(str::to_owned)
}

fn backup_existing_hook(cli: &KtlintCli, hooks_dir: &Path, hook_file: &Path, expected: &[u8], name: &str) -> Result<(), Exit> {
    let working_dir = cli.working_dir.to_path_buf();
    let actual = fs::read(working_dir.join(hook_file)).map_err(|e| Exit::Crash(format!("java.io.IOException: {e}")))?;
    if !actual.is_empty() && actual != expected {
        let backup_file = hooks_dir.join(format!("{name}.ktlint-backup.{}", to_unique_id(&actual)));
        cli.console
            .println_out(&format!("Existing git hook {} is copied to {}", java_path(hook_file), java_path(&backup_file)));
        fs::copy(working_dir.join(hook_file), working_dir.join(&backup_file))
            .map_err(|e| Exit::Crash(format!("java.io.IOException: {e}")))?;
    }
    Ok(())
}

/// `BigInteger(sha256(bytes)).toString(16)`: signed, without leading zeros.
fn to_unique_id(bytes: &[u8]) -> String {
    let mut digest = sha256(bytes);
    let negative = digest[0] & 0x80 != 0;
    if negative {
        let mut carry = true;
        for byte in digest.iter_mut().rev() {
            let (value, overflow) = (!*byte).overflowing_add(carry as u8);
            *byte = value;
            carry = overflow;
        }
    }
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    let hex = hex.trim_start_matches('0');
    format!("{}{}", if negative { "-" } else { "" }, if hex.is_empty() { "0" } else { hex })
}

/// `File.path`: platform separators.
fn java_path(path: &Path) -> String {
    let shown = path.to_string_lossy();
    if cfg!(windows) { shown.replace('/', "\\") } else { shown.into_owned() }
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn set_executable(_: &Path) {}
