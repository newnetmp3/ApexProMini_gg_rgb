//! Native desktop control center (egui/eframe), runs on X11 and Wayland.
//! No direct HID writes or keyboard event reads: systemd owns the sole RGB
//! writer. Profile editing is local and application-safe by default.
use apex_pro_mini::effects::{KeyHit, Preset};
use apex_pro_mini::effects;
use apex_pro_mini::profile::{color_hex, default_profile_path, Profile};
use apex_pro_mini::Rgb;
use apex_pro_mini::factory;
use apex_pro_mini::layers::Stack;
use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Vec2};
use std::process::Command;
use std::time::Instant;

const SERVICE: &str = "apex-mini-ripple.service";
const EFFECTS: [Preset; 14] = [
    Preset::Solid, Preset::Gradient, Preset::Rainbow, Preset::Wave,
    Preset::Breathing, Preset::ColorShift, Preset::Sparkle,
    Preset::Ripple, Preset::Reactive, Preset::Comet,
    Preset::RainbowSplit, Preset::PrismaticSweep, Preset::Heatmap,
    Preset::ReactiveLine,
];
// Historical locale IDs extracted from GG 108.3 bundled Prism migrations.
// NOT assertions that these exact effects are implemented or currently visible
// in SteelSeries GG.
const GG_IDS: &[&str] = &[
    "aqua", "chasingGhosts", "clown", "colorFusion", "comet", "disabled",
    "discoMode", "drain", "faze", "freeway", "haze", "heatmap", "pride",
    "prism", "radioactive", "rainbow", "rainbowSplit", "reactiveLine",
    "reactiveRipple", "reactiveSingleKey", "redPulse", "selfDestruct",
    "shavedIce", "solar", "staticFade", "steelseriesOrange", "twilight",
    "vaporDreams", "wabashAndLake", "warpDrive", "westCoast",
];
fn rgb_array(c: Rgb) -> [u8; 3] {
    [c.red, c.green, c.blue]
}
fn as_rgb(c: [u8; 3]) -> Rgb {
    Rgb { red: c[0], green: c[1], blue: c[2] }
}
fn ui_color(c: Rgb) -> Color32 {
    Color32::from_rgb(c.red, c.green, c.blue)
}
fn label(hid: u8) -> String {
    match hid {
        4..=29 => ((b'A' + hid - 4) as char).to_string(),
        30..=38 => (hid - 29).to_string(),
        39 => "0".into(),
        40 => "Enter".into(),
        41 => "Esc".into(),
        42 => "Bksp".into(),
        43 => "Tab".into(),
        44 => "Space".into(),
        225 | 229 => "Shift".into(),
        224 | 228 => "Ctrl".into(),
        226 | 230 => "Alt".into(),
        227 | 231 => "Meta".into(),
        57 => "Caps".into(),
        _ => format!("{hid}"),
    }
}
fn command(args: &[&str]) -> Result<String, String> {
    let result = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .output()
        .map_err(|err| format!("Cannot launch systemctl --user: {err}"))?;
    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("systemctl --user {} failed: {}", args.join(" "), stderr.trim()))
    }
}
fn compatible_unit() -> Result<(), String> {
    let unit = command(&["cat", SERVICE])?;
    if !unit.contains("--profile %h/.config/apex-mini-rgb/profile.conf") {
        return Err("Installed service uses the old command. Update it once using:\nbash scripts/install-user-service.sh".into());
    }
    Ok(())
}

struct ControlCenter {
    profile: Profile,
    feedback: String,
    selected_key: Option<u8>,
    preview_hit: Option<KeyHit>,
    clock: Instant,
    active: bool,
    show_catalog: bool,
    dirty: bool,
}

