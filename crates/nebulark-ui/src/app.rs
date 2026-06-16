use eframe::egui;
use egui::{Color32, FontId, RichText, Stroke, Vec2};
use nebulark_core::profiles::ProfileManager;
use std::time::{Duration, Instant};

fn config_path() -> String {
    let home = std::env::var("SUDO_USER")
        .ok()
        .and_then(|user| {
            std::process::Command::new("getent")
                .args(["passwd", &user])
                .output()
                .ok()
                .and_then(|o| {
                    String::from_utf8(o.stdout).ok().and_then(|s| {
                        s.split(':')
                            .nth(5)
                            .map(|h| std::path::PathBuf::from(h.trim()))
                    })
                })
        })
        .or_else(|| std::env::var("HOME").ok().map(std::path::PathBuf::from))
        .unwrap_or_default();

    home.join(".config")
        .join("nebulark")
        .join("config.toml")
        .to_string_lossy()
        .to_string()
}

#[derive(PartialEq)]
enum ConnState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct NebularkApp {
    config_path: String,
    profiles: Vec<String>,
    selected: Option<usize>,
    state: ConnState,
    status_msg: String,
    last_check: Instant,
    import_path: String,
    import_name: String,
    show_import: bool,
}

impl NebularkApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let config_path = config_path();
        let profiles = load_profiles(&config_path);
        let connected = crate::daemon::is_connected();

        Self {
            config_path,
            profiles,
            selected: None,
            state: if connected {
                ConnState::Connected
            } else {
                ConnState::Disconnected
            },
            status_msg: String::new(),
            last_check: Instant::now(),
            import_path: String::new(),
            import_name: String::new(),
            show_import: false,
        }
    }
}

fn load_profiles(config_path: &str) -> Vec<String> {
    ProfileManager::load(config_path)
        .map(|m| m.profiles().iter().map(|p| p.name.clone()).collect())
        .unwrap_or_default()
}

