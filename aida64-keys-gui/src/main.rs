#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use crate::app::App;

fn main() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: false,
        resizable: false,
        initial_window_size: Some(eframe::egui::Vec2::new(520.0, 300.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(App::default()), options);
}