impl ControlCenter {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let (profile, feedback) = match default_profile_path()
            .and_then(|path| Profile::load(&path)) {
            Ok(profile) => (profile, "Loaded your saved lighting profile".into()),
            Err(_) => (Profile::default(), "New profile. Save to keep your settings.".into()),
        };
        let active = command(&["is-active", SERVICE])
            .is_ok_and(|output| output == "active");
        Self {
            profile, feedback, selected_key: Some(4),
            preview_hit: None, clock: Instant::now(), active,
            show_catalog: false, dirty: false,
        }
    }

    fn save_profile(&mut self) -> Result<(), String> {
        self.profile.validate()?;
        let path = default_profile_path()?;
        self.profile.save(&path)?;
        self.dirty = false;
        Ok(())
    }

    fn action(&mut self, instruction: &str) {
        self.feedback = match instruction {
            "save" => self.save_profile().map(|_| "Profile saved locally. The running keyboard effect has not been changed.".to_string()),
            "apply" => {
                compatible_unit()
                    .and_then(|_| self.save_profile())
                    .and_then(|_| command(&["restart", SERVICE]))
                    .map(|_| "Profile applied. The existing user service restarted; it remains the only RGB writer.".into())
            }
            "start" => compatible_unit()
                .and_then(|_| self.save_profile())
                .and_then(|_| command(&["enable", "--now", SERVICE]))
                .map(|_| "Background lighting service started and enabled for user login.".into()),
            "stop" => command(&["stop", SERVICE])
                .map(|_| "Service stopped; normal onboard lighting should be restored.".into()),
            "disable" => command(&["disable", "--now", SERVICE])
                .map(|_| "Service disabled and stopped; onboard lighting should be restored.".into()),
            _ => Err("Unknown GUI action".into()),
        }.unwrap_or_else(|err| format!("Action unsuccessful: {err}"));
        self.active = command(&["is-active", SERVICE])
            .is_ok_and(|value| value == "active");
    }

    fn colors(&self) -> Vec<apex_pro_mini::KeyColor> {
        let seconds = self.clock.elapsed().as_secs_f32();
        let hits = self.preview_hit.iter().copied().filter(|hit| seconds - hit.when < 2.5).collect::<Vec<_>>();
        let mut colors = Stack {
            base: self.profile.preset,
            idle: self.profile.idle,
            reactive: self.profile.reactive,
            idle_after_seconds: self.profile.idle_after_seconds,
        }.render(self.profile.config, seconds,
            self.preview_hit.map(|hit| hit.when).unwrap_or(0.0),
            &hits);
        for item in &mut colors {
            if let Some(color) = self.profile.key_overrides.get(&item.hid) {
                item.rgb = *color;
            }
        }
        colors
    }

    fn draw_keyboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Keyboard lighting preview");
        ui.label("Preview is simulated locally — this is not a live LED readback. Click a key to paint an override.");
        ui.add_space(10.0);
        let canvas_width = ui.available_width().max(300.0);
        let height = (canvas_width * 0.37).clamp(220.0, 365.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(canvas_width, height), Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, egui::CornerRadius::same(12), Color32::from_rgb(21, 29, 44));
        let left = rect.left() + 14.0;
        let top = rect.top() + 19.0;
        let step_x = (rect.width() - 35.0) / 15.3;
        let step_y = (rect.height() - 38.0) / 5.0;
        let now = self.colors();
        for item in &now {
            let Some((x, y)) = effects::position(item.hid) else { continue };
            let x_pos = left + x * 14.0 * step_x;
            let y_pos = top + y * 4.0 * step_y;
            let key_width = if item.hid == 44 { step_x * 2.15 } else { step_x * 0.88 };
            let key = Rect::from_min_size(
                Pos2::new(x_pos, y_pos),
                Vec2::new(key_width, step_y * 0.74),
            );
            let selected = self.selected_key == Some(item.hid);
            painter.rect_filled(key, egui::CornerRadius::same(5), ui_color(item.rgb));
            if selected {
                painter.rect_stroke(key, egui::CornerRadius::same(5),
                    egui::Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Outside);
            }
            let text = if u16::from(item.rgb.red) + u16::from(item.rgb.green) + u16::from(item.rgb.blue) > 370 {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            painter.text(key.center(), egui::Align2::CENTER_CENTER,
                label(item.hid), FontId::proportional(if item.hid == 44 { 11.0 } else { 10.0 }), text);
            if response.clicked() && response.interact_pointer_pos().is_some_and(|pos| key.contains(pos)) {
                self.selected_key = Some(item.hid);
            }
        }
        if self.profile.preset.needs_key_events() {
            ui.horizontal(|ui| {
                if ui.button("Simulate ripple from selected key").clicked() {
                    self.preview_hit = self.selected_key.map(|hid| KeyHit {
                        hid, when: self.clock.elapsed().as_secs_f32(),
                    });
                }
                ui.label("Simulation uses no keyboard input.");
            });
        }
        if let Some(hid) = self.selected_key {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Selected: {} (HID {})", label(hid), hid));
                let initial = self.profile.key_overrides.get(&hid)
                    .copied().unwrap_or(self.profile.config.base);
                let mut color = rgb_array(initial);
                if ui.color_edit_button_srgb(&mut color).changed() {
                    self.profile.key_overrides.insert(hid, as_rgb(color));
                    self.dirty = true;
                }
                if ui.button("Clear key override").clicked() {
                    self.profile.key_overrides.remove(&hid);
                    self.dirty = true;
                }
                ui.label(format!("Override: {}", self.profile.key_overrides.get(&hid)
                    .map(|rgb| format!("#{}", color_hex(*rgb))).unwrap_or("None".into())));
            });
        }
    }
}

