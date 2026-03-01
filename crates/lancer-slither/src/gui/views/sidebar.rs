use eframe::egui;

use crate::gui::LancerApp;

pub fn render(app: &mut LancerApp, ui: &mut egui::Ui) {
    ui.heading("Input");
    ui.label("Path or 0x… address:");
    let response = ui.text_edit_singleline(&mut app.target_input);
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        app.target = Some(app.target_input.clone().into());
    }
    if ui.button("Pick…").clicked() {
        if let Some(p) = rfd::FileDialog::new().pick_folder() {
            app.target_input = p.display().to_string();
            app.target = Some(p);
        }
    }
    ui.separator();
    ui.heading("Severity filter");
    ui.checkbox(&mut app.impact_filter.high, "High");
    ui.checkbox(&mut app.impact_filter.medium, "Medium");
    ui.checkbox(&mut app.impact_filter.low, "Low");
    ui.checkbox(&mut app.impact_filter.info, "Informational");
    ui.checkbox(&mut app.impact_filter.opt, "Optimization");
    ui.separator();
    ui.heading("Search");
    ui.text_edit_singleline(&mut app.search);
    ui.separator();
    ui.heading("Tool status");
    ui.label(format!("detectors registered: {}", slither_detectors::all().len()));
    ui.label(format!("printers registered:  {}", slither_printers::all().len()));
    let dropped = drop_zone(ui);
    if let Some(p) = dropped {
        app.target_input = p.display().to_string();
        app.target = Some(p);
    }
}

fn drop_zone(ui: &mut egui::Ui) -> Option<std::path::PathBuf> {
    let mut out = None;
    ui.input(|i| {
        for file in &i.raw.dropped_files {
            if let Some(p) = &file.path {
                out = Some(p.clone());
            }
        }
    });
    out
}
