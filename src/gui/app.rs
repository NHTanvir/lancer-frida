//! The egui application: three-pane layout, menu bar, status bar, background
//! workers, and the integration server wired into the GUI state.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::core::{Backend, Device, DeviceManager, Process, Session};
use super::server::{self, Integration};

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Config {
    dark: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Tab {
    Processes,
    Editor,
    Console,
    Memory,
    Classes,
    Ai,
    Backends,
}

/// Messages from background worker threads back to the UI thread.
enum Msg {
    Devices(Result<Vec<Device>, String>),
    Processes(Result<Vec<Process>, String>),
    Console(String),
}

pub struct LancerApp {
    integration: Arc<Mutex<Integration>>,
    port: u16,

    devices: Vec<Device>,
    selected_device: Option<usize>,
    processes: Vec<Process>,
    process_filter: String,
    selected_process: Option<usize>,
    target_input: String,

    tab: Tab,
    script: String,
    console: String,

    mem_addr: String,
    mem_len: String,
    classes_ios: bool,

    ai_goal: String,
    ai_output: String,

    busy: usize,
    status: String,
    dark: bool,

    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl LancerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let cfg = load_config();
        cc.egui_ctx.set_visuals(if cfg.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        let integration = Arc::new(Mutex::new(Integration::default()));
        let port = server::start(Arc::clone(&integration)).unwrap_or(0);

        let (tx, rx) = std::sync::mpsc::channel();

        LancerApp {
            integration,
            port,
            devices: Vec::new(),
            selected_device: None,
            processes: Vec::new(),
            process_filter: String::new(),
            selected_process: None,
            target_input: String::new(),
            tab: Tab::Processes,
            script: default_script(),
            console: String::new(),
            mem_addr: String::new(),
            mem_len: "64".to_string(),
            classes_ios: false,
            ai_goal: String::new(),
            ai_output: String::new(),
            busy: 0,
            status: "idle".to_string(),
            dark: cfg.dark,
            tx,
            rx,
        }
    }

    fn selector(&self) -> Vec<String> {
        match self.selected_device {
            Some(i) => self.devices[i].selector(),
            None => Vec::new(),
        }
    }

    fn target(&self) -> Option<String> {
        if !self.target_input.trim().is_empty() {
            return Some(self.target_input.trim().to_string());
        }
        self.selected_process
            .and_then(|i| self.processes.get(i))
            .map(|p| p.pid.to_string())
    }

    /// Spawn a blocking job on a worker thread; deliver its `Msg` and repaint.
    fn spawn<F>(&mut self, ctx: &egui::Context, job: F)
    where
        F: FnOnce() -> Msg + Send + 'static,
    {
        self.busy += 1;
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let msg = job();
            let _ = tx.send(msg);
            ctx.request_repaint();
        });
    }

    fn refresh_devices(&mut self, ctx: &egui::Context) {
        self.status = "enumerating devices...".to_string();
        self.spawn(ctx, || {
            Msg::Devices(DeviceManager::enumerate_devices().map_err(|e| e.to_string()))
        });
    }

    fn refresh_processes(&mut self, ctx: &egui::Context) {
        let Some(i) = self.selected_device else {
            self.status = "select a device first".to_string();
            return;
        };
        let device = self.devices[i].clone();
        self.status = "listing processes...".to_string();
        self.spawn(ctx, move || {
            Msg::Processes(device.enumerate_processes(false).map_err(|e| e.to_string()))
        });
    }

    fn run_script(&mut self, ctx: &egui::Context) {
        let Some(target) = self.target() else {
            self.status = "no target — pick a process or type one".to_string();
            return;
        };
        let session = Session::new(self.selector(), target.clone());
        let source = self.script.clone();
        self.status = format!("running script on {target}...");
        self.spawn(ctx, move || match session.run_script(&source) {
            Ok(out) => Msg::Console(out),
            Err(e) => Msg::Console(format!("[error] {e}")),
        });
    }

    fn read_memory(&mut self, ctx: &egui::Context) {
        let Some(target) = self.target() else {
            self.status = "no target selected".to_string();
            return;
        };
        let session = Session::new(self.selector(), target);
        let addr = self.mem_addr.trim().to_string();
        let len: usize = self.mem_len.trim().parse().unwrap_or(64);
        if addr.is_empty() {
            self.status = "enter an address".to_string();
            return;
        }
        self.status = format!("reading {len} bytes @ {addr}...");
        self.spawn(ctx, move || match session.read_memory(&addr, len) {
            Ok(out) => Msg::Console(format!("{addr}:\n{out}")),
            Err(e) => Msg::Console(format!("[error] {e}")),
        });
    }

    fn list_classes(&mut self, ctx: &egui::Context) {
        let Some(target) = self.target() else {
            self.status = "no target selected".to_string();
            return;
        };
        let session = Session::new(self.selector(), target);
        let ios = self.classes_ios;
        self.status = "enumerating classes...".to_string();
        self.spawn(ctx, move || match session.enumerate_classes(ios) {
            Ok(out) => Msg::Console(out),
            Err(e) => Msg::Console(format!("[error] {e}")),
        });
    }

    fn ai_generate(&mut self, ctx: &egui::Context) {
        let goal = self.ai_goal.trim().to_string();
        if goal.is_empty() {
            self.status = "describe what to hook first".to_string();
            return;
        }
        self.status = "asking Claude...".to_string();
        let tx = self.tx.clone();
        let ctx2 = ctx.clone();
        self.busy += 1;
        std::thread::spawn(move || {
            let msg = match crate::commands::generate::call_claude(&format!(
                "Write a Frida 17.x script: {goal}. Output only JavaScript."
            )) {
                Ok(out) => Msg::Console(format!("// generated for: {goal}\n{out}")),
                Err(e) => Msg::Console(format!("[ai error] {e}")),
            };
            let _ = tx.send(msg);
            ctx2.request_repaint();
        });
    }

    fn drain(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            if self.busy > 0 {
                self.busy -= 1;
            }
            match msg {
                Msg::Devices(Ok(d)) => {
                    self.status = format!("{} device(s)", d.len());
                    self.devices = d;
                    self.selected_device = None;
                    self.processes.clear();
                }
                Msg::Devices(Err(e)) => self.console_err(&e),
                Msg::Processes(Ok(p)) => {
                    self.status = format!("{} process(es)", p.len());
                    self.processes = p;
                    self.selected_process = None;
                }
                Msg::Processes(Err(e)) => self.console_err(&e),
                Msg::Console(text) => {
                    self.status = "done".to_string();
                    if self.tab == Tab::Ai {
                        self.ai_output = text.clone();
                    }
                    if !self.console.is_empty() {
                        self.console.push('\n');
                    }
                    self.console.push_str(&text);
                }
            }
        }
        self.sync_integration();
    }

    fn console_err(&mut self, e: &str) {
        self.status = "error".to_string();
        if !self.console.is_empty() {
            self.console.push('\n');
        }
        self.console.push_str("[error] ");
        self.console.push_str(e);
    }

    fn sync_integration(&self) {
        if let Ok(mut s) = self.integration.lock() {
            s.status = self.status.clone();
            s.target = self.target();
            s.results = self.console.lines().map(|l| l.to_string()).collect();
            // surface any imported payloads into the script editor is handled in update().
        }
    }

    fn take_inbox(&mut self) -> Vec<String> {
        if let Ok(mut s) = self.integration.lock() {
            std::mem::take(&mut s.inbox)
        } else {
            Vec::new()
        }
    }

    fn set_theme(&mut self, ctx: &egui::Context, dark: bool) {
        self.dark = dark;
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        save_config(&Config { dark });
    }
}

