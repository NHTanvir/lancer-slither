use lancer_slither::doctor::run;

fn main() -> eyre::Result<()> {
    let _ = color_eyre::install();
    let code = run()?;
    std::process::exit(code);
}
