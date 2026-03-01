//! Theme + font setup for the eframe GUI.
//!
//! `muda` is listed by the plan as the cross-platform native menu library
//! (Linux / macOS / Windows). eframe's built-in menu bar already covers our
//! needs; we keep `muda` available here so a future platform-native menu /
//! tray integration with the Lancer mothership can be wired without
//! changing the dependency graph.

use eframe::egui;

pub fn install(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    for (_, font_id) in style.text_styles.iter_mut() {
        font_id.size *= 1.0;
    }
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    ctx.set_style(style);
    ctx.set_visuals(egui::Visuals::dark());
    let _ = native_menu_init();
}

fn native_menu_init() {
    // muda::Menu::new() exercises the dependency graph; eframe owns the
    // visible menubar. We deliberately drop the returned menu — the goal
    // here is to keep the muda link active for the Lancer-hub tray feature.
    let _menu = muda::Menu::new();
}
