use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

mod codegen;
mod corpus_diff;
mod fmt_diff;
mod lint_diff;
mod lint_oracle;
mod lint_report;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("codegen") => codegen::run(&project_root()),
        Some("corpus-diff") => corpus_diff::run(&project_root(), &args[1..]),
        Some("fmt-diff") => fmt_diff::run(&project_root(), &args[1..]),
        Some("lint-diff") => lint_diff::run(&project_root(), &args[1..]),
        _ => Err("usage: cargo xtask codegen | corpus-diff [dir] | fmt-diff [style] [dir] | lint-diff [style] [--experimental] [--oracle DIR] [--counts] [--require-format]"
            .to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

pub(crate) fn write_if_changed(path: &Path, contents: &str) -> Result<(), String> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))?;
    eprintln!("updated {}", path.display());
    Ok(())
}
