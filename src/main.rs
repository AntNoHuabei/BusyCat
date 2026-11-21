#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod input;
mod interaction;
mod keyboard_layout;
mod phone_expression;

use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use interaction::{Interaction, key_rect};
use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

const ATLAS: &[u8] = include_bytes!("../assets/cat/atlas.png");
const COLS: usize = 6;

struct Cat {
    texture: egui::TextureHandle,
    mouse_texture: egui::TextureHandle,
    phone_faces: Vec<egui::TextureHandle>,
    phone_expression: phone_expression::PhoneExpression,
    input: Receiver<input::Input>,
    activity: Interaction,
    status: String,
    start: Instant,
    settings: bool,
    passthrough: bool,
    demo: bool,
    demo_at: Instant,
    demo_step: usize,
    opacity: f32,
    highlights: bool,
    mouse_feedback: bool,
    gaze: Vec2,
    mouse_travel: Vec2,
}

impl Cat {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let image = image::load_from_memory(ATLAS)
            .expect("embedded cat atlas")
            .to_rgba8();
        let texture = cc.egui_ctx.load_texture(
            "busycat",
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            ),
            egui::TextureOptions::LINEAR,
        );
        let mouse = image::load_from_memory(include_bytes!("../assets/cat/mouse.png"))
            .expect("mouse layer")
            .to_rgba8();
        let mouse_texture = cc.egui_ctx.load_texture(
            "mouse",
            egui::ColorImage::from_rgba_unmultiplied([512, 512], mouse.as_raw()),
            egui::TextureOptions::LINEAR,
        );
        let phone_faces = [
            &include_bytes!("../assets/cat/phone_happy_face.png")[..],
            &include_bytes!("../assets/cat/phone_surprise_face.png")[..],
            &include_bytes!("../assets/cat/phone_unimpressed_face.png")[..],
            &include_bytes!("../assets/cat/phone_sleepy_face.png")[..],
            &include_bytes!("../assets/cat/phone_blush_face.png")[..],
        ]
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            let img = image::load_from_memory(bytes)
                .expect("phone expression")
                .to_rgba8();
            cc.egui_ctx.load_texture(
                format!("phone-face-{i}"),
                egui::ColorImage::from_rgba_unmultiplied([512, 512], img.as_raw()),
                egui::TextureOptions::LINEAR,
            )
        })
        .collect();
        Self {
            phone_faces,
            phone_expression: phone_expression::PhoneExpression::default(),
            texture,
            mouse_texture,
            input: input::listen(Some(cc.egui_ctx.clone())),
            activity: Interaction::default(),
            status: "Connecting input…".into(),
            start: Instant::now(),
            settings: false,
            passthrough: false,
            demo: std::env::args().any(|a| a == "--demo"),
            demo_at: Instant::now(),
            demo_step: 0,
            opacity: 1.0,
            highlights: true,
            mouse_feedback: true,
            gaze: Vec2::ZERO,
            mouse_travel: Vec2::ZERO,
        }
    }
    fn events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.input.try_recv() {
            if let input::Input::Status(s) = event {
                self.status = s;
                continue;
            }
            if !self.mouse_feedback
                && matches!(
                    event,
                    input::Input::Button { .. }
                        | input::Input::Move { .. }
                        | input::Input::Wheel { .. }
                )
            {
                self.activity.last_activity = Instant::now();
                continue;
            }
            if self.activity.handle(&event, Instant::now()) {
                self.passthrough = !self.passthrough;
                if self.passthrough {
                    self.settings = false;
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(self.passthrough));
            }
        }
    }
    fn demo_tick(&mut self, now: Instant) {
        if !self.demo || now.duration_since(self.demo_at) < Duration::from_millis(280) {
            return;
        }
        self.demo_at = now;
        let events = match self.demo_step % 12 {
            0..=5 => {
                let vk = [0x41, 0x53, 0x44, 0x4a, 0x4b, 0x4c][self.demo_step % 12];
                vec![
                    input::Input::Key { vk, down: true },
                    input::Input::Key { vk, down: false },
                ]
            }
            6 => vec![input::Input::Move { x: 0.15, y: 0.35 }],
            7 => vec![input::Input::Move { x: 0.85, y: 0.65 }],
            8 => vec![input::Input::Button {
                button: 0,
                down: true,
            }],
            9 => vec![input::Input::Button {
                button: 0,
                down: false,
            }],
            10 => vec![input::Input::Wheel {
                delta: 120,
                horizontal: false,
            }],
            _ => vec![input::Input::Wheel {
                delta: -120,
                horizontal: false,
            }],
        };
        for e in events {
            self.activity.handle(&e, now);
        }
        self.demo_step = self.demo_step.wrapping_add(1);
    }
    fn overlay(&mut self, ui: &egui::Ui, rect: Rect, frame: usize, now: Instant) {
        let scale = rect.width() / 512.0;
        let pos = |x: f32, y: f32| rect.min + Vec2::new(x, y) * scale;
        let fade = |c: Color32| c.gamma_multiply(self.opacity);
        // Redraw irises and pupils so gaze follows the global mouse, including outside this window.
        let target = if frame >= 12 {
            Vec2::new(0.0, 6.0)
        } else {
            Vec2::new(
                (self.activity.cursor[0] - 0.5) * 18.0,
                (self.activity.cursor[1] - 0.5) * 9.0,
            )
        };
        self.gaze += (target - self.gaze) * 0.22;
        let openness = match frame {
            2 => 0.45,
            3 => 0.0,
            5 | 7 | 9 => 0.88,
            13 => 0.85,
            14 => 0.3,
            _ => 1.0,
        };
        if openness > 0.0 {
            for x in [211.0, 300.0] {
                ui.painter().add(egui::Shape::ellipse_filled(
                    pos(x, 211.0),
                    Vec2::new(29.0, 31.0 * openness) * scale,
                    fade(Color32::from_rgb(199, 231, 130)),
                ));
                let pupil = pos(x + 4.0 + self.gaze.x, 214.0 + self.gaze.y * openness);
                ui.painter().add(egui::Shape::ellipse_filled(
                    pupil,
                    Vec2::new(10.0, 22.0 * openness) * scale,
                    fade(Color32::from_rgb(21, 32, 43)),
                ));
                ui.painter().add(egui::Shape::ellipse_filled(
                    pupil + Vec2::new(4.0, -11.0 * openness) * scale,
                    Vec2::new(4.0, 5.0 * openness) * scale,
                    fade(Color32::from_rgb(244, 255, 223)),
                ));
            }
        }
        if let Some((index, alpha)) = self.phone_expression.sample(frame >= 12, now) {
            ui.painter().image(
                self.phone_faces[index].id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE.gamma_multiply(self.opacity * alpha),
            );
        }
        if frame == 10 || frame == 11 {
            self.mouse_arm(ui, rect, now);
        }
        if self.highlights && frame < 12 {
            for vk in 0..=255 {
                if self.activity.active_key(vk, now)
                    && let Some([x, y, w, h]) = key_rect(vk)
                {
                    let key = Rect::from_min_size(pos(x, y), Vec2::new(w, h) * scale);
                    ui.painter().rect_filled(
                        key,
                        2.0,
                        fade(Color32::from_rgba_unmultiplied(241, 176, 58, 220)),
                    );
                }
            }
        }
        if frame < 12 {
            self.keyboard_paws(ui, rect, frame, now);
        }
        for b in &self.activity.bubbles {
            let t = now.duration_since(b.start).as_secs_f32() / 1.25;
            let x = if b.mouse {
                0.10 + self.mouse_travel.x / 512.0 + (b.slot % 3) as f32 * 0.055
            } else {
                0.24 + b.slot as f32 * 0.075
            };
            let p = rect.min
                + Vec2::new(
                    rect.width() * (x + (t * 5.0).sin() * 0.025),
                    rect.height() * (0.66 - t * 0.31),
                );
            let a = (((1.0 - t).clamp(0.0, 1.0).sqrt() * 255.0) * self.opacity) as u8;
            let font = egui::FontId::proportional((rect.width() * 0.043).clamp(12.0, 23.0));
            let galley = ui.painter().layout_no_wrap(
                b.text.clone(),
                font,
                Color32::from_rgba_unmultiplied(32, 39, 51, a),
            );
            let mut box_rect = Rect::from_center_size(p, galley.size() + Vec2::new(16.0, 10.0));
            // Keep long combinations inside the transparent viewport.
            box_rect = box_rect.translate(Vec2::new(
                (rect.left() - box_rect.left()).max(0.0)
                    - (box_rect.right() - rect.right()).max(0.0),
                0.0,
            ));
            ui.painter().rect_filled(
                box_rect.translate(Vec2::new(0.0, 2.0)),
                5.0,
                Color32::from_rgba_unmultiplied(18, 26, 35, a / 5),
            );
            ui.painter().rect_filled(
                box_rect,
                5.0,
                Color32::from_rgba_unmultiplied(245, 240, 223, a),
            );
            ui.painter()
                .galley(box_rect.min + Vec2::new(8.0, 5.0), galley, Color32::WHITE);
        }
    }
    fn keyboard_paws(&self, ui: &egui::Ui, rect: Rect, frame: usize, now: Instant) {
        let scale = rect.width() / 512.0;
        let p = |x: f32, y: f32| rect.min + Vec2::new(x, y) * scale;
        for viewer_left in [true, false] {
            if viewer_left && (frame == 10 || frame == 11) {
                continue;
            }
            let shoulder = if viewer_left { 195.0 } else { 315.0 };
            let rest = if viewer_left { 211.0 } else { 301.0 };
            let contact = self.activity.paw_key(viewer_left, now);
            let (x, y, depth) = contact.map_or((rest, 354.0, 0.0), |([x, y], d)| {
                (x, y - 10.0 - (1.0 - d) * 18.0, d)
            });
            let points = [
                p(shoulder, 301.0),
                p(shoulder, 340.0),
                p(x, y - 34.0),
                p(x, y - 10.0),
            ];
            for (w, c) in [
                (34.0, Color32::from_rgb(20, 29, 40)),
                (29.0, Color32::from_rgb(37, 46, 58)),
            ] {
                ui.painter()
                    .add(egui::epaint::CubicBezierShape::from_points_stroke(
                        points,
                        false,
                        Color32::TRANSPARENT,
                        egui::Stroke::new(w * scale, c.gamma_multiply(self.opacity)),
                    ));
            }
            if contact.is_some() {
                ui.painter().add(egui::Shape::ellipse_filled(
                    p(x, y + 11.0),
                    Vec2::new(10.0, 2.5) * scale,
                    Color32::from_black_alpha((depth * 85.0) as u8).gamma_multiply(self.opacity),
                ));
            }
            // The central fingertip ends exactly at the shared keycap center when down.
            for (dx, dy) in [(-8.0, -2.0), (0.0, 0.0), (8.0, -2.0)] {
                ui.painter().add(egui::Shape::ellipse_filled(
                    p(x + dx, y + dy),
                    Vec2::new(5.5, 10.0) * scale,
                    Color32::from_rgb(20, 29, 40).gamma_multiply(self.opacity),
                ));
                ui.painter().add(egui::Shape::ellipse_filled(
                    p(x + dx, y + dy - 1.0),
                    Vec2::new(4.0, 9.0) * scale,
                    Color32::from_rgb(37, 46, 58).gamma_multiply(self.opacity),
                ));
                ui.painter().line_segment(
                    [p(x + dx, y + dy + 5.0), p(x + dx, y + dy + 8.0)],
                    egui::Stroke::new(
                        1.2 * scale,
                        Color32::from_rgb(82, 96, 108).gamma_multiply(self.opacity),
                    ),
                );
            }
        }
    }
    fn mouse_buttons(&self, ui: &egui::Ui, rect: Rect, now: Instant) {
        let scale = rect.width() / 512.0;
        for button in [0, 1] {
            let d = self.activity.button_depth(button, now);
            // Viewer-right is the cat's left mouse button. Curved edges follow the shell.
            let outline = [
                (49.5, 411.5),
                (68.0, 411.5),
                (69.0, 422.0),
                (68.0, 430.0),
                (64.0, 436.0),
                (57.0, 439.5),
                (49.5, 440.5),
            ];
            let points = |dy: f32| {
                outline
                    .iter()
                    .map(|&(x, y)| {
                        let x = if button == 0 { x } else { 96.0 - x };
                        rect.min + (Vec2::new(x, y + dy) + self.mouse_travel) * scale
                    })
                    .collect::<Vec<_>>()
            };
            ui.painter().add(egui::Shape::convex_polygon(
                points(1.0),
                Color32::from_rgb(110, 119, 116).gamma_multiply(self.opacity),
                egui::Stroke::NONE,
            ));
            let shade = (239.0 - d * 17.0) as u8;
            ui.painter().add(egui::Shape::convex_polygon(
                points(-1.2 + d * 2.4),
                Color32::from_rgb(shade, shade - 5, shade - 18).gamma_multiply(self.opacity),
                egui::Stroke::new(
                    0.8 * scale,
                    Color32::from_rgb(161, 170, 163).gamma_multiply(self.opacity),
                ),
            ));
        }
    }
    fn mouse_arm(&self, ui: &egui::Ui, rect: Rect, now: Instant) {
        let scale = rect.width() / 512.0;
        let p = |x: f32, y: f32| rect.min + Vec2::new(x, y) * scale;
        let left = self.activity.button_depth(0, now);
        let right = self.activity.button_depth(1, now);
        let end = Vec2::new(48.0 + (left - right) * 1.1, 408.0 + left.max(right) * 1.6)
            + self.mouse_travel;
        let points = [
            p(195.0, 306.0),
            p(172.0, 345.0),
            p(end.x + 46.0, end.y - 13.0),
            p(end.x, end.y),
        ];
        for (width, color) in [
            (46.0, Color32::from_rgb(20, 29, 40)),
            (40.0, Color32::from_rgb(37, 46, 58)),
        ] {
            ui.painter()
                .add(egui::epaint::CubicBezierShape::from_points_stroke(
                    points,
                    false,
                    Color32::TRANSPARENT,
                    egui::Stroke::new(width * scale, color.gamma_multiply(self.opacity)),
                ));
            ui.painter().circle_filled(
                p(end.x, end.y),
                width * scale * 0.5,
                color.gamma_multiply(self.opacity),
            );
        }
        // Two rounded toes contact the corresponding button; the wrist stays anchored.
        for (dx, depth) in [(-10.0, right), (10.0, left)] {
            let toe = Vec2::new(48.0 + dx, 425.0 + depth * 3.4) + self.mouse_travel;
            ui.painter().add(egui::Shape::ellipse_filled(
                p(toe.x, toe.y + 2.5),
                Vec2::new(9.0, 12.0 - depth) * scale,
                Color32::from_black_alpha(65).gamma_multiply(self.opacity),
            ));
            ui.painter().add(egui::Shape::ellipse_filled(
                p(toe.x, toe.y),
                Vec2::new(8.0 + depth * 0.5, 11.0 - depth) * scale,
                Color32::from_rgb(20, 29, 40).gamma_multiply(self.opacity),
            ));
            ui.painter().add(egui::Shape::ellipse_filled(
                p(toe.x, toe.y - 1.0),
                Vec2::new(6.5 + depth * 0.5, 9.0 - depth) * scale,
                Color32::from_rgb(37, 46, 58).gamma_multiply(self.opacity),
            ));
            ui.painter().line_segment(
                [p(toe.x - 1.0, toe.y + 5.0), p(toe.x - 1.5, toe.y + 8.0)],
                egui::Stroke::new(
                    1.4 * scale,
                    Color32::from_rgb(82, 96, 108).gamma_multiply(self.opacity),
                ),
            );
        }
    }
}

