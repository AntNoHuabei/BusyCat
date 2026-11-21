use crate::settings_ui::Language;
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Settings,
    ToggleVisible,
    TogglePassthrough,
    ToggleLabels,
    TogglePhone,
    Quit,
}
#[cfg(windows)]
pub use platform::Tray;
#[cfg(windows)]
mod platform {
    use super::*;
    use std::sync::mpsc::{self, Receiver};
    use tray_icon::{
        TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };
    pub struct Tray {
        _icon: TrayIcon,
        events: Receiver<Action>,
        settings: MenuItem,
        visible: CheckMenuItem,
        passthrough: CheckMenuItem,
        labels: CheckMenuItem,
        phone: CheckMenuItem,
        quit: MenuItem,
        state: Option<(Language, bool, bool, bool, bool)>,
    }
    impl Tray {
        pub fn new(ctx: eframe::egui::Context) -> Result<Self, String> {
            let menu = Menu::new();
            let settings = MenuItem::new("设置", true, None);
            let visible = CheckMenuItem::new("显示猫咪", true, true, None);
            let passthrough = CheckMenuItem::new("鼠标穿透（F8）", true, false, None);
            let labels = CheckMenuItem::new("显示按键气泡", true, true, None);
            let phone = CheckMenuItem::new("空闲时刷手机", true, true, None);
            let quit = MenuItem::new("退出 BusyCat", true, None);
            menu.append_items(&[
                &settings,
                &visible,
                &PredefinedMenuItem::separator(),
                &passthrough,
                &labels,
                &phone,
                &PredefinedMenuItem::separator(),
                &quit,
            ])
            .map_err(|e| e.to_string())?;
            let image = image::load_from_memory(include_bytes!("../assets/branding/tray.png"))
                .map_err(|e| e.to_string())?
                .to_rgba8();
            let icon =
                tray_icon::Icon::from_rgba(image.into_raw(), 32, 32).map_err(|e| e.to_string())?;
            let icon = TrayIconBuilder::new()
                .with_tooltip("BusyCat · 双击打开设置 / Double-click for settings")
                .with_icon(icon)
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .build()
                .map_err(|e| e.to_string())?;
            let (tx, events) = mpsc::channel();
            let actions = [
                (settings.id().clone(), Action::Settings),
                (visible.id().clone(), Action::ToggleVisible),
                (passthrough.id().clone(), Action::TogglePassthrough),
                (labels.id().clone(), Action::ToggleLabels),
                (phone.id().clone(), Action::TogglePhone),
                (quit.id().clone(), Action::Quit),
            ];
            let wake = ctx.clone();
            let menu_tx = tx.clone();
            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                if let Some((_, action)) = actions.iter().find(|(id, _)| *id == event.id) {
                    let _ = menu_tx.send(*action);
                    wake.request_repaint();
                }
            }));
            TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
                if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                    let _ = tx.send(Action::Settings);
                    ctx.request_repaint();
                }
            }));
            Ok(Self {
                _icon: icon,
                events,
                settings,
                visible,
                passthrough,
                labels,
                phone,
                quit,
                state: None,
            })
        }
        pub fn actions(&self) -> Vec<Action> {
            self.events.try_iter().collect()
        }
        pub fn sync(
            &mut self,
            lang: Language,
            visible: bool,
            passthrough: bool,
            labels: bool,
            phone: bool,
        ) {
            let state = (lang, visible, passthrough, labels, phone);
            if self.state == Some(state) {
                return;
            }
            self.state = Some(state);
            self.settings.set_text(lang.text("设置…", "Settings…"));
            self.visible.set_text(lang.text("显示猫咪", "Show cat"));
            self.visible.set_checked(visible);
            self.passthrough
                .set_text(lang.text("鼠标穿透（F8）", "Click-through (F8)"));
            self.passthrough.set_checked(passthrough);
            self.labels
                .set_text(lang.text("显示按键气泡", "Show key labels"));
            self.labels.set_checked(labels);
            self.phone
                .set_text(lang.text("空闲时刷手机", "Browse phone when idle"));
            self.phone.set_checked(phone);
            self.quit
                .set_text(lang.text("退出 BusyCat", "Quit BusyCat"));
        }
    }
    impl Drop for Tray {
        fn drop(&mut self) {
            MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
            TrayIconEvent::set_event_handler(None::<fn(TrayIconEvent)>);
        }
    }
}
#[cfg(not(windows))]
pub struct Tray;
#[cfg(not(windows))]
impl Tray {
    pub fn new(_: eframe::egui::Context) -> Result<Self, String> {
        Err("Tray requires Windows".into())
    }
    pub fn actions(&self) -> Vec<Action> {
        vec![]
    }
    pub fn sync(&mut self, _: Language, _: bool, _: bool, _: bool, _: bool) {}
}
