//! One golden case on disk (`testdata/detekt/<RuleName>/<case>.*`, written by
//! tools/detekt-tests/extract/CaseRecorder.kt): `.input.kt|kts`, `.options.json` (`rule`, `path` = the file's
//! `virtualFilePath`, `config` = the test's `TestConfig` pairs with their types, `test`), and what the real rule
//! returned from `visitFile`: `.findings` rows `line:col\tendLine:endCol\tstart:end\t<signature>\t<message>`
//! (absent when none) or `.error` (the exception it threw).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ktrs_detekt::api::{Config, Value, config};

pub struct Case {
    pub name: String,
    pub input: String,
    pub rule: String,
    pub path: String,
    /// `Err`: a value the recorder could not express.
    pub config: Result<Vec<(String, Value)>, String>,
    pub findings: Vec<String>,
    pub error: Option<String>,
}

/// Case base paths (`<dir>/<case>`, without `.input.kt[s]`), sorted.
pub fn collect(dir: &Path) -> Vec<PathBuf> {
    fn collect_into(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_into(&path, out);
            } else if let Some(base) = path.to_str().and_then(|p| p.strip_suffix(".input.kt").or_else(|| p.strip_suffix(".input.kts"))) {
                out.push(PathBuf::from(base));
            }
        }
    }
    let mut out = Vec::new();
    collect_into(dir, &mut out);
    out.sort();
    out
}

fn read(base: &Path, suffix: &str) -> Option<String> {
    fs::read_to_string(format!("{}{suffix}", base.display())).ok()
}

pub fn load(root: &Path, base: &Path) -> Case {
    let input = read(base, ".input.kt").or_else(|| read(base, ".input.kts")).unwrap();
    let options: serde_json::Value = serde_json::from_str(&read(base, ".options.json").unwrap()).unwrap();
    Case {
        name: base.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"),
        input,
        rule: options["rule"].as_str().unwrap().to_owned(),
        path: options["path"].as_str().unwrap().to_owned(),
        config: map_of(&options["config"]),
        findings: read(base, ".findings").map(|text| text.lines().map(str::to_owned).collect()).unwrap_or_default(),
        error: read(base, ".error"),
    }
}

fn map_of(json: &serde_json::Value) -> Result<Vec<(String, Value)>, String> {
    let object = json.as_object().ok_or("config is not an object")?;
    object.iter().map(|(key, value)| Ok((key.clone(), value_of(value)?))).collect()
}

fn value_of(json: &serde_json::Value) -> Result<Value, String> {
    Ok(match json {
        serde_json::Value::String(text) => Value::String(text.clone()),
        serde_json::Value::Bool(flag) => Value::Boolean(*flag),
        serde_json::Value::Number(number) => match number.as_i64() {
            Some(integer) => i32::try_from(integer).map_or(Value::Long(integer), Value::Int),
            None => Value::Double(number.as_f64().unwrap()),
        },
        serde_json::Value::Array(items) => Value::List(items.iter().map(value_of).collect::<Result<_, _>>()?),
        serde_json::Value::Object(object) if object.keys().any(|key| key.starts_with("unsupported ")) => {
            return Err(format!("unsupported: {json}"));
        }
        serde_json::Value::Object(_) => Value::Map(map_of(json)?),
        serde_json::Value::Null => return Err("null value".to_owned()),
    })
}

/// detekt-test's `TestConfig`: the pairs as given, no coercion; its parent is `Config.empty`.
pub struct TestConfig {
    values: Vec<(String, Value)>,
    parent: Option<Arc<dyn Config>>,
}

impl TestConfig {
    pub fn new(values: Vec<(String, Value)>) -> Arc<dyn Config> {
        Arc::new(TestConfig { values, parent: Some(config::empty()) })
    }
}

impl Config for TestConfig {
    fn parent(&self) -> Option<Arc<dyn Config>> {
        self.parent.clone()
    }

    fn sub_config(self: Arc<Self>, key: &str) -> Arc<dyn Config> {
        let values = match self.value_or_null(key) {
            Some(Value::Map(values)) => values,
            _ => Vec::new(),
        };
        Arc::new(TestConfig { values, parent: Some(self) })
    }

    fn sub_config_keys(&self) -> Vec<String> {
        self.values.iter().map(|(key, _)| key.clone()).collect()
    }

    fn value_or_null(&self, key: &str) -> Option<Value> {
        self.values.iter().find(|(k, _)| k == key).map(|(_, value)| value.clone())
    }
}
