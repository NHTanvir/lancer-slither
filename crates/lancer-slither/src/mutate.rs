//! `lancer-slither-mutate` — mutation testing.
//!
//! Walks `.sol` sources, applies operator mutations (`+`→`-`, `<`→`<=`, `==`→`!=`),
//! re-runs `forge test`, and reports surviving mutants.

use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct Args {
    pub target: PathBuf,
    /// Test command (Foundry by default).
    #[arg(long, default_value = "forge test")]
    pub test_cmd: String,
    /// Limit mutants per file (cap).
    #[arg(long, default_value_t = 10)]
    pub max_per_file: usize,
}

pub fn run(args: &Args) -> eyre::Result<i32> {
    let files = walkdir::WalkDir::new(&args.target)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("sol"))
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>();

    let mutators: &[(&str, &str)] = &[
        ("+", "-"),
        ("-", "+"),
        ("*", "/"),
        ("<", "<="),
        (">", ">="),
        ("==", "!="),
        ("!=", "=="),
        ("true", "false"),
        ("false", "true"),
    ];

    let mut killed = 0usize;
    let mut survived = 0usize;
    for file in &files {
        let orig = std::fs::read_to_string(file)?;
        let mut applied = 0;
        for (from, to) in mutators {
            if applied >= args.max_per_file { break; }
            if let Some(pos) = orig.find(from) {
                let mut mutated = orig.clone();
                mutated.replace_range(pos..pos + from.len(), to);
                std::fs::write(file, &mutated)?;
                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&args.test_cmd)
                    .current_dir(&args.target)
                    .status();
                let test_failed = matches!(status, Ok(s) if !s.success());
                if test_failed {
                    killed += 1;
                } else {
                    survived += 1;
                    println!("SURVIVED: {} {}→{} @ offset {}", file.display(), from, to, pos);
                }
                std::fs::write(file, &orig)?;
                applied += 1;
            }
        }
    }
    println!("mutants killed:    {killed}");
    println!("mutants survived:  {survived}");
    Ok(if survived == 0 { 0 } else { 1 })
}
