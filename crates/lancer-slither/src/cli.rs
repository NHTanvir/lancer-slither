//! Full clap-derived CLI mirroring upstream `slither` 0.11.5 flag surface.

use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};
use slither_core::SlitherConfig;

/// `lancer-slither [target] [flags]`
#[derive(Debug, Parser)]
#[command(
    name = "lancer-slither",
    version,
    about = "Solidity static analyzer (Rust port of crytic/slither, GUI-by-default).",
    long_about = "lancer-slither — part of the Lancer ecosystem.\n\
                  Without arguments, opens the GUI. Pass `--cli` (or a positional target) to use the CLI."
)]
pub struct Cli {
    /// Target: directory, .sol file, .json, or 0x-address (Etherscan).
    pub target: Option<String>,

    /// Force the CLI surface (skip GUI launch).
    #[arg(long, action = ArgAction::SetTrue)]
    pub cli: bool,

    // ----- Detector control -----
    #[arg(long, value_name = "list")]
    pub detect: Option<String>,
    #[arg(long, value_name = "list")]
    pub exclude: Option<String>,
    #[arg(long, value_name = "list")]
    pub include_detectors: Option<String>,
    #[arg(long)]
    pub exclude_dependencies: bool,
    #[arg(long)]
    pub exclude_optimization: bool,
    #[arg(long)]
    pub exclude_informational: bool,
    #[arg(long)]
    pub exclude_low: bool,
    #[arg(long)]
    pub exclude_medium: bool,
    #[arg(long)]
    pub exclude_high: bool,
    #[arg(long)]
    pub show_ignored_findings: bool,
    #[arg(long)]
    pub filter_paths: Option<String>,
    #[arg(long)]
    pub include_paths: Option<String>,

    // ----- Printer control -----
    #[arg(long, value_name = "name")]
    pub print: Option<String>,
    #[arg(long)]
    pub list_detectors: bool,
    #[arg(long)]
    pub list_printers: bool,

    // ----- Output formats -----
    #[arg(long, value_name = "out.json")]
    pub json: Option<PathBuf>,
    #[arg(long, value_name = "out.sarif")]
    pub sarif: Option<PathBuf>,
    #[arg(long, value_name = "out.md")]
    pub markdown: Option<PathBuf>,
    #[arg(long)]
    pub markdown_root: Option<String>,
    #[arg(long)]
    pub checklist: bool,
    #[arg(long, value_name = "out.zip")]
    pub zip: Option<PathBuf>,
    #[arg(long)]
    pub no_fail_pedantic: bool,
    #[arg(long)]
    pub fail_pedantic: bool,
    #[arg(long)]
    pub fail_low: bool,
    #[arg(long)]
    pub fail_medium: bool,
    #[arg(long)]
    pub fail_high: bool,

    // ----- crytic-compile pass-through -----
    #[arg(long, value_name = "framework")]
    pub compile_force_framework: Option<String>,
    #[arg(long, value_name = "lib=addr,...")]
    pub compile_libraries: Option<String>,
    #[arg(long)]
    pub compile_remove_metadata: bool,
    #[arg(long)]
    pub compile_custom_build: Option<String>,
    #[arg(long)]
    pub solc: Option<String>,
    #[arg(long)]
    pub solc_args: Option<String>,
    #[arg(long)]
    pub solc_disable_warnings: bool,
    #[arg(long)]
    pub solc_working_dir: Option<PathBuf>,
    #[arg(long)]
    pub solc_remaps: Option<String>,
    #[arg(long)]
    pub solc_solcs_select: Option<String>,
    #[arg(long)]
    pub solc_solcs_bin: Option<PathBuf>,
    #[arg(long)]
    pub ignore_compile: bool,
    #[arg(long)]
    pub skip_clean: bool,
    #[arg(long, value_name = "standard-input.json")]
    pub standard_json: Option<PathBuf>,

    // ----- Etherscan -----
    #[arg(long, env = "ETHERSCAN_API_KEY")]
    pub etherscan_apikey: Option<String>,
    #[arg(long)]
    pub etherscan_export_directory: Option<PathBuf>,
    #[arg(long)]
    pub etherscan_only_source_code: bool,
    #[arg(long)]
    pub etherscan_only_bytecode: bool,
    #[arg(long)]
    pub etherscan_export_dir_format: Option<String>,
    #[arg(long, default_value = "mainnet")]
    pub etherscan_network: String,

