fn main() {
    if let Err(err) = to::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