impl eframe::App for NebularkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        if self.last_check.elapsed() > Duration::from_secs(2) {
            self.last_check = Instant::now();
            let connected = crate::daemon::is_connected();
            if connected && self.state == ConnState::Connecting {
                self.state = ConnState::Connected;
                self.status_msg = "Connected".into();
            }
            if !connected && self.state == ConnState::Connected {
                self.state = ConnState::Disconnected;
                self.status_msg = "Connection lost".into();
            }
        }

        if self.state == ConnState::Connecting {
            ctx.request_repaint_after(Duration::from_millis(200));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(18, 18, 22)))
            .show(ctx, |ui| {
                ui.add_space(24.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("NEBULARK")
                            .font(FontId::proportional(28.0))
                            .color(Color32::from_rgb(100, 200, 255))
                            .strong(),
                    );
                    ui.label(
                        RichText::new("AmneziaWG 2.0 client")
                            .font(FontId::proportional(11.0))
                            .color(Color32::from_rgb(80, 80, 100)),
                    );
                });

                ui.add_space(20.0);

                ui.vertical_centered(|ui| {
                    let (dot_color, state_text) = match self.state {
                        ConnState::Connected => (Color32::from_rgb(80, 220, 120), "Connected"),
                        ConnState::Connecting => (Color32::from_rgb(255, 200, 50), "Connecting..."),
                        ConnState::Disconnected => {
                            (Color32::from_rgb(100, 100, 120), "Disconnected")
                        }
                    };
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() / 2.0 - 50.0);
                        let (resp, painter) =
                            ui.allocate_painter(Vec2::splat(12.0), egui::Sense::hover());
                        painter.circle_filled(resp.rect.center(), 5.0, dot_color);
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(state_text)
                                .font(FontId::proportional(13.0))
                                .color(dot_color),
                        );
                    });
                });

                ui.add_space(24.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(
                    RichText::new("Profiles")
                        .font(FontId::proportional(12.0))
                        .color(Color32::from_rgb(120, 120, 140)),
                );
                ui.add_space(6.0);

                let available_w = ui.available_width();
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if self.profiles.is_empty() {
                            ui.label(
                                RichText::new("No profiles. Import a .conf file.")
                                    .color(Color32::from_rgb(80, 80, 100))
                                    .italics(),
                            );
                        }
                        for (i, name) in self.profiles.iter().enumerate() {
                            let selected = self.selected == Some(i);
                            let bg = if selected {
                                Color32::from_rgb(30, 60, 90)
                            } else {
                                Color32::from_rgb(24, 24, 30)
                            };

                            let resp = egui::Frame::none()
                                .fill(bg)
                                .stroke(Stroke::new(
                                    1.0,
                                    if selected {
                                        Color32::from_rgb(100, 180, 255)
                                    } else {
                                        Color32::from_rgb(40, 40, 55)
                                    },
                                ))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(available_w - 24.0);
                                    ui.label(
                                        RichText::new(name).font(FontId::proportional(13.0)).color(
                                            if selected {
                                                Color32::from_rgb(180, 220, 255)
                                            } else {
                                                Color32::from_rgb(200, 200, 210)
                                            },
                                        ),
                                    );
                                });

                            if resp.response.interact(egui::Sense::click()).clicked() {
                                self.selected = Some(i);
                            }
                            ui.add_space(4.0);
                        }
                    });

                ui.add_space(16.0);

                ui.vertical_centered(|ui| match self.state {
                    ConnState::Disconnected => {
                        let enabled = self.selected.is_some();
                        let btn = egui::Button::new(
                            RichText::new("  Connect  ")
                                .font(FontId::proportional(14.0))
                                .color(Color32::WHITE),
                        )
                        .fill(if enabled {
                            Color32::from_rgb(40, 120, 220)
                        } else {
                            Color32::from_rgb(40, 40, 60)
                        })
                        .rounding(8.0)
                        .min_size(Vec2::new(160.0, 38.0));

                        if ui.add_enabled(enabled, btn).clicked() {
                            if let Some(idx) = self.selected {
                                let profile = self.profiles[idx].clone();
                                let exe = std::env::current_exe().unwrap_or_default();
                                match crate::daemon::spawn_daemon(&exe, &self.config_path, &profile)
                                {
                                    Ok(_) => {
                                        self.state = ConnState::Connecting;
                                        self.status_msg = format!("Connecting to {profile}...");
                                    }
                                    Err(e) => {
                                        self.status_msg = format!("Error: {e}");
                                    }
                                }
                            }
                        }
                    }
                    ConnState::Connected => {
                        let btn = egui::Button::new(
                            RichText::new("  Disconnect  ")
                                .font(FontId::proportional(14.0))
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(180, 50, 50))
                        .rounding(8.0)
                        .min_size(Vec2::new(160.0, 38.0));

                        if ui.add(btn).clicked() {
                            let _ = crate::daemon::disconnect();
                            self.state = ConnState::Disconnected;
                            self.status_msg = "Disconnected".into();
                        }
                    }
                    _ => {
                        ui.add_enabled(
                            false,
                            egui::Button::new(
                                RichText::new("  Please wait...  ")
                                    .font(FontId::proportional(14.0)),
                            )
                            .min_size(Vec2::new(160.0, 38.0)),
                        );
                    }
                });

                if !self.status_msg.is_empty() {
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(&self.status_msg)
                                .font(FontId::proportional(11.0))
                                .color(Color32::from_rgb(120, 140, 160)),
                        );
                    });
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    if ui
                        .button(
                            RichText::new("+ Import .conf")
                                .font(FontId::proportional(12.0))
                                .color(Color32::from_rgb(100, 160, 220)),
                        )
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("AmneziaWG config", &["conf"])
                            .set_title("Select .conf file")
                            .pick_file()
                        {
                            self.import_path = path.to_string_lossy().to_string();
                            self.import_name = path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            self.show_import = true;
                        }
                    }

                    if self.selected.is_some() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(16.0);
                            if ui
                                .button(
                                    RichText::new("Delete")
                                        .font(FontId::proportional(12.0))
                                        .color(Color32::from_rgb(180, 80, 80)),
                                )
                                .clicked()
                            {
                                if let Some(idx) = self.selected {
                                    let name = self.profiles[idx].clone();
                                    if let Ok(mut mgr) = ProfileManager::load(&self.config_path) {
                                        match mgr.remove(&name) {
                                            Ok(_) => eprintln!("Remove ok"),
                                            Err(e) => eprintln!("Remove err: {e}"),
                                        }
                                    }
                                    self.profiles = load_profiles(&self.config_path);
                                    self.selected = None;
                                    self.status_msg = format!("Deleted '{name}'");
                                }
                            }
                        });
                    }
                });

                if self.show_import {
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(Color32::from_rgb(22, 22, 30))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 70)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Path to .conf file:")
                                    .font(FontId::proportional(11.0))
                                    .color(Color32::from_rgb(120, 120, 140)),
                            );
                            ui.text_edit_singleline(&mut self.import_path);
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Profile name:")
                                    .font(FontId::proportional(11.0))
                                    .color(Color32::from_rgb(120, 120, 140)),
                            );
                            ui.text_edit_singleline(&mut self.import_name);
                            ui.add_space(8.0);

                            if ui.button("Import").clicked() {
                                let path = self.import_path.trim().to_string();
                                let name = if self.import_name.trim().is_empty() {
                                    std::path::Path::new(&path)
                                        .file_stem()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string()
                                } else {
                                    self.import_name.trim().to_string()
                                };

                                match do_import(&self.config_path, &path, &name) {
                                    Ok(_) => {
                                        self.profiles = load_profiles(&self.config_path);
                                        self.status_msg = format!("Imported '{name}'");
                                        self.import_path.clear();
                                        self.import_name.clear();
                                        self.show_import = false;
                                    }
                                    Err(e) => {
                                        self.status_msg = format!("Import error: {e}");
                                    }
                                }
                            }
                        });
                }
            });
    }
}

fn do_import(config_path: &str, path: &str, name: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path).or_else(|_| {
        let out = std::process::Command::new("sudo")
            .args(["cat", path])
            .output()?;
        if out.status.success() {
            Ok(String::from_utf8(out.stdout)?)
        } else {
            anyhow::bail!("Cannot read file")
        }
    })?;

    let tunnel = nebulark_awg::parser::parse_conf(&content)?;
    let profile = nebulark_common::config::Profile {
        name: name.to_string(),
        tunnel,
    };
    let mut mgr = ProfileManager::load(config_path)?;
    mgr.add(profile)?;
    Ok(())
}
