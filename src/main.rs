fn main() {
    mindmirror::init_tracing();
    if let Err(e) = mindmirror::run() {
        eprintln!("error: {e}");
        // Print the chain of sources for context.
        let mut src = std::error::Error::source(&e);
        while let Some(s) = src {
            eprintln!("  caused by: {s}");
            src = s.source();
        }
        std::process::exit(1);
    }
}
