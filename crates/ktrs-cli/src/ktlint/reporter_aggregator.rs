//! Port of ktlint-cli `internal/ReporterAggregator.kt`: `--reporter=id[?query][,artifact=jar][,output=file]`
//! configurations to the reporters, fanned out by one aggregated reporter.

use std::fs::{self, File};
use std::io::BufWriter;

use crate::ktlint::baseline::{Baseline, BaselineStatus};
use crate::ktlint::command_line::{Exit, ExitCode};
use crate::ktlint::console::{Console, Printer, Sink};
use crate::ktlint::file_utils::location;
use crate::github_annotations::{Annotations, REPORTER_ID};
use crate::ktlint::gradle::GradleEventsReporter;
use crate::ktlint::reporter::github::GithubReporter;
use crate::ktlint::jar_providers::{load_from_jar_file, to_files_uri_list};
use crate::ktlint::version::package;
use crate::ktlint::jpath::JPath;
use crate::ktlint::logger::{Logger, REPORTER_AGGREGATOR};
use crate::ktlint::reporter::{
    KtlintCliError, REPORTER_PROVIDER_IDS, ReporterEnvironment, ReporterOptions, ReporterV2, get_reporter,
};

pub struct ReporterSettings<'a> {
    pub reporter_configurations: &'a [String],
    pub color: bool,
    pub color_name: &'a str,
    pub stdin: bool,
    pub format: bool,
    pub relative: bool,
    /// `--ktrs-gradle-events` (`crate::ktlint::gradle`).
    pub gradle_events: Option<&'a str>,
    /// `ktrs lint`: `github` is a reporter id too (`reporter/github.rs`).
    pub github: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReporterConfiguration {
    id: String,
    artifact: Option<String>,
    additional_config: ReporterOptions,
    output: Option<String>,
}

pub struct Context<'a> {
    pub console: &'a Console,
    pub logger: &'a Logger,
    pub working_dir: &'a JPath,
    pub user_home: &'a str,
    pub env: &'a ReporterEnvironment,
}

pub fn aggregated_reporter(baseline: &Baseline, settings: &ReporterSettings, cx: &Context) -> Result<AggregatedReporter, Exit> {
    let defaults = ["plain".to_owned()];
    let configurations = if settings.reporter_configurations.is_empty() { &defaults[..] } else { settings.reporter_configurations };
    let mut parsed: Vec<ReporterConfiguration> = Vec::new();
    for configuration in configurations {
        let configuration = parse_reporter_configuration(configuration, settings).map_err(Exit::Crash)?;
        if !parsed.contains(&configuration) {
            parsed.push(configuration);
        }
    }
    if matches!(baseline.status, BaselineStatus::Invalid | BaselineStatus::NotFound) {
        parsed.push(ReporterConfiguration {
            id: "baseline".to_owned(),
            artifact: None,
            additional_config: ReporterOptions::default(),
            output: baseline.path.clone(),
        });
    }
    let artifacts: Vec<String> = parsed.iter().filter_map(|c| c.artifact.clone()).collect();
    if let Some(url) = to_files_uri_list(&artifacts, cx.working_dir, cx.user_home, cx.logger)?.first() {
        let interface = format!("{}.cli.reporter.core.api.ReporterProviderV2", package(cx.logger.ktlint_version()));
        return Err(load_from_jar_file(url, &interface, cx.logger));
    }
    let mut reporters: Vec<Box<dyn ReporterV2>> = Vec::new();
    let ids: Vec<&str> = REPORTER_PROVIDER_IDS.iter().copied().chain(settings.github.then_some(REPORTER_ID)).collect();
    for configuration in &parsed {
        if !ids.contains(&configuration.id.as_str()) {
            cx.logger.error(REPORTER_AGGREGATOR, || {
                format!("reporter \"{}\" wasn't found (available: {})", configuration.id, ids.join(", "))
            });
            return Err(Exit::Code(ExitCode::InvalidReporterConfiguration));
        }
        reporters.push(to_reporter_v2(configuration, settings, cx)?);
    }
    if let Some(events) = settings.gradle_events {
        let path = cx.working_dir.resolve(events).map(|p| p.to_path_buf()).unwrap_or_else(|| events.into());
        let file = File::create(&path).map_err(|e| Exit::Crash(format!("java.io.FileNotFoundException: {events} ({e})")))?;
        reporters.push(Box::new(GradleEventsReporter::new(file)));
    }
    Ok(AggregatedReporter { reporters })
}

