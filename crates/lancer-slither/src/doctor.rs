//! `lancer-slither-doctor` — diagnose toolchain.

pub fn run() -> eyre::Result<i32> {
    let mut errors = 0;
    println!("== lancer-slither doctor ==");
    check(&mut errors, "solc", &["--version"]);
    check(&mut errors, "forge", &["--version"]);
    check(&mut errors, "npx", &["--version"]);
    check(&mut errors, "truffle", &["version"]);
    check(&mut errors, "brownie", &["--version"]);
    check(&mut errors, "vyper", &["--version"]);
    if errors == 0 {
        println!("All checks passed.");
    } else {
        println!("{errors} tool(s) missing or broken.");
    }
    Ok(if errors == 0 { 0 } else { 1 })
}

fn check(errors: &mut u32, bin: &str, args: &[&str]) {
    match which::which(bin) {
        Ok(p) => {
            let out = std::process::Command::new(&p).args(args).output();
            if let Ok(o) = out {
                let v = String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").to_owned();
                println!("  [OK]    {bin}: {} ({})", v.trim(), p.display());
            } else {
                println!("  [WARN]  {bin}: present at {} but did not respond", p.display());
            }
        }
        Err(_) => {
            println!("  [MISS]  {bin}: not on PATH");
            *errors += 1;
        }
    }
}
