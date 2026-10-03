//! The Windows `java` launcher's wildcard expansion of application arguments, which ktfmt and ktlint get
//! before their `main` runs: an argument with an unquoted `*` or `?` becomes the matching entries of its parent
//! directory. Ports JDK 21 `cmdtoargs.c` (`next_arg`, `JLI_CmdToArgs`), `java_md.c` (`CreateApplicationArgs`) and
//! `LauncherHelper.expandArgs`. Other platforms pass arguments through (the shell expands there).

use crate::ktlint::java_glob::PathMatcher;

/// The program's arguments without its name, as the drop-ins' JVM `main` would receive them.
pub fn application_args() -> Vec<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    {
        expand_application_args(args, &windows::command_line())
    }
    #[cfg(not(windows))]
    {
        args
    }
}

/// `CreateApplicationArgs`: expands an argument only where the launcher's own split of `command_line` has the same
/// text with an unquoted wildcard; when the two splits disagree on the count, everything is passed as is.
pub fn expand_application_args(args: Vec<String>, command_line: &str) -> Vec<String> {
    let std_args = cmd_to_args(command_line);
    if std_args.len() != args.len() + 1 {
        return args;
    }
    args.into_iter()
        .zip(std_args.into_iter().skip(1))
        .flat_map(|(arg, (std_arg, has_wildcard))| if has_wildcard && std_arg == arg { expand_arg(&arg) } else { vec![arg] })
        .collect()
}

/// `JLI_CmdToArgs` without `JDK_JAVA_OPTIONS` and launcher `@argfiles` (neither reaches application arguments).
fn cmd_to_args(command_line: &str) -> Vec<(String, bool)> {
    let chars: Vec<char> = command_line.chars().collect();
    let mut pos = 0;
    let mut out = Vec::new();
    loop {
        let (arg, wildcard, more) = next_arg(&chars, &mut pos);
        out.push((arg, wildcard));
        if !more {
            return out;
        }
    }
}

/// `next_arg`: the argument at `pos`, whether it has a `*`/`?` outside quotes, and whether another one follows.
fn next_arg(chars: &[char], pos: &mut usize) -> (String, bool, bool) {
    let mut dest = String::new();
    let (mut separator, mut quotes, mut slashes, mut prev) = (false, 0i32, 0usize, '\0');
    let mut wildcard = false;
    let mut done = false;
    let push_slashes = |dest: &mut String, n: usize| dest.extend(std::iter::repeat_n('\\', n));
    while !done {
        let Some(&ch) = chars.get(*pos) else { break };
        match ch {
            '"' => {
                if separator {
                    done = true;
                } else {
                    if prev == '\\' {
                        push_slashes(&mut dest, slashes / 2);
                        if slashes % 2 == 1 {
                            dest.push(ch);
                        } else {
                            quotes += 1;
                        }
                    } else if prev == '"' && quotes % 2 == 0 {
                        quotes += 1;
                        dest.push(ch);
                    } else if quotes == 0 {
                        quotes += 1;
                    } else {
                        quotes -= 1;
                    }
                    slashes = 0;
                }
            }
            '\\' => {
                slashes += 1;
                if separator {
                    done = true;
                    separator = false;
                }
            }
            ' ' | '\t' => {
                if prev == '\\' {
                    push_slashes(&mut dest, slashes);
                }
                if quotes % 2 == 1 {
                    dest.push(ch);
                } else {
                    separator = true;
                }
                slashes = 0;
            }
            '*' | '?' => {
                if separator {
                    done = true;
                    separator = false;
                } else {
                    if quotes % 2 == 0 {
                        wildcard = true;
                    }
                    if prev == '\\' {
                        push_slashes(&mut dest, slashes);
                    }
                    dest.push(ch);
                    slashes = 0;
                }
            }
            _ => {
                if prev == '\\' {
                    push_slashes(&mut dest, slashes);
                    dest.push(ch);
                } else if separator {
                    done = true;
                } else {
                    dest.push(ch);
                }
                slashes = 0;
            }
        }
        if !done {
            prev = ch;
            *pos += 1;
        }
    }
    if prev == '\\' {
        push_slashes(&mut dest, slashes);
    }
    (dest, wildcard, done)
}

/// `LauncherHelper.expandArgs` for one argument: `Files.newDirectoryStream(parent, name)`, each entry's
/// `normalize().toString()`; the argument itself when nothing matches or the stream throws.
fn expand_arg(arg: &str) -> Vec<String> {
    let file = windows_path::File::new(arg);
    let parent = file.parent().unwrap_or_else(|| ".".to_owned());
    let Some(dir) = windows_path::Path::parse(&parent) else { return vec![arg.to_owned()] };
    let Ok(matcher) = PathMatcher::new(&file.name(), true) else { return vec![arg.to_owned()] };
    let Ok(entries) = std::fs::read_dir(dir.to_string()) else { return vec![arg.to_owned()] };
    let expanded: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| matcher.matches(name))
        .map(|name| dir.resolve(&name).normalize().to_string())
        .collect();
    if expanded.is_empty() { vec![arg.to_owned()] } else { expanded }
}

mod windows_path;

#[cfg(windows)]
mod windows {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCommandLineW() -> *const u16;
    }

    pub fn command_line() -> String {
        // SAFETY: GetCommandLineW returns a NUL-terminated string owned by the process for its lifetime.
        unsafe {
            let p = GetCommandLineW();
            let len = (0..).take_while(|&i| *p.add(i) != 0).count();
            String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
        }
    }
}

#[cfg(test)]
mod tests;
