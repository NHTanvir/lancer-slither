use eframe::egui;

use crate::gui::LancerApp;

pub fn render(app: &mut LancerApp, ui: &mut egui::Ui) {
    ui.heading("Findings");
    let findings = app.findings.lock().clone();
    let q = app.search.to_lowercase();
    let visible: Vec<(usize, &slither_core::Finding)> = findings
        .iter()
        .enumerate()
        .filter(|(_, f)| app.impact_matches(f.impact))
        .filter(|(_, f)| q.is_empty() || f.check.to_lowercase().contains(&q) || f.description.to_lowercase().contains(&q))
        .collect();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (idx, f) in visible {
            let bg = crate::gui::color_for_impact(f.impact);
            let row = egui::RichText::new(format!(
                "[{}] {:<28} {:<7} {}",
                f.impact,
                f.check,
                f.confidence,
                f.first_markdown_element
            ))
            .color(bg);
            let resp = ui.selectable_label(app.selected == Some(idx), row);
            if resp.clicked() {
                app.selected = Some(idx);
            }
            if resp.double_clicked() {
                if let Some(e) = f.elements.first() {
                    let _ = arboard_copy(&e.source_mapping.pretty());
                }
            }
        }
    });
}

fn arboard_copy(text: &str) -> Result<(), arboard::Error> {
    let mut c = arboard::Clipboard::new()?;
    c.set_text(text)?;
    Ok(())
}