impl eframe::App for Cat {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.events(ctx);
        let now = Instant::now();
        self.demo_tick(now);
        let offset = interaction::mouse_offset(self.activity.cursor);
        let target = if self.mouse_feedback {
            Vec2::from(offset)
        } else {
            Vec2::ZERO
        };
        self.mouse_travel += (target - self.mouse_travel) * 0.25;
        self.activity
            .bubbles
            .retain(|b| now.duration_since(b.start).as_secs_f32() < 1.25);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let area = ui.max_rect();
                let side = area.width().min(area.height());
                let rect = Rect::from_center_size(area.center(), Vec2::splat(side));
                let i = self
                    .activity
                    .frame(now, now.duration_since(self.start).as_millis());
                let uv = Rect::from_min_max(
                    Pos2::new((i % COLS) as f32 / COLS as f32, (i / COLS) as f32 / 3.0),
                    Pos2::new(
                        (i % COLS + 1) as f32 / COLS as f32,
                        (i / COLS + 1) as f32 / 3.0,
                    ),
                );
                ui.painter().image(
                    self.mouse_texture.id(),
                    rect.translate(self.mouse_travel * (side / 512.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE.gamma_multiply(self.opacity),
                );
                self.mouse_buttons(ui, rect, now);
                ui.painter().image(
                    self.texture.id(),
                    rect,
                    uv,
                    Color32::WHITE.gamma_multiply(self.opacity),
                );
                self.overlay(ui, rect, i, now);
                let response =
                    ui.interact(rect, ui.id().with("cat"), egui::Sense::click_and_drag());
                if response.drag_started_by(egui::PointerButton::Primary) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if response.double_clicked() {
                    self.settings = !self.settings;
                }
                response.context_menu(|ui| {
                    if ui.button("Settings").clicked() {
                        self.settings = true;
                        ui.close();
                    }
                    ui.checkbox(&mut self.activity.labels, "Show pressed keys");
                    if ui.button("Click through (F8 to restore)").clicked() {
                        self.passthrough = true;
                        self.settings = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
                        ui.close();
                    }
                    if ui.button("Quit BusyCat").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        if self.settings {
            let mut open = true;
            egui::Window::new("BusyCat · Input & animation")
                .open(&mut open)
                .default_pos([8.0, 8.0])
                .default_width(300.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.small(&self.status);
                    ui.separator();
                    if ui
                        .checkbox(&mut self.activity.labels, "Floating key / mouse labels")
                        .changed()
                        && !self.activity.labels
                    {
                        self.activity.bubbles.clear();
                    }
                    ui.checkbox(&mut self.highlights, "Highlight pressed keyboard keys");
                    if ui
                        .checkbox(&mut self.mouse_feedback, "Mouse movement, clicks and wheel")
                        .changed()
                    {
                        self.activity.buttons.clear();
                        self.activity.mouse_at = None;
                        self.activity.wheel_at = None;
                        self.activity.mouse_press = None;
                        self.activity.button_down_at = [None; 5];
                        self.activity.button_up_at = [None; 5];
                    }
                    if ui
                        .checkbox(&mut self.demo, "Demo keyboard + mouse")
                        .changed()
                    {
                        self.activity.keys.clear();
                        self.activity.buttons.clear();
                    }
                    ui.add(egui::Slider::new(&mut self.opacity, 0.3..=1.0).text("Opacity"));
                    ui.checkbox(&mut self.activity.idle_phone, "Browse phone when idle");
                    ui.add(
                        egui::Slider::new(&mut self.activity.idle_seconds, 5.0..=120.0)
                            .text("Idle seconds"),
                    );
                    ui.horizontal(|ui| {
                        for (label, size) in [("Small", 300.0), ("Medium", 420.0), ("Large", 560.0)]
                        {
                            if ui.button(label).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                    Vec2::splat(size),
                                ));
                            }
                        }
                    });
                    ui.separator();
                    ui.small("Drag to move · Double-click / right-click for settings");
                    ui.small("F8: click-through · Key labels stay visible");
                    ui.small("Labels show physical keys, not IME-composed text.");
                    if ui.button("Hide settings").clicked() {
                        self.settings = false;
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            if !open {
                self.settings = false;
            }
        }
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--check-input") {
        let rx = input::listen(None);
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Ok(input::Input::Status(s)) = rx.recv_timeout(Duration::from_millis(100)) {
                println!("{s}");
                if s != "Keyboard + mouse connected" {
                    std::process::exit(1);
                }
                return Ok(());
            }
        }
        eprintln!("Input hook initialization timed out");
        std::process::exit(1);
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_title("BusyCat")
            .with_inner_size([420.0, 420.0])
            .with_min_inner_size([250.0, 250.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "BusyCat",
        options,
        Box::new(|cc| Ok(Box::new(Cat::new(cc)))),
    )
}