    // ----- Triage -----
    #[arg(long)]
    pub triage_mode: bool,
    #[arg(long)]
    pub triage_database: Option<PathBuf>,
    #[arg(long)]
    pub change_line_prefix: Option<String>,

    // ----- Config / misc -----
    #[arg(long, value_name = "slither.config.json")]
    pub config_file: Option<PathBuf>,
    #[arg(long)]
    pub disable_color: bool,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub ai_rank: bool,
    #[arg(long)]
    pub self_test: bool,
    #[arg(long)]
    pub mcp_server: bool,
    #[arg(long, value_name = "addr:port")]
    pub serve: Option<String>,
    #[arg(long, value_name = "baseline.json")]
    pub cross_check: Option<PathBuf>,

    #[command(subcommand)]
    pub sub: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// Read storage from a live contract.
    ReadStorage(crate::read_storage::Args),
    /// Storage layout differ for upgradeable proxies.
    CheckUpgradeability(crate::upgradeability::Args),
    /// Operator-mutation testing.
    Mutate(crate::mutate::Args),
    /// Echidna / Medusa property scaffolds.
    Prop(crate::prop::Args),
    /// Flatten a Solidity project into one file.
    Flat(crate::flatten::Args),
    /// Find paths between two functions in the callgraph.
    FindPaths(FindPathsArgs),
    /// Mechanical fix engine.
    Format(FormatArgs),
    /// ERC conformance checker.
    CheckErc(crate::erc::Args),
    /// Generate Solidity `interface` for a target.
    Interface(crate::interface_gen::Args),
    /// Diagnose toolchain issues.
    Doctor,
    /// Generate NatSpec scaffolds.
    Documentation(DocsArgs),
}

#[derive(Debug, clap::Args)]
pub struct FindPathsArgs {
    pub target: PathBuf,
    pub from: String,
    pub to: String,
}

#[derive(Debug, clap::Args)]
pub struct FormatArgs {
    pub target: PathBuf,
}