impl eframe::App for LancerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain();

        // Pull any cross-tool imports into the console.
        for payload in self.take_inbox() {
            if !self.console.is_empty() {
                self.console.push('\n');
            }
            self.console.push_str("[import] ");
            self.console.push_str(&payload);
            self.status = "received import from another Lancer tool".to_string();
        }

        let mut act_refresh_devices = false;
        let mut act_refresh_procs = false;
        let mut act_run = false;
        let mut act_read_mem = false;
        let mut act_classes = false;
        let mut act_ai = false;
        let mut set_dark: Option<bool> = None;
        let mut clear_console = false;

        // ---- Top menu bar ----
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Refresh devices").clicked() {
                        act_refresh_devices = true;
                        ui.close_menu();
                    }
                    if ui.button("Clear console").clicked() {
                        clear_console = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Tools", |ui| {
                    if ui.button("Processes").clicked() {
                        self.tab = Tab::Processes;
                        ui.close_menu();
                    }
                    if ui.button("Script editor").clicked() {
                        self.tab = Tab::Editor;
                        ui.close_menu();
                    }
                    if ui.button("Memory viewer").clicked() {
                        self.tab = Tab::Memory;
                        ui.close_menu();
                    }
                    if ui.button("Class browser").clicked() {
                        self.tab = Tab::Classes;
                        ui.close_menu();
                    }
                    if ui.button("AI panel").clicked() {
                        self.tab = Tab::Ai;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("Dark theme").clicked() {
                        set_dark = Some(true);
                        ui.close_menu();
                    }
                    if ui.button("Light theme").clicked() {
                        set_dark = Some(false);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.label("lancer-frida");
                    ui.label(format!("integration port: {}", self.port));
                    ui.hyperlink_to("Frida docs", "https://frida.re/docs/");
                });
            });
        });

        // ---- Bottom status bar ----
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.busy > 0 {
                    ui.spinner();
                }
                ui.label(format!("Status: {}", self.status));
                ui.separator();
                ui.label(format!("Items: {}", self.console.lines().count()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Run").clicked() {
                        act_run = true;
                    }
                    if ui.button("Refresh").clicked() {
                        act_refresh_devices = true;
                    }
                    ui.label(format!("http://127.0.0.1:{}", self.port));
                });
            });
        });

        // ---- Left sidebar: devices + processes ----
        egui::SidePanel::left("left_pane")
            .resizable(true)
            .default_width(270.0)
            .show(ctx, |ui| {
                ui.heading("Devices");
                if ui.button("⟳ Refresh devices").clicked() {
                    act_refresh_devices = true;
                }
                let mut clicked_device = None;
                egui::ScrollArea::vertical()
                    .id_source("devices")
                    .max_height(180.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, d) in self.devices.iter().enumerate() {
                            let label = format!("{} [{}]", d.name_or_id(), d.id);
                            if ui
                                .selectable_label(self.selected_device == Some(i), label)
                                .clicked()
                            {
                                clicked_device = Some(i);
                            }
                        }
                    });
                if let Some(i) = clicked_device {
                    self.selected_device = Some(i);
                    act_refresh_procs = true;
                }

                ui.separator();
                ui.heading("Processes");
                ui.horizontal(|ui| {
                    ui.label("filter:");
                    ui.text_edit_singleline(&mut self.process_filter);
                });
                let filter = self.process_filter.to_lowercase();
                let mut clicked_proc = None;
                egui::ScrollArea::vertical()
                    .id_source("procs")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, p) in self.processes.iter().enumerate() {
                            if !filter.is_empty() && !p.name.to_lowercase().contains(&filter) {
                                continue;
                            }
                            if ui
                                .selectable_label(
                                    self.selected_process == Some(i),
                                    format!("{:>6}  {}", p.pid, p.name),
                                )
                                .clicked()
                            {
                                clicked_proc = Some(i);
                            }
                        }
                    });
                if let Some(i) = clicked_proc {
                    self.selected_process = Some(i);
                    if let Some(p) = self.processes.get(i) {
                        self.target_input = p.name.clone();
                    }
                }
            });

        // ---- Right detail pane ----
        egui::SidePanel::right("right_pane")
            .resizable(true)
            .default_width(330.0)
            .show(ctx, |ui| {
                ui.heading("Detail");
                ui.separator();
                if let Some(d) = self.selected_device.and_then(|i| self.devices.get(i)) {
                    ui.label(format!("Device: {}", d.name_or_id()));
                    ui.label(format!("  id:   {}", d.id));
                    ui.label(format!("  kind: {:?}", d.kind));
                    ui.separator();
                }
                if let Some(p) = self.selected_process.and_then(|i| self.processes.get(i)) {
                    ui.label("Selected process:");
                    ui.label(format!("  pid:  {}", p.pid));
                    ui.label(format!("  name: {}", p.name));
                    ui.separator();
                }
                ui.label("Target:");
                ui.text_edit_singleline(&mut self.target_input);
                ui.separator();
                ui.label("Quick actions:");
                if ui.button("List modules").clicked() {
                    self.script =
                        "Process.enumerateModules().forEach(function(m){console.log(m.name+'  '+m.base+'  '+m.size);});".to_string();
                    act_run = true;
                }
                if ui.button("SSL bypass").clicked() {
                    if let Some(s) = crate::scripts::find("ssl-bypass") {
                        self.script = s.source.to_string();
                        act_run = true;
                    }
                }
                if ui.button("Root bypass").clicked() {
                    if let Some(s) = crate::scripts::find("root-bypass") {
                        self.script = s.source.to_string();
                        act_run = true;
                    }
                }
            });

        // ---- Center main view ----
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Processes, "Processes");
                ui.selectable_value(&mut self.tab, Tab::Editor, "Editor");
                ui.selectable_value(&mut self.tab, Tab::Console, "Console");
                ui.selectable_value(&mut self.tab, Tab::Memory, "Memory");
                ui.selectable_value(&mut self.tab, Tab::Classes, "Classes");
                ui.selectable_value(&mut self.tab, Tab::Ai, "AI");
                ui.selectable_value(&mut self.tab, Tab::Backends, "Backends");
            });
            ui.separator();

            match self.tab {
                Tab::Processes => {
                    ui.label("Processes for the selected device (select one in the left pane).");
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        egui::Grid::new("proc_grid").striped(true).num_columns(2).show(ui, |ui| {
                            ui.strong("PID");
                            ui.strong("Name");
                            ui.end_row();
                            for p in &self.processes {
                                ui.label(p.pid.to_string());
                                ui.label(&p.name);
                                ui.end_row();
                            }
                        });
                    });
                }
                Tab::Editor => {
                    ui.horizontal(|ui| {
                        if ui.button("▶ Run").clicked() {
                            act_run = true;
                        }
                        ui.label("Edits run as a one-shot agent script on the target.");
                    });
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.script)
                                .code_editor()
                                .desired_rows(24)
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
                Tab::Console => {
                    if ui.button("Clear").clicked() {
                        clear_console = true;
                    }
                    // Read-only view: edit a throwaway copy so it stays selectable/copyable.
                    let mut view = self.console.clone();
                    egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut view)
                                .code_editor()
                                .desired_rows(28)
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
                Tab::Memory => {
                    ui.horizontal(|ui| {
                        ui.label("Address:");
                        ui.text_edit_singleline(&mut self.mem_addr);
                        ui.label("Length:");
                        ui.add(egui::TextEdit::singleline(&mut self.mem_len).desired_width(60.0));
                        if ui.button("Read").clicked() {
                            act_read_mem = true;
                        }
                    });
                    ui.separator();
                    ui.label("Output appears in the Console tab.");
                }
                Tab::Classes => {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.classes_ios, "iOS (ObjC) — otherwise Android (Java)");
                        if ui.button("Enumerate classes").clicked() {
                            act_classes = true;
                        }
                    });
                    ui.separator();
                    ui.label("Output appears in the Console tab.");
                }
                Tab::Ai => {
                    ui.label("Describe what you want to hook; Claude generates a script.");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.ai_goal);
                        if ui.button("Generate").clicked() {
                            act_ai = true;
                        }
                    });
                    ui.separator();
                    let mut ai_view = self.ai_output.clone();
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut ai_view)
                                .code_editor()
                                .desired_rows(22)
                                .desired_width(f32::INFINITY),
                        );
                    });
                    ui.label("Requires ANTHROPIC_API_KEY.");
                }
                Tab::Backends => {
                    ui.label("frida-core backend taxonomy (instrumentation delegated to upstream):");
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        egui::Grid::new("backend_grid").striped(true).num_columns(2).show(ui, |ui| {
                            ui.strong("Backend");
                            ui.strong("Description");
                            ui.end_row();
                            for b in Backend::ALL {
                                ui.label(b.id());
                                ui.label(b.description());
                                ui.end_row();
                            }
                        });
                    });
                }
            }
        });

        // ---- Apply collected actions (self is freely mutable here) ----
        if clear_console {
            self.console.clear();
        }
        if let Some(dark) = set_dark {
            self.set_theme(ctx, dark);
        }
        if act_refresh_devices {
            self.refresh_devices(ctx);
        }
        if act_refresh_procs {
            self.refresh_processes(ctx);
        }
        if act_run {
            self.tab = Tab::Console;
            self.run_script(ctx);
        }
        if act_read_mem {
            self.read_memory(ctx);
        }
        if act_classes {
            self.list_classes(ctx);
        }
        if act_ai {
            self.ai_generate(ctx);
        }
    }
}

impl Device {
    fn name_or_id(&self) -> String {
        if self.name.is_empty() {
            self.id.clone()
        } else {
            self.name.clone()
        }
    }
}

fn default_script() -> String {
    "// One-shot agent script. Click Run (or ▶) to execute on the target.\n\
     console.log('[lancer] pid ' + Process.id + ' (' + Process.arch + ')');\n\
     Process.enumerateModules().slice(0, 20).forEach(function (m) {\n\
     \u{20}\u{20}console.log('  ' + m.name + '  @' + m.base);\n\
     });\n"
        .to_string()
}

fn load_config() -> Config {
    crate::core::config_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &Config) {
    if let Some(dir) = crate::core::config_dir() {
        let _ = std::fs::create_dir_all(&dir);
    }
    if let Some(p) = crate::core::config_file() {
        if let Ok(s) = serde_json::to_string_pretty(cfg) {
            let _ = std::fs::write(p, s);
        }
    }
}
