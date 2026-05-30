//! Native GUI frontend (eframe/egui) — the default `lancer-frida` experience.
//!
//! Plan ## GUI (Lancer-standard pattern): a three-pane desktop window (left
//! sidebar / center main view / right detail pane) with a top menu bar and a
//! bottom status bar, sitting on top of the GUI-agnostic `core` library, plus a
//! local HTTP integration server for cross-Lancer-tool communication.

mod app;
pub mod server;

/// Launch the native GUI window. Blocks until the window is closed.
pub fn launch() -> anyhow::Result<()> {
    use eframe::egui;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([880.0, 560.0])
            .with_title("lancer-frida"),
        ..Default::default()
    };

    eframe::run_native(
        "lancer-frida",
        native_options,
        Box::new(|cc| Ok(Box::new(app::LancerApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("GUI failed to start: {e}"))
}