#[derive(Debug, clap::Args)]
pub struct DocsArgs {
    pub target: PathBuf,
    #[arg(long)]
    pub apply: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ExitMode {
    Ok,
    FailHigh,
    FailMedium,
    FailLow,
    FailPedantic,
}

impl Cli {
    /// Merge CLI flags into a `SlitherConfig`. CLI overrides config-file.
    pub fn into_config(&self) -> SlitherConfig {
        let mut cfg = if let Some(path) = &self.config_file {
            SlitherConfig::load(path).unwrap_or_default()
        } else if let Some(found) = SlitherConfig::discover(&std::env::current_dir().unwrap_or_else(|_| ".".into())) {
            SlitherConfig::load(&found).unwrap_or_default()
        } else {
            SlitherConfig::default()
        };
        if let Some(v) = &self.detect {
            cfg.detectors_to_run = v.clone();
        }
        if let Some(v) = &self.exclude {
            cfg.detectors_to_exclude = v.clone();
        }
        if let Some(v) = &self.include_detectors {
            cfg.include_detectors = v.clone();
        }
        cfg.exclude_dependencies |= self.exclude_dependencies;
        cfg.exclude_optimization |= self.exclude_optimization;
        cfg.exclude_informational |= self.exclude_informational;
        cfg.exclude_low |= self.exclude_low;
        cfg.exclude_medium |= self.exclude_medium;
        cfg.exclude_high |= self.exclude_high;
        cfg.show_ignored_findings |= self.show_ignored_findings;
        if self.filter_paths.is_some() {
            cfg.filter_paths = self.filter_paths.clone();
        }
        if self.include_paths.is_some() {
            cfg.include_paths = self.include_paths.clone();
        }
        cfg.fail_pedantic |= self.fail_pedantic;
        cfg.no_fail_pedantic |= self.no_fail_pedantic;
        cfg.fail_low |= self.fail_low;
        cfg.fail_medium |= self.fail_medium;
        cfg.fail_high |= self.fail_high;
        cfg.compile_force_framework = self.compile_force_framework.clone().or(cfg.compile_force_framework);
        cfg.compile_libraries = self.compile_libraries.clone().or(cfg.compile_libraries);
        cfg.compile_remove_metadata |= self.compile_remove_metadata;
        cfg.compile_custom_build = self.compile_custom_build.clone().or(cfg.compile_custom_build);
        cfg.solc = self.solc.clone().or(cfg.solc);
        cfg.solc_args = self.solc_args.clone().or(cfg.solc_args);
        cfg.solc_disable_warnings |= self.solc_disable_warnings;
        cfg.solc_working_dir = self.solc_working_dir.clone().or(cfg.solc_working_dir);
        cfg.solc_remaps = self.solc_remaps.clone().or(cfg.solc_remaps);
        cfg.solc_solcs_select = self.solc_solcs_select.clone().or(cfg.solc_solcs_select);
        cfg.solc_solcs_bin = self.solc_solcs_bin.clone().or(cfg.solc_solcs_bin);
        cfg.ignore_compile |= self.ignore_compile;
        cfg.skip_clean |= self.skip_clean;
        cfg.standard_json = self.standard_json.clone().or(cfg.standard_json);
        cfg.etherscan_apikey = self.etherscan_apikey.clone().or(cfg.etherscan_apikey);
        cfg.etherscan_export_directory = self.etherscan_export_directory.clone().or(cfg.etherscan_export_directory);
        cfg.etherscan_only_source_code |= self.etherscan_only_source_code;
        cfg.etherscan_only_bytecode |= self.etherscan_only_bytecode;
        cfg.etherscan_export_dir_format = self.etherscan_export_dir_format.clone().or(cfg.etherscan_export_dir_format);
        cfg.etherscan_network = Some(self.etherscan_network.clone());
        cfg.triage_mode |= self.triage_mode;
        cfg.triage_database = self.triage_database.clone().or(cfg.triage_database);
        cfg.change_line_prefix = self.change_line_prefix.clone().or(cfg.change_line_prefix);
        cfg.json = self.json.clone().or(cfg.json);
        cfg.sarif = self.sarif.clone().or(cfg.sarif);
        cfg.markdown = self.markdown.clone().or(cfg.markdown);
        cfg.markdown_root = self.markdown_root.clone().or(cfg.markdown_root);
        cfg.checklist |= self.checklist;
        cfg.zip = self.zip.clone().or(cfg.zip);
        cfg.disable_color |= self.disable_color;
        cfg.ai_rank |= self.ai_rank;
        cfg.mcp_server |= self.mcp_server;
        cfg.print = self.print.clone().or(cfg.print);
        cfg
    }
}

/// Run the full CLI surface. Returns process exit code.
pub fn run(cli: &Cli) -> eyre::Result<i32> {
    install_logger(cli.debug);

    if cli.list_detectors {
        list_detectors();
        return Ok(0);
    }
    if cli.list_printers {
        list_printers();
        return Ok(0);
    }
    if cli.self_test {
        return crate::self_test::run();
    }
    if let Some(addr) = &cli.serve {
        return crate::server::serve(addr);
    }

    if let Some(sub) = &cli.sub {
        return dispatch_sub(sub);
    }

    let Some(target) = &cli.target else {
        eprintln!("error: no target supplied. Pass a path or address, or use `--list-detectors`.");
        return Ok(2);
    };

    let config = cli.into_config();
    let target_path = std::path::PathBuf::from(target);
    let loaded = slither_core::project::load_project(&target_path, config)?;
    let slither = loaded.slither;

    if let Some(printer) = &cli.print {
        let Some(p) = slither_printers::find(printer) else {
            eprintln!("unknown printer: {printer}");
            return Ok(2);
        };
        let mut stdout = std::io::stdout().lock();
        p.output(&slither, &mut stdout)?;
        return Ok(0);
    }

    let mut findings = slither_detectors::run_all(&slither);

    if cli.ai_rank {
        crate::ai_rank::rank(&mut findings)?;
    }

    let results = slither_core::OutputResults {
        success: true,
        error: None,
        results: slither_core::finding::ResultsBucket {
            detectors: findings.clone(),
            printers: Vec::new(),
        },
    };

    let mut color = !cli.disable_color;
    if std::env::var_os("NO_COLOR").is_some() {
        color = false;
    }
    println!("{}", slither_core::output::text::render(&results, color));

    if let Some(json_out) = &slither.config.json {
        slither_core::output::json::write(json_out, &results)?;
    }
    if let Some(sarif_out) = &slither.config.sarif {
        slither_core::output::sarif::write(sarif_out, &results)?;
    }
    if let Some(md_out) = &slither.config.markdown {
        slither_core::output::markdown::write(md_out, &results, slither.config.markdown_root.as_deref())?;
    }
    if slither.config.checklist {
        let out = std::path::Path::new("slither-checklist.md");
        slither_core::output::markdown::write_checklist(out, &results)?;
    }
    if let Some(zip_out) = &slither.config.zip {
        slither_core::output::zip::write(zip_out, &results)?;
    }
    if let Some(baseline) = &cli.cross_check {
        let raw = std::fs::read_to_string(baseline)?;
        let old: slither_core::OutputResults = serde_json::from_str(&raw).unwrap_or_default();
        let diff = slither_core::output::cross_check::diff(&old, &results);
        println!("cross-check: {} same / {} added / {} removed", diff.same, diff.added.len(), diff.removed.len());
    }
    if slither.config.triage_mode {
        crate::ide::triage_loop(&slither, &results)?;
    }

    let code = compute_exit_code(&slither.config, &findings);
    Ok(code)
}

fn list_detectors() {
    println!("{:<32} {:<14} {:<10} HELP", "argument", "impact", "confidence");
    for d in slither_detectors::all() {
        println!(
            "{:<32} {:<14} {:<10} {}",
            d.argument(),
            d.impact().to_string(),
            d.confidence().to_string(),
            d.help()
        );
    }
}

fn list_printers() {
    println!("{:<22} HELP", "argument");
    for p in slither_printers::all() {
        println!("{:<22} {}", p.argument(), p.help());
    }
}

fn install_logger(debug: bool) {
    use tracing_subscriber::EnvFilter;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(if debug {
            EnvFilter::new("debug")
        } else {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
        })
        .try_init();
}

fn compute_exit_code(cfg: &SlitherConfig, findings: &[slither_core::Finding]) -> i32 {
    use slither_core::Impact;
    let has_high = findings.iter().any(|f| f.impact == Impact::High);
    let has_med = findings.iter().any(|f| f.impact == Impact::Medium);
    let has_low = findings.iter().any(|f| f.impact == Impact::Low);
    let has_any = !findings.is_empty();
    if cfg.fail_high && has_high {
        return 1;
    }
    if cfg.fail_medium && (has_high || has_med) {
        return 1;
    }
    if cfg.fail_low && (has_high || has_med || has_low) {
        return 1;
    }
    if cfg.fail_pedantic && has_any && !cfg.no_fail_pedantic {
        return 1;
    }
    0
}

fn dispatch_sub(sub: &SubCommand) -> eyre::Result<i32> {
    match sub {
        SubCommand::ReadStorage(a) => crate::read_storage::run(a),
        SubCommand::CheckUpgradeability(a) => crate::upgradeability::run(a),
        SubCommand::Mutate(a) => crate::mutate::run(a),
        SubCommand::Prop(a) => crate::prop::run(a),
        SubCommand::Flat(a) => crate::flatten::run(a),
        SubCommand::FindPaths(a) => find_paths_run(a),
        SubCommand::Format(a) => format_run(a),
        SubCommand::CheckErc(a) => crate::erc::run(a),
        SubCommand::Interface(a) => crate::interface_gen::run(a),
        SubCommand::Doctor => crate::doctor::run(),
        SubCommand::Documentation(a) => documentation_run(a),
    }
}

fn find_paths_run(a: &FindPathsArgs) -> eyre::Result<i32> {
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&a.target, cfg)?;
    let slither = loaded.slither;
    let re = regex::Regex::new(&format!(r"\b{}\s*\(", regex::escape(&a.to))).expect("regex");
    for (c, f) in slither.all_functions() {
        if f.full_name() == a.from || f.name == a.from {
            if re.is_match(&f.body_excerpt) {
                println!("{}.{} -> {}", c.name, f.name, a.to);
            }
        }
    }
    Ok(0)
}

fn format_run(a: &FormatArgs) -> eyre::Result<i32> {
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&a.target, cfg)?;
    let _slither = loaded.slither;
    println!("lancer-slither-format: mechanical fixes available for the following detectors:");
    println!("  - boolean-equal       (rewrite `x == true` -> `x`)");
    println!("  - missing-zero-check  (insert require(x != address(0)))");
    println!("  - constable-states    (add `constant` to qualifying state vars)");
    println!("Run with --apply to mutate sources (not implemented yet).");
    Ok(0)
}

fn documentation_run(a: &DocsArgs) -> eyre::Result<i32> {
    let cfg = SlitherConfig::default();
    let loaded = slither_core::project::load_project(&a.target, cfg)?;
    let slither = loaded.slither;
    for (c, f) in slither.all_functions() {
        println!("/// @notice TODO: describe {}.{}", c.name, f.name);
        for p in &f.parameters {
            println!("/// @param {} TODO", p.name);
        }
        for r in &f.returns {
            println!("/// @return {} TODO", if r.name.is_empty() { "ret" } else { &r.name });
        }
    }
    let _ = a.apply;
    Ok(0)
}
