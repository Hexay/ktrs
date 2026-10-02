//! ktlint's logging: kotlin-logging over logback with its default `TTLLLayout` console appender, which
//! writes `HH:mm:ss.SSS [thread] LEVEL logger -- message` to *stdout* (also with `--stdin`).

use ktrs_lint::editorconfig::KtlintVersion;

use crate::ktlint::console::{Console, LINE_SEPARATOR};
use crate::ktlint::version;

pub const KTLINT_COMMAND_LINE: &str = "io.github.ktlint.core.cli.internal.KtlintCommandLine";
pub const FILE_UTILS: &str = "io.github.ktlint.core.cli.internal.FileUtils";
pub const REPORTER_AGGREGATOR: &str = "io.github.ktlint.core.cli.internal.ReporterAggregator";
pub const KTLINT_SERVICE_LOADER: &str = "io.github.ktlint.core.cli.internal.KtlintServiceLoader";
pub const LOAD_RULE_PROVIDERS: &str = "io.github.ktlint.core.cli.internal.LoadRuleProviders";
pub const BASELINE: &str = "io.github.ktlint.core.cli.reporter.baseline.Baseline";
pub const EDITOR_CONFIG_DEFAULTS_LOADER: &str = "io.github.ktlint.core.rule.engine.internal.EditorConfigDefaultsLoader";
pub const GENERATE_EDITOR_CONFIG_SUB_COMMAND: &str = "io.github.ktlint.core.cli.internal.GenerateEditorConfigSubCommand";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    /// `--log-level=none`.
    Off,
}

impl Level {
    /// `--log-level`'s converter; `Err` is its `error("Invalid log level '$it'")`.
    pub fn parse(value: &str) -> Result<Level, String> {
        match value.to_uppercase().as_str() {
            "TRACE" => Ok(Level::Trace),
            "DEBUG" => Ok(Level::Debug),
            "INFO" => Ok(Level::Info),
            "WARN" => Ok(Level::Warn),
            "ERROR" => Ok(Level::Error),
            "NONE" => Ok(Level::Off),
            _ => Err(format!("Invalid log level '{value}'")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Off => "OFF",
        }
    }
}

#[derive(Clone)]
pub struct Logger {
    console: Console,
    min_level: Level,
    ktlint_version: KtlintVersion,
}

impl Logger {
    pub fn new(console: Console, min_level: Level, ktlint_version: KtlintVersion) -> Logger {
        Logger { console, min_level, ktlint_version }
    }

    pub fn is_enabled(&self, level: Level) -> bool {
        level != Level::Off && level >= self.min_level
    }

    /// Logs `message()` from `logger` (a 2.0 class name, renamed to the run's package) on the main
    /// thread, if `level` is enabled.
    pub fn log(&self, level: Level, logger: &str, message: impl FnOnce() -> String) {
        if self.is_enabled(level) {
            let logger = match logger.strip_prefix(version::package(KtlintVersion::V2_0)) {
                Some(class) => format!("{}{class}", version::package(self.ktlint_version)),
                None => logger.to_owned(),
            };
            self.console
                .out(&format!("{} [main] {} {logger} -- {}{LINE_SEPARATOR}", local_time(), level.name(), message()));
        }
    }

    pub fn trace(&self, logger: &str, message: impl FnOnce() -> String) {
        self.log(Level::Trace, logger, message);
    }

    pub fn debug(&self, logger: &str, message: impl FnOnce() -> String) {
        self.log(Level::Debug, logger, message);
    }

    pub fn info(&self, logger: &str, message: impl FnOnce() -> String) {
        self.log(Level::Info, logger, message);
    }

    pub fn warn(&self, logger: &str, message: impl FnOnce() -> String) {
        self.log(Level::Warn, logger, message);
    }

    pub fn error(&self, logger: &str, message: impl FnOnce() -> String) {
        self.log(Level::Error, logger, message);
    }
}

/// `%d{HH:mm:ss.SSS}` in the local time zone.
fn local_time() -> String {
    let (hour, minute, second, millis) = local_clock();
    format!("{hour:02}:{minute:02}:{second:02}.{millis:03}")
}

#[cfg(windows)]
fn local_clock() -> (u16, u16, u16, u16) {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLocalTime(system_time: *mut [u16; 8]);
    }
    let mut st = [0u16; 8];
    // SAFETY: SYSTEMTIME is eight WORDs; GetLocalTime only writes it.
    unsafe { GetLocalTime(&mut st) };
    (st[4], st[5], st[6], st[7])
}

#[cfg(all(unix, target_pointer_width = "64"))]
fn local_clock() -> (u16, u16, u16, u16) {
    unsafe extern "C" {
        fn localtime_r(time: *const i64, tm: *mut [i32; 16]) -> *mut [i32; 16];
    }
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs() as i64;
    // `struct tm` starts with tm_sec, tm_min, tm_hour (ints); 64 bytes covers glibc/musl/macOS layouts.
    let mut tm = [0i32; 16];
    // SAFETY: time_t is i64 on 64-bit unix; localtime_r writes at most sizeof(struct tm) <= 64 bytes.
    let ok = unsafe { !localtime_r(&secs, &mut tm).is_null() };
    if ok {
        (tm[2] as u16, tm[1] as u16, tm[0] as u16, now.subsec_millis() as u16)
    } else {
        utc_clock(now)
    }
}

#[cfg(not(any(windows, all(unix, target_pointer_width = "64"))))]
fn local_clock() -> (u16, u16, u16, u16) {
    utc_clock(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default())
}

#[cfg_attr(windows, allow(dead_code))]
fn utc_clock(now: std::time::Duration) -> (u16, u16, u16, u16) {
    let day_secs = now.as_secs() % 86_400;
    ((day_secs / 3600) as u16, (day_secs / 60 % 60) as u16, (day_secs % 60) as u16, now.subsec_millis() as u16)
}
