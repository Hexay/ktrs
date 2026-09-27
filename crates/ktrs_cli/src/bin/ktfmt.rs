#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let main = ktrs_cli::ktfmt::Main::new(std::io::stdin(), std::io::stdout(), std::io::stderr());
    std::process::exit(main.run(&args));
}
