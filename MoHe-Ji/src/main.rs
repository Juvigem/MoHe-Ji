use eframe::egui;

fn load_window_icon() -> Option<egui::IconData> {
    // Embed the icon image in the executable so the window/titlebar/taskbar icon
    // does not depend on the current working directory at runtime.
    let bytes = include_bytes!("../assets/app_icon.png");
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("MoHe-Ji")
        .with_icon(load_window_icon().unwrap_or_else(|| egui::IconData {
            rgba: vec![0, 0, 0, 255],
            width: 1,
            height: 1,
        }));

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "MoHe-Ji",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Ok(Box::new(app::VectorEditorApp::new(cc)))
        }),
    )
}

mod app;
mod io;
mod model;
mod ppw;
mod render;
