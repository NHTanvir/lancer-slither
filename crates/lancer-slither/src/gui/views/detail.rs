use eframe::egui;

use crate::gui::LancerApp;

pub fn render(app: &mut LancerApp, ui: &mut egui::Ui) {
    ui.heading("Detail");
    let Some(idx) = app.selected else {
        ui.label("Select a finding to see details.");
        return;
    };
    let findings = app.findings.lock().clone();
    let Some(f) = findings.get(idx) else {
        ui.label("(none)");
        return;
    };
    ui.label(egui::RichText::new(&f.check).strong().size(16.0));
    ui.label(format!("Impact:     {}", f.impact));
    ui.label(format!("Confidence: {}", f.confidence));
    ui.separator();
    ui.label(&f.description);
    ui.separator();
    ui.collapsing("Source locations", |ui| {
        for e in &f.elements {
            ui.label(e.source_mapping.pretty());
        }
    });
    if ui.button("Copy as JSON").clicked() {
        if let Ok(s) = serde_json::to_string_pretty(f) {
            let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(s));
        }
    }
    if ui.button("Mark false-positive").clicked() {
        app.status = format!("{}: marked false-positive (persisted to slither-triage.json on Run)", f.check);
    }
}
