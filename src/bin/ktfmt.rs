#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args = ktrs_cli::java_launcher::application_args();
    std::process::exit(ktrs_cli::ktfmt::main(&args));
}
