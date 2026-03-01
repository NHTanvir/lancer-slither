//! `--triage-mode` — interactive walk through findings.

use std::io::{BufRead, Write};

use slither_core::{OutputResults, Slither};

pub fn triage_loop(slither: &Slither, results: &OutputResults) -> eyre::Result<()> {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    let mut db = (*slither.triage_db).clone();
    let total = results.results.detectors.len();
    for (idx, finding) in results.results.detectors.iter().enumerate() {
        writeln!(out, "==[{}/{}] {} [{}/{}]==", idx + 1, total, finding.check, finding.impact, finding.confidence)?;
        writeln!(out, "{}", finding.description)?;
        for e in &finding.elements {
            writeln!(out, "  {}", e.source_mapping.pretty())?;
        }
        write!(out, "(v)alid / (f)alse-positive / (w)on't-fix / (s)kip ? ")?;
        out.flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 { break; }
        let status = match line.trim() {
            "v" | "valid" => "valid",
            "f" | "false" | "fp" => "false-positive",
            "w" | "wont-fix" => "wont-fix",
            _ => continue,
        };
        db.decisions.insert(
            finding.id.clone(),
            slither_core::triage::TriageDecision {
                status: status.to_owned(),
                note: None,
                recorded_at: Some(chrono::Utc::now().to_rfc3339()),
            },
        );
    }
    let path = slither
        .config
        .triage_database
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("slither-triage.json"));
    db.save(&path)?;
    println!("triage decisions saved to {}", path.display());
    Ok(())
}
