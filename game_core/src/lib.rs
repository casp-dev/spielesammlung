use dioxus::prelude::*;
use egui::Ui;

/// legacy trait für EGUI
pub trait Game {
    fn name(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui);
}

/// neuer trait für dioxus-spiele
pub trait DioxusGame {
    fn name(&self) -> &str;
    fn render(&self) -> Element;
}
