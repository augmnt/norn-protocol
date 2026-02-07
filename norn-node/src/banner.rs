use console::Style;

const BANNER: &str = r#"
 ███╗   ██╗  ██████╗  ██████╗  ███╗   ██╗
 ████╗  ██║ ██╔═══██╗ ██╔══██╗ ████╗  ██║
 ██╔██╗ ██║ ██║   ██║ ██████╔╝ ██╔██╗ ██║
 ██║╚██╗██║ ██║   ██║ ██╔══██╗ ██║╚██╗██║
 ██║ ╚████║ ╚██████╔╝ ██║  ██║ ██║ ╚████║
 ╚═╝  ╚═══╝  ╚═════╝  ╚═╝  ╚═╝ ╚═╝  ╚═══╝"#;

/// Print the NORN startup banner with version info.
pub fn print_banner() {
    let purple = Style::new().magenta().bold();
    let dim = Style::new().dim();

    println!("{}", purple.apply_to(BANNER));
    println!(
        "  {}",
        dim.apply_to(format!(
            "v{} — Thread-based L1 with off-chain execution",
            env!("CARGO_PKG_VERSION")
        ))
    );
    println!();
}
