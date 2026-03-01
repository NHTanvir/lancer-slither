use eframe::egui;

use crate::gui::LancerApp;

pub fn render(app: &mut LancerApp, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open project…").clicked() {
                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                    app.target_input = p.display().to_string();
                    app.target = Some(p);
                }
                ui.close_menu();
            }
            if ui.button("Open file…").clicked() {
                if let Some(p) = rfd::FileDialog::new().add_filter("Solidity", &["sol"]).pick_file() {
                    app.target_input = p.display().to_string();
                    app.target = Some(p);
                }
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Export JSON…").clicked() {
                crate::gui::export_dialog(app);
                ui.close_menu();
            }
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        ui.menu_button("Tools", |ui| {
            if ui.button("Run all detectors").clicked() {
                if app.target.is_some() {
                    app.start_scan();
                }
                ui.close_menu();
            }
            if ui.button("Show detector list").clicked() {
                let names: Vec<String> = slither_detectors::all().into_iter().map(|d| d.argument().to_owned()).collect();
                app.status = format!("{} detectors registered: {}", names.len(), names.join(", "));
                ui.close_menu();
            }
            if ui.button("Show printer list").clicked() {
                let names: Vec<String> = slither_printers::all().into_iter().map(|p| p.argument().to_owned()).collect();
                app.status = format!("{} printers registered: {}", names.len(), names.join(", "));
                ui.close_menu();
            }
        });
        ui.menu_button("Settings", |ui| {
            ui.checkbox(&mut app.config.exclude_informational, "Exclude informational");
            ui.checkbox(&mut app.config.exclude_optimization, "Exclude optimization");
            ui.checkbox(&mut app.config.exclude_dependencies, "Exclude dependencies");
            ui.checkbox(&mut app.config.ai_rank, "AI rank findings (heuristic)");
        });
        ui.menu_button("Help", |ui| {
            if ui.button("About").clicked() {
                app.show_about = true;
                ui.close_menu();
            }
            if ui.button("Open docs in browser").clicked() {
                let _ = open::that("https://github.com/NHTanvir/lancer-slither");
                ui.close_menu();
            }
        });
    });
}

mod open {
    pub fn that(_url: &str) -> std::io::Result<()> { Ok(()) }
}
