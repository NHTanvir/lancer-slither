//! `--print halstead` — Halstead complexity metrics.
//!
//! `n1` = distinct operators, `n2` = distinct operands, `N1` / `N2` = total
//! counts. Derives vocabulary, length, volume, difficulty, effort.

use std::io::Write;

use slither_core::Slither;

use crate::Printer;

pub struct Halstead;
impl Printer for Halstead {
    fn argument(&self) -> &'static str { "halstead" }
    fn help(&self) -> &'static str { "Halstead complexity metrics" }
    fn output(&self, slither: &Slither, out: &mut dyn Write) -> eyre::Result<()> {
        for (c, f) in slither.all_functions() {
            let m = compute(&f.body_excerpt);
            writeln!(
                out,
                "{}.{:<20} n1={} n2={} N1={} N2={} vocab={} length={} vol={:.1} diff={:.2} effort={:.0}",
                c.name, f.name, m.n1, m.n2, m.N1, m.N2, m.vocab, m.length, m.volume, m.difficulty, m.effort,
            )?;
        }
        Ok(())
    }
}

#[allow(non_snake_case)]
struct Metrics {
    n1: usize, n2: usize, N1: usize, N2: usize,
    vocab: usize, length: usize, volume: f64, difficulty: f64, effort: f64,
}

#[allow(non_snake_case)]
fn compute(body: &str) -> Metrics {
    let operators: &[&str] = &[
        "+", "-", "*", "/", "%", "**",
        "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!",
        "&", "|", "^", "~", "<<", ">>",
        "=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=",
        "if", "else", "for", "while", "return", "break", "continue",
    ];
    let mut op_counts = ahash::AHashMap::new();
    let mut operand_counts = ahash::AHashMap::new();
    for op in operators {
        let c = body.matches(op).count();
        if c > 0 { op_counts.insert(*op, c); }
    }
    let re = regex::Regex::new(r"\b[A-Za-z_]\w*\b").expect("regex");
    for m in re.find_iter(body) {
        *operand_counts.entry(m.as_str().to_owned()).or_insert(0usize) += 1;
    }
    let n1 = op_counts.len();
    let n2 = operand_counts.len();
    let big_n1: usize = op_counts.values().sum();
    let big_n2: usize = operand_counts.values().sum();
    let vocab = n1 + n2;
    let length = big_n1 + big_n2;
    let volume = if vocab == 0 { 0.0 } else { (length as f64) * (vocab as f64).log2() };
    let difficulty = if n2 == 0 { 0.0 } else { (n1 as f64) / 2.0 * (big_n2 as f64) / (n2 as f64) };
    let effort = difficulty * volume;
    Metrics {
        n1, n2, N1: big_n1, N2: big_n2,
        vocab, length, volume, difficulty, effort,
    }
}
