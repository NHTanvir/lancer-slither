use eframe::egui;

use crate::gui::LancerApp;

pub fn render(app: &mut LancerApp, ctx: &egui::Context) {
    egui::Window::new("About lancer-slither")
        .collapsible(false)
        .resizable(false)
        .open(&mut app.show_about)
        .show(ctx, |ui| {
            ui.label("lancer-slither — Solidity static analyzer.");
            ui.label("Part of the Lancer ecosystem.");
            ui.label(format!("Version: v{}", env!("CARGO_PKG_VERSION")));
            ui.label("Upstream parity target: crytic/slither 0.11.5");
            ui.separator();
            ui.label(format!("Detectors: {}", slither_detectors::all().len()));
            ui.label(format!("Printers:  {}", slither_printers::all().len()));
            ui.label("Repo: https://github.com/NHTanvir/lancer-slither");
        });
}
