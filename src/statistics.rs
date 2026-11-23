use crate::{input::Input, settings_ui::Language};
use std::collections::HashSet;

#[derive(Default)]
pub struct Statistics {
    held: HashSet<u32>,
    keys: u64,
    clicks: [u64; 3],
    wheel: u64,
}
impl Statistics {
    pub fn record(&mut self, event: &Input) {
        match *event {
            Input::Key { vk, down: true } => {
                if self.held.insert(vk) {
                    self.keys += 1;
                }
            }
            Input::Key { vk, down: false } => {
                self.held.remove(&vk);
            }
            Input::Button { button, down: true } => self.clicks[usize::from(button).min(2)] += 1,
            Input::Wheel { delta, .. } => self.wheel += u64::from(delta.unsigned_abs()),
            _ => {}
        }
    }
    pub fn show(&mut self, ui: &mut eframe::egui::Ui, lang: Language) {
        ui.label(lang.text("本次运行统计", "Session statistics"));
        ui.label(format!(
            "{}: {}",
            lang.text("键盘按下", "Key presses"),
            self.keys
        ));
        ui.label(format!(
            "{}: {} / {} / {}",
            lang.text("鼠标 左 / 右 / 其他", "Mouse left / right / other"),
            self.clicks[0],
            self.clicks[1],
            self.clicks[2]
        ));
        ui.label(format!(
            "{}: {:.1}",
            lang.text("滚轮格数（含横向）", "Wheel notches (incl. horizontal)"),
            self.wheel as f64 / 120.0
        ));
        ui.small(lang.text(
            "仅本次运行，不保存按键内容；长按不重复计数。",
            "Session only; no typed text stored; key repeats excluded.",
        ));
        if ui
            .button(lang.text("清零统计", "Reset statistics"))
            .clicked()
        {
            self.keys = 0;
            self.clicks = [0; 3];
            self.wheel = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeats_releases_and_high_resolution_wheel() {
        let mut stats = Statistics::default();
        for down in [true, true, false, true] {
            stats.record(&Input::Key { vk: 65, down });
        }
        assert_eq!(stats.keys, 2);
        stats.record(&Input::Button {
            button: 4,
            down: true,
        });
        stats.record(&Input::Button {
            button: 4,
            down: false,
        });
        assert_eq!(stats.clicks, [0, 0, 1]);
        for delta in [30, -30, 60] {
            stats.record(&Input::Wheel {
                delta,
                horizontal: false,
            });
        }
        assert_eq!(stats.wheel, 120);
    }
}
