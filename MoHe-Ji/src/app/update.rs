use super::VectorEditorApp;

pub fn update(app: &mut VectorEditorApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    super::view::show(app, ctx);
}
