//! eframe GUI shell — three-pane layout matching the Lancer mothership.

use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use egui::Color32;
use parking_lot::Mutex;
use slither_core::{Finding, Impact, SlitherConfig};

mod theme;
mod views;

/// Launch the GUI. Blocks until the window is closed.
pub fn launch() -> eyre::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0_f32, 800.0_f32])
            .with_min_inner_size([900.0_f32, 600.0_f32])
            .with_title("lancer-slither"),
        persist_window: true,
        ..Default::default()
    };
    let result = eframe::run_native(
        "lancer-slither",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Box::new(LancerApp::new(cc))
        }),
    );
    if let Err(e) = result {
        eyre::bail!("GUI failed: {e}");
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    Progress(String),
    Done(Vec<Finding>),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum WorkerCommand {
    Scan { target: PathBuf, config: SlitherConfig },
    Cancel,
}

pub struct LancerApp {
    target: Option<PathBuf>,
    target_input: String,
    findings: Arc<Mutex<Vec<Finding>>>,
    selected: Option<usize>,
    status: String,
    busy: bool,
    impact_filter: ImpactFilter,
    search: String,
    tx: Sender<WorkerCommand>,
    rx: Receiver<WorkerEvent>,
    show_about: bool,
    config: SlitherConfig,
    last_scan_started: Option<std::time::Instant>,
}

#[derive(Debug, Clone, Copy)]
struct ImpactFilter {
    high: bool,
    medium: bool,
    low: bool,
    info: bool,
    opt: bool,
}

impl Default for ImpactFilter {
    fn default() -> Self {
        Self { high: true, medium: true, low: true, info: false, opt: false }
    }
}

impl LancerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<WorkerCommand>();
        let (evt_tx, evt_rx) = crossbeam_channel::unbounded::<WorkerEvent>();
        std::thread::spawn(move || worker(cmd_rx, evt_tx));
        Self {
            target: None,
            target_input: String::new(),
            findings: Arc::new(Mutex::new(Vec::new())),
            selected: None,
            status: "ready".to_owned(),
            busy: false,
            impact_filter: ImpactFilter::default(),
            search: String::new(),
            tx: cmd_tx,
            rx: evt_rx,
            show_about: false,
            config: SlitherConfig::default(),
            last_scan_started: None,
        }
    }

    fn drain_worker(&mut self) {
        while let Ok(e) = self.rx.try_recv() {
            match e {
                WorkerEvent::Progress(s) => self.status = s,
                WorkerEvent::Done(fs) => {
                    *self.findings.lock() = fs;
                    self.busy = false;
                    self.status = format!("{} findings", self.findings.lock().len());
                }
                WorkerEvent::Error(e) => {
                    self.busy = false;
                    self.status = format!("error: {e}");
                }
            }
        }
    }

    fn start_scan(&mut self) {
        let Some(target) = self.target.clone() else { return };
        self.busy = true;
        self.last_scan_started = Some(std::time::Instant::now());
        self.status = "starting…".to_owned();
        let _ = self.tx.send(WorkerCommand::Scan {
            target,
            config: self.config.clone(),
        });
    }

    fn impact_matches(&self, i: Impact) -> bool {
        match i {
            Impact::High => self.impact_filter.high,
            Impact::Medium => self.impact_filter.medium,
            Impact::Low => self.impact_filter.low,
            Impact::Informational => self.impact_filter.info,
            Impact::Optimization => self.impact_filter.opt,
        }
    }
}

impl eframe::App for LancerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_worker();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            views::menu_bar::render(self, ui);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(if self.busy { "● working" } else { "○ idle" });
                ui.separator();
                ui.label(&self.status);
                ui.separator();
                ui.label(format!("items: {}", self.findings.lock().len()));
                if ui.button("Open").clicked() {
                    if let Some(p) = rfd::FileDialog::new().pick_folder().or_else(|| rfd::FileDialog::new().pick_file()) {
                        self.target_input = p.display().to_string();
                        self.target = Some(p);
                    }
                }
                if ui.button("Run").clicked() {
                    if let Some(_t) = &self.target {
                        self.start_scan();
                    }
                }
                if ui.button("Export").clicked() {
                    export_dialog(self);
                }
                if self.busy && ui.button("Stop").clicked() {
                    let _ = self.tx.send(WorkerCommand::Cancel);
                }
            });
        });

        egui::SidePanel::left("inputs")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                views::sidebar::render(self, ui);
            });

        egui::SidePanel::right("detail")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| {
                views::detail::render(self, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            views::findings_table::render(self, ui);
        });

        if self.show_about {
            views::about::render(self, ctx);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}

fn worker(cmd_rx: Receiver<WorkerCommand>, evt_tx: Sender<WorkerEvent>) {
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            WorkerCommand::Scan { target, config } => {
                let _ = evt_tx.send(WorkerEvent::Progress("loading project…".into()));
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    slither_core::project::load_project(&target, config)
                }));
                let loaded = match res {
                    Ok(Ok(l)) => l,
                    Ok(Err(e)) => { let _ = evt_tx.send(WorkerEvent::Error(e.to_string())); continue; }
                    Err(_) => { let _ = evt_tx.send(WorkerEvent::Error("project loader panicked".into())); continue; }
                };
                let _ = evt_tx.send(WorkerEvent::Progress("running detectors…".into()));
                let findings = slither_detectors::run_all(&loaded.slither);
                let _ = evt_tx.send(WorkerEvent::Done(findings));
            }
            WorkerCommand::Cancel => {}
        }
    }
}

fn export_dialog(app: &LancerApp) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .add_filter("SARIF", &["sarif"])
        .add_filter("Markdown", &["md"])
        .save_file()
    else {
        return;
    };
    let findings = app.findings.lock().clone();
    let results = slither_core::OutputResults {
        success: true,
        error: None,
        results: slither_core::finding::ResultsBucket {
            detectors: findings, printers: Vec::new(),
        },
    };
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let _ = match ext {
        "json" => slither_core::output::json::write(&path, &results),
        "sarif" => slither_core::output::sarif::write(&path, &results),
        "md" => slither_core::output::markdown::write(&path, &results, None),
        _ => slither_core::output::json::write(&path, &results),
    };
}

pub(crate) fn color_for_impact(i: Impact) -> Color32 {
    match i {
        Impact::High => Color32::from_rgb(220, 70, 70),
        Impact::Medium => Color32::from_rgb(220, 170, 50),
        Impact::Low => Color32::from_rgb(80, 130, 220),
        Impact::Informational => Color32::from_rgb(140, 140, 140),
        Impact::Optimization => Color32::from_rgb(120, 200, 140),
    }
}
