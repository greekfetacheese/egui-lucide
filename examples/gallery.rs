//! Browse every Lucide icon and tweak size, stroke, and color.
//!
//! ```text
//! cargo run --example gallery
//! ```

use eframe::egui::{self, CentralPanel, Color32, Panel, ScrollArea, Slider, Vec2};
use egui_lucide::{Lucide, LucideIcon};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("egui-lucide")
            .with_inner_size([960.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui-lucide",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(GalleryApp::default()))
        }),
    )
}

struct GalleryApp {
    query: String,
    size: f32,
    stroke_width: f32,
    color: Color32,
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self {
            query: String::new(),
            size: 24.0,
            stroke_width: 2.0,
            color: Color32::from_rgb(0xe4, 0xe4, 0xe7),
        }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Panel::top("controls").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Search");
                ui.text_edit_singleline(&mut self.query);
                ui.separator();
                ui.add(Slider::new(&mut self.size, 12.0..=48.0).text("size"));
                ui.add(Slider::new(&mut self.stroke_width, 0.5..=3.0).text("stroke"));
                ui.color_edit_button_srgba(&mut self.color);
            });
            ui.add_space(4.0);
        });

        CentralPanel::default().show(ui, |ui| {
            let query = self.query.trim().to_ascii_lowercase();
            let filtered: Vec<Lucide> = Lucide::ALL
                .iter()
                .copied()
                .filter(|icon| query.is_empty() || icon.name().contains(&query))
                .collect();

            ui.label(format!(
                "{} icons  ·  lucide {}  ·  showing {}",
                Lucide::ALL.len(),
                egui_lucide::LUCIDE_VERSION,
                filtered.len()
            ));

            let gap = 8.0;
            let cell = self.size + 8.0;
            let cols = ((ui.available_width() + gap) / (cell + gap))
                .floor()
                .max(1.0) as usize;
            let rows = filtered.len().div_ceil(cols);
            let row_height = cell + gap;

            ScrollArea::vertical().show_rows(ui, row_height, rows, |ui, row_range| {
                for row in row_range {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::splat(gap);
                        for col in 0..cols {
                            let index = row * cols + col;
                            let Some(&icon) = filtered.get(index) else {
                                break;
                            };
                            let response = ui.add(
                                LucideIcon::new(icon)
                                    .size(self.size)
                                    .stroke_width(self.stroke_width)
                                    .color(self.color),
                            );
                            response.on_hover_text(icon.name());
                        }
                    });
                }
            });
        });
    }
}