fn parse_reporter_configuration(configuration: &str, settings: &ReporterSettings) -> Result<ReporterConfiguration, String> {
    let elements: Vec<&str> = configuration.split(',').collect();
    let option = |name: &str| {
        let prefix = format!("{name}=");
        elements.iter().rev().find(|e| e.starts_with(&prefix)).map(|e| e.split_once('=').unwrap().1.to_owned())
    };
    let first = elements[0];
    let mut additional_config = parse_query(first.split_once('?').map_or(first, |(_, query)| query))?;
    additional_config.put("color", &settings.color.to_string());
    additional_config.put("color_name", settings.color_name);
    additional_config.put("format", &settings.format.to_string());
    Ok(ReporterConfiguration {
        id: first.split_once('?').map_or(first, |(id, _)| id).to_owned(),
        artifact: option("artifact"),
        additional_config,
        output: option("output"),
    })
}

fn parse_query(query: &str) -> Result<ReporterOptions, String> {
    let mut map = ReporterOptions::default();
    for s in query.split('&').filter(|s| !s.is_empty()) {
        let (key, value) = s.split_once('=').unwrap_or((s, "true"));
        map.put(key, &url_decode(value)?);
    }
    Ok(map)
}

/// `URLDecoder.decode(s, "UTF-8")`; `Err` is its `IllegalArgumentException`.
fn url_decode(s: &str) -> Result<String, String> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' => {
                let hex = s.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(h, 16).ok());
                let Some(byte) = hex else {
                    return Err("java.lang.IllegalArgumentException: URLDecoder: Illegal hex characters in escape (%) pattern".to_owned());
                };
                out.push(byte);
                i += 2;
            }
            b => out.push(b),
        }
        i += 1;
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

fn to_reporter_v2(configuration: &ReporterConfiguration, settings: &ReporterSettings, cx: &Context) -> Result<Box<dyn ReporterV2>, Exit> {
    cx.logger.debug(REPORTER_AGGREGATOR, || {
        let output = configuration.output.as_ref().map(|o| format!(", output={o}")).unwrap_or_default();
        format!("Initializing \"{}\" reporter with {}{output}", configuration.id, configuration.additional_config.to_java_string())
    });
    let sink = match &configuration.output {
        Some(output) => {
            let path = cx.working_dir.resolve(output).map(|p| p.to_path_buf()).unwrap_or_else(|| output.into());
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
                && fs::create_dir_all(parent).is_err()
            {
                return Err(Exit::Crash(format!("java.io.IOException: Unable to create \"{}\" directory", parent.display())));
            }
            let file = File::create(&path).map_err(|e| Exit::Crash(format!("java.io.FileNotFoundException: {output} ({e})")))?;
            Sink::File(BufWriter::new(file))
        }
        None if settings.stdin => Sink::Err(cx.console.clone()),
        None => Sink::Out(cx.console.clone()),
    };
    let reporter: Box<dyn ReporterV2> = if configuration.id == REPORTER_ID {
        Box::new(GithubReporter::new(Printer::new(sink), Annotations::from_env(&cx.env.working_dir)))
    } else {
        get_reporter(&configuration.id, Printer::new(sink), &configuration.additional_config, cx.env)
            .expect("a built-in reporter id")
            .map_err(Exit::Crash)?
    };
    Ok(match &configuration.output {
        Some(output) => {
            let absolute = cx.working_dir.resolve(output).unwrap_or_else(|| cx.working_dir.clone());
            let message = format!("\"{}\" report written to {}", configuration.id, location(&absolute, settings.relative, cx.working_dir));
            Box::new(FileReporter { reporter, logger: cx.logger.clone(), message })
        }
        None => reporter,
    })
}

/// A reporter with `output=`: after its `afterAll` the location is logged (the file is flushed when the
/// reporter is dropped).
struct FileReporter {
    reporter: Box<dyn ReporterV2>,
    logger: Logger,
    message: String,
}

impl ReporterV2 for FileReporter {
    fn before_all(&mut self) {
        self.reporter.before_all();
    }

    fn before(&mut self, file: &str) {
        self.reporter.before(file);
    }

    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        self.reporter.on_lint_error(file, ktlint_cli_error);
    }

    fn after(&mut self, file: &str) {
        self.reporter.after(file);
    }

    fn after_all(&mut self) {
        self.reporter.after_all();
        self.logger.info(REPORTER_AGGREGATOR, || self.message.clone());
    }
}

pub struct AggregatedReporter {
    reporters: Vec<Box<dyn ReporterV2>>,
}

impl ReporterV2 for AggregatedReporter {
    fn before_all(&mut self) {
        self.reporters.iter_mut().for_each(|r| r.before_all());
    }

    fn before(&mut self, file: &str) {
        self.reporters.iter_mut().for_each(|r| r.before(file));
    }

    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        self.reporters.iter_mut().for_each(|r| r.on_lint_error(file, ktlint_cli_error));
    }

    fn after(&mut self, file: &str) {
        self.reporters.iter_mut().for_each(|r| r.after(file));
    }

    fn after_all(&mut self) {
        self.reporters.iter_mut().for_each(|r| r.after_all());
    }
}