impl eframe::App for ControlCenter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(40));
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.heading("APEX MINI • PRISM");
                ui.separator();
                ui.label("Linux lighting control center");
                ui.separator();
                ui.colored_label(if self.active { Color32::LIGHT_GREEN } else { Color32::LIGHT_YELLOW },
                    if self.active { "● Daemon active" } else { "○ Daemon stopped" });
                if self.dirty { ui.colored_label(Color32::YELLOW, "• Unsaved edits"); }
            });
            ui.add_space(8.0);
        });
        egui::SidePanel::left("editor").resizable(true).default_width(258.0).show(ctx, |ui| {
            ui.heading("Lighting");
            ui.label("Effect");
            egui::ComboBox::from_id_salt("effect")
                .selected_text(self.profile.preset.as_str())
                .width(210.0)
                .show_ui(ui, |ui| {
                    for preset in EFFECTS {
                        if ui.selectable_value(&mut self.profile.preset, preset, preset.as_str()).changed() {
                            self.dirty = true;
                        }
                    }
                });
            ui.separator();
            ui.label("Primary color");
            let mut base = rgb_array(self.profile.config.base);
            if ui.color_edit_button_srgb(&mut base).changed() {
                self.profile.config.base = as_rgb(base);
                self.dirty = true;
            }
            ui.label("Accent / reactive color");
            let mut accent = rgb_array(self.profile.config.accent);
            if ui.color_edit_button_srgb(&mut accent).changed() {
                self.profile.config.accent = as_rgb(accent);
                self.dirty = true;
            }
            self.dirty |= ui.add(egui::Slider::new(&mut self.profile.config.speed, 0.1..=5.0).text("Speed")).changed();
            self.dirty |= ui.add(egui::Slider::new(&mut self.profile.config.brightness, 0.0..=1.0).text("Brightness")).changed();
            self.dirty |= ui.add(egui::Slider::new(&mut self.profile.fps, 5..=40).text("FPS")).changed();
            ui.separator();
            ui.collapsing("SteelSeries GG factory-style previews (31)", |ui| {
                ui.small("Recreated palettes/timing; compositor, regions and fidelity remain unverified.");
                egui::ScrollArea::vertical().max_height(245.0).show(ui, |ui| {
                    for (index, spec) in factory::PRESETS.iter().enumerate() {
                        if ui.selectable_value(
                            &mut self.profile.preset,
                            Preset::Factory(index as u8),
                            format!("{} · {} ms", spec.id, spec.period_ms),
                        ).changed() {
                            self.dirty = true;
                        }
                    }
                });
            });
            ui.separator();
            ui.collapsing("Base + Idle + Reactive layers", |ui| {
                ui.small("Optional GG-style idle timer and reactive overlay; preview is simulated.");
                let current_idle = self.profile.idle.map(Preset::as_str).unwrap_or("none");
                egui::ComboBox::from_id_salt("idle_mode")
                    .selected_text(format!("Idle: {current_idle}"))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.profile.idle, None, "None").changed() {
                            self.dirty = true;
                        }
                        for preset in EFFECTS {
                            if !preset.needs_key_events()
                                && ui.selectable_value(&mut self.profile.idle, Some(preset), preset.as_str()).changed() {
                                self.dirty = true;
                            }
                        }
                        for (i, spec) in factory::PRESETS.iter().enumerate() {
                            let mode = Preset::Factory(i as u8);
                            if !mode.needs_key_events()
                                && ui.selectable_value(&mut self.profile.idle, Some(mode), spec.cli).changed() {
                                self.dirty = true;
                            }
                        }
                    });
                self.dirty |= ui.add(egui::Slider::new(
                    &mut self.profile.idle_after_seconds, 1.0..=120.0,
                ).text("Idle after (seconds)")).changed();
                let current_reactive = self.profile.reactive.map(Preset::as_str).unwrap_or("none");
                egui::ComboBox::from_id_salt("reactive_overlay")
                    .selected_text(format!("Reactive: {current_reactive}"))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.profile.reactive, None, "None").changed() {
                            self.dirty = true;
                        }
                        for preset in EFFECTS {
                            if preset.needs_key_events()
                                && ui.selectable_value(&mut self.profile.reactive, Some(preset), preset.as_str()).changed() {
                                self.dirty = true;
                            }
                        }
                        for (i, spec) in factory::PRESETS.iter().enumerate() {
                            let mode = Preset::Factory(i as u8);
                            if mode.needs_key_events()
                                && ui.selectable_value(&mut self.profile.reactive, Some(mode), spec.cli).changed() {
                                self.dirty = true;
                            }
                        }
                    });
            });
            ui.separator();
            ui.heading("Service controls");
            if ui.button("Save profile (no device change)").clicked() { self.action("save"); }
            if ui.button("Apply & restart lighting").clicked() { self.action("apply"); }
            if ui.button("Enable at login & start").clicked() { self.action("start"); }
            if ui.button("Stop lighting").clicked() { self.action("stop"); }
            if ui.button("Disable at login & stop").clicked() { self.action("disable"); }
            ui.add_space(7.0);
            ui.small("One writer: the systemd user service owns the HID device. The GUI never sends RGB packets.");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_keyboard(ui);
            ui.separator();
            ui.label(self.feedback.as_str());
            ui.add_space(8.0);
            ui.checkbox(&mut self.show_catalog, "Show 31 GG factory-style preview entries and fidelity status");
            if self.show_catalog {
                ui.label("These historical entries are now selectable as parameter-derived previews. None is certified as faithful GG parity.");
                egui::ScrollArea::vertical().max_height(185.0).show(ui, |ui| {
                    egui::Grid::new("catalog").striped(true).show(ui, |ui| {
                        for id in GG_IDS {
                            ui.monospace(*id);
                            ui.label("Parameter-derived preview · fidelity unverified");
                            ui.end_row();
                        }
                    });
                });
            }
            ui.add_space(6.0);
            ui.small("Wired SteelSeries Apex Pro Mini (1038:161e) • native Rust • not affiliated with SteelSeries");
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([850.0, 620.0])
            .with_title("Apex Mini Prism — Linux"),
        ..Default::default()
    };
    eframe::run_native("Apex Mini Prism", options, Box::new(|cc| Ok(Box::new(ControlCenter::new(cc)))))
}
