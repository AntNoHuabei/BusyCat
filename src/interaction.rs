use crate::input::Input;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

pub struct Bubble {
    pub text: String,
    pub start: Instant,
    pub slot: usize,
    pub mouse: bool,
}
pub struct Interaction {
    pub keys: HashSet<u32>,
    pub buttons: HashSet<u8>,
    pub bubbles: Vec<Bubble>,
    pub cursor: [f32; 2],
    pub left: Option<Instant>,
    pub right: Option<Instant>,
    pub mouse_at: Option<Instant>,
    pub keyboard_at: Option<Instant>,
    pub wheel_at: Option<(Instant, i16)>,
    pub mouse_press: Option<(u8, Instant)>,
    pub button_down_at: [Option<Instant>; 5],
    pub button_up_at: [Option<Instant>; 5],
    pub last_key: Option<(u32, Instant)>,
    key_times: [Option<Instant>; 256],
    pub labels: bool,
    pub last_activity: Instant,
    pub idle_seconds: f32,
    pub idle_phone: bool,
    next_slot: usize,
}
impl Default for Interaction {
    fn default() -> Self {
        Self {
            keys: HashSet::new(),
            buttons: HashSet::new(),
            bubbles: vec![],
            cursor: [0.5, 0.5],
            left: None,
            right: None,
            mouse_at: None,
            keyboard_at: None,
            wheel_at: None,
            mouse_press: None,
            button_down_at: [None; 5],
            button_up_at: [None; 5],
            last_key: None,
            key_times: [None; 256],
            labels: true,
            last_activity: Instant::now(),
            idle_seconds: 15.0,
            idle_phone: true,
            next_slot: 0,
        }
    }
}
impl Interaction {
    fn bubble(&mut self, text: String, now: Instant, mouse: bool) {
        if !self.labels {
            return;
        }
        if self.bubbles.len() >= 18 {
            self.bubbles.remove(0);
        }
        self.bubbles.push(Bubble {
            text,
            start: now,
            slot: self.next_slot % 7,
            mouse,
        });
        self.next_slot = self.next_slot.wrapping_add(1);
    }
    pub fn handle(&mut self, event: &Input, now: Instant) -> bool {
        if !matches!(event, Input::Status(_)) {
            self.last_activity = now;
        }
        match *event {
            Input::Key { vk, down } => {
                if !down {
                    self.keys.remove(&vk);
                    return false;
                }
                let first = self.keys.insert(vk);
                if vk == 0x77 {
                    return first;
                }
                self.keyboard_at = Some(now);
                self.last_key = Some((vk, now));
                if let Some(t) = self.key_times.get_mut(vk as usize) {
                    *t = Some(now);
                }
                if is_left(vk) {
                    self.left = Some(now);
                } else {
                    self.right = Some(now);
                }
                let mut parts = Vec::new();
                for (group, label) in [
                    (&[0x11, 0xa2, 0xa3][..], "Ctrl"),
                    (&[0x12, 0xa4, 0xa5][..], "Alt"),
                    (&[0x10, 0xa0, 0xa1][..], "Shift"),
                    (&[0x5b, 0x5c][..], "Win"),
                ] {
                    if !group.contains(&vk) && group.iter().any(|k| self.keys.contains(k)) {
                        parts.push(label.to_owned());
                    }
                }
                parts.push(key_label(vk));
                self.bubble(parts.join(" + "), now, false);
            }
            Input::Button { button, down } => {
                self.mouse_at = Some(now);
                if down {
                    self.buttons.insert(button);
                    if let Some(t) = self.button_down_at.get_mut(button as usize) {
                        *t = Some(now);
                    }
                    if let Some(t) = self.button_up_at.get_mut(button as usize) {
                        *t = None;
                    }
                    self.mouse_press = Some((button, now));
                    self.bubble(
                        match button {
                            0 => "LMB",
                            1 => "RMB",
                            2 => "MMB",
                            3 => "Mouse 4",
                            _ => "Mouse 5",
                        }
                        .into(),
                        now,
                        true,
                    );
                } else {
                    self.buttons.remove(&button);
                    if let Some(t) = self.button_up_at.get_mut(button as usize) {
                        *t = Some(now);
                    }
                }
            }
            Input::Move { x, y } => {
                self.cursor = [x, y];
                self.mouse_at = Some(now);
            }
            Input::Wheel { delta, horizontal } => {
                self.mouse_at = Some(now);
                self.wheel_at = Some((now, delta));
                let direction = match (horizontal, delta > 0) {
                    (false, true) => "Up",
                    (false, false) => "Down",
                    (true, true) => "Right",
                    (true, false) => "Left",
                };
                self.bubble(format!("Wheel {direction}"), now, true);
            }
            Input::Status(_) => {}
        }
        false
    }
    // One paw can touch one key at a time. Prefer the newest held key on each side;
    // when it is released, return to another held key instead of losing the chord.
    pub fn paw_key(&self, viewer_left: bool, now: Instant) -> Option<([f32; 2], f32)> {
        let candidate = |held_only: bool| {
            self.key_times
                .iter()
                .enumerate()
                .filter_map(|(vk, t)| t.map(|t| (vk as u32, t)))
                .filter(|&(vk, t)| {
                    vk != 0x77
                        && is_left(vk) == viewer_left
                        && key_rect(vk).is_some()
                        && if held_only {
                            self.keys.contains(&vk)
                        } else {
                            now.saturating_duration_since(t) < Duration::from_millis(210)
                        }
                })
                .max_by_key(|&(vk, t)| (t, vk))
        };
        let (vk, t) = candidate(true).or_else(|| candidate(false))?;
        let [x, y, w, h] = key_rect(vk)?;
        let age = now.saturating_duration_since(t).as_secs_f32();
        let depth = if self.keys.contains(&vk) {
            (age / 0.03).clamp(0.0, 1.0)
        } else {
            (age / 0.03).min((0.21 - age) / 0.07).clamp(0.0, 1.0)
        };
        Some(([x + w * 0.5, y + h * 0.5], depth))
    }
    pub fn active_key(&self, vk: u32, now: Instant) -> bool {
        self.keys.contains(&vk)
            || self.last_key.is_some_and(|(key, t)| {
                key == vk && now.duration_since(t) < Duration::from_millis(180)
            })
    }
    pub fn mouse_pose(&self, now: Instant) -> bool {
        let mouse = self
            .mouse_at
            .is_some_and(|t| now.duration_since(t) < Duration::from_millis(650));
        let keyboard = self
            .keyboard_at
            .is_some_and(|t| now.duration_since(t) < Duration::from_millis(220));
        let held = self.keys.iter().any(|&vk| vk != 0x77);
        !keyboard && !held && (mouse || !self.buttons.is_empty())
    }
    // Preserve very short clicks, then ease each button back independently.
    pub fn button_depth(&self, button: u8, now: Instant) -> f32 {
        let Some(Some(down)) = self.button_down_at.get(button as usize) else {
            return 0.0;
        };
        let smooth = |v: f32| {
            let x = v.clamp(0.0, 1.0);
            x * x * (3.0 - 2.0 * x)
        };
        let attack = smooth(now.saturating_duration_since(*down).as_secs_f32() / 0.035);
        let release =
            self.button_up_at[button as usize].map(|up| up.max(*down + Duration::from_millis(70)));
        attack
            * release.map_or(1.0, |up| {
                1.0 - smooth(now.saturating_duration_since(up).as_secs_f32() / 0.12)
            })
    }
    pub fn frame(&self, now: Instant, elapsed: u128) -> usize {
        if self.idle_phone
            && self.keys.is_empty()
            && self.buttons.is_empty()
            && now
                .saturating_duration_since(self.last_activity)
                .as_secs_f32()
                >= self.idle_seconds
        {
            let ms = (now
                .saturating_duration_since(self.last_activity)
                .as_millis()
                .saturating_sub((self.idle_seconds * 1000.0) as u128))
                % 3000;
            return match ms {
                700..=1050 | 2200..=2450 => 13,
                1600..=1750 => 14,
                _ => 12,
            };
        }
        if self.mouse_pose(now) {
            return if !self.buttons.is_empty()
                || self
                    .mouse_press
                    .is_some_and(|(_, t)| now.duration_since(t) < Duration::from_millis(180))
                || self
                    .wheel_at
                    .is_some_and(|(t, _)| now.duration_since(t) < Duration::from_millis(170))
            {
                11
            } else {
                10
            };
        }
        let phase = |start: Option<Instant>| {
            start
                .map(|t| now.duration_since(t).as_millis())
                .filter(|&ms| ms < 180)
        };
        let l = phase(self.left);
        let r = phase(self.right);
        let lh = self.keys.iter().any(|&k| k != 0x77 && is_left(k));
        let rh = self.keys.iter().any(|&k| k != 0x77 && !is_left(k));
        if l.is_some() || r.is_some() || lh || rh {
            let age = l.into_iter().chain(r).min().unwrap_or(1000);
            let down = lh || rh || (30..120).contains(&age);
            return match (l.is_some() || lh, r.is_some() || rh, down) {
                (true, true, false) => 8,
                (true, true, true) => 9,
                (true, false, false) => 4,
                (true, false, true) => 5,
                (_, true, false) => 6,
                (_, true, true) => 7,
                _ => 0,
            };
        }
        match elapsed % 5400 {
            4100..=4154 | 4250..=4304 => 2,
            4155..=4249 => 3,
            2400..=2850 => 1,
            _ => 0,
        }
    }
}

pub fn mouse_offset(cursor: [f32; 2]) -> [f32; 2] {
    [
        (0.5 - cursor[0].clamp(0.0, 1.0)) * 36.0,
        (0.5 - cursor[1].clamp(0.0, 1.0)) * 24.0,
    ]
}

pub fn is_left(vk: u32) -> bool {
    let cat_left_hand = match vk {
        0x41..=0x5a => "QWERTASDFGZXCVB".contains(char::from_u32(vk).unwrap_or(' ')),
        0x30..=0x39 => (0x31..=0x35).contains(&vk),
        0x09 | 0x14 | 0x1b | 0xa0 | 0xa2 | 0xa4 | 0x5b | 0xc0 => true,
        _ => false,
    };
    // Facing the cat: its left hand is on the viewer's right.
    !cat_left_hand
}
pub fn key_label(vk: u32) -> String {
    match vk {
        0x30..=0x39 | 0x41..=0x5a => return char::from_u32(vk).unwrap_or('?').to_string(),
        0x70..=0x87 => return format!("F{}", vk - 0x6f),
        0x60..=0x69 => return format!("Num {}", vk - 0x60),
        0x08 => "Backspace",
        0x09 => "Tab",
        0x0d => "Enter",
        0x10 | 0xa0 | 0xa1 => "Shift",
        0x11 | 0xa2 | 0xa3 => "Ctrl",
        0x12 | 0xa4 | 0xa5 => "Alt",
        0x13 => "Pause",
        0x14 => "Caps Lock",
        0x1b => "Esc",
        0x20 => "Space",
        0x21 => "Page Up",
        0x22 => "Page Down",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "Left",
        0x26 => "Up",
        0x27 => "Right",
        0x28 => "Down",
        0x2c => "Print Screen",
        0x2d => "Insert",
        0x2e => "Delete",
        0x5b | 0x5c => "Win",
        0x5d => "Menu",
        0x6a => "Num *",
        0x6b => "Num +",
        0x6d => "Num -",
        0x6e => "Num .",
        0x6f => "Num /",
        0x90 => "Num Lock",
        0x91 => "Scroll Lock",
        0xad => "Mute",
        0xae => "Volume -",
        0xaf => "Volume +",
        0xb0 => "Next",
        0xb1 => "Previous",
        0xb2 => "Stop",
        0xb3 => "Play/Pause",
        0xba => ";",
        0xbb => "=",
        0xbc => ",",
        0xbd => "-",
        0xbe => ".",
        0xbf => "/",
        0xc0 => "`",
        0xdb => "[",
        0xdc => "\\",
        0xdd => "]",
        0xde => "'",
        _ => return format!("Key {vk:02X}"),
    }
    .into()
}

// The exact same rectangles generate SVG keycaps and runtime highlights.
pub fn key_rect(vk: u32) -> Option<[f32; 4]> {
    let vk = match vk {
        0x10 => 0xa0,
        0x11 => 0xa2,
        0x12 => 0xa4,
        _ => vk,
    };
    crate::keyboard_layout::KEYS
        .iter()
        .find(|(key, _)| *key == vk)
        .map(|(_, rect)| *rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paws_touch_actual_keys_and_preserve_chords() {
        let t = Instant::now();
        let mut s = Interaction::default();
        for vk in [0x41, 0x4c, 0x31, 0xa2, 0x20] {
            s.handle(&Input::Key { vk, down: true }, t);
            let (point, depth) = s
                .paw_key(is_left(vk), t + Duration::from_millis(60))
                .unwrap();
            let [x, y, w, h] = key_rect(vk).unwrap();
            assert_eq!(point, [x + w / 2.0, y + h / 2.0]);
            assert_eq!(depth, 1.0);
            s.handle(&Input::Key { vk, down: false }, t);
        }
        assert!(s.paw_key(true, t + Duration::from_secs(1)).is_none());
        s.handle(
            &Input::Key {
                vk: 0xa2,
                down: true,
            },
            t,
        );
        s.handle(
            &Input::Key {
                vk: 0x53,
                down: true,
            },
            t + Duration::from_millis(10),
        );
        s.handle(
            &Input::Key {
                vk: 0x53,
                down: false,
            },
            t + Duration::from_millis(20),
        );
        let [x, y, w, h] = key_rect(0xa2).unwrap();
        assert_eq!(
            s.paw_key(false, t + Duration::from_secs(1)).unwrap().0,
            [x + w / 2.0, y + h / 2.0]
        );
    }
    #[test]
    fn mouse_buttons_depress_and_release_independently() {
        let mut s = Interaction::default();
        let t = Instant::now();
        s.handle(
            &Input::Button {
                button: 0,
                down: true,
            },
            t,
        );
        s.handle(
            &Input::Button {
                button: 0,
                down: false,
            },
            t,
        );
        s.handle(
            &Input::Button {
                button: 1,
                down: true,
            },
            t,
        );
        let peak = t + Duration::from_millis(70);
        assert_eq!(s.button_depth(0, peak), 1.0);
        assert_eq!(s.button_depth(1, peak), 1.0);
        assert!((0.0..1.0).contains(&s.button_depth(0, t + Duration::from_millis(130))));
        assert_eq!(s.button_depth(0, t + Duration::from_millis(200)), 0.0);
        assert_eq!(s.button_depth(1, t + Duration::from_secs(2)), 1.0);
        s.handle(
            &Input::Button {
                button: 1,
                down: false,
            },
            t + Duration::from_secs(2),
        );
        assert_eq!(s.button_depth(1, t + Duration::from_millis(2200)), 0.0);
        assert_eq!(s.button_depth(255, peak), 0.0);
    }
    #[test]
    fn phone_waits_for_idle_and_any_input_wakes_cat() {
        let t = Instant::now();
        let mut s = Interaction {
            last_activity: t,
            ..Interaction::default()
        };
        assert!(s.frame(t + Duration::from_secs(14), 0) < 12);
        assert_eq!(s.frame(t + Duration::from_secs(15), 0), 12);
        assert_eq!(s.frame(t + Duration::from_millis(15800), 0), 13);
        s.handle(&Input::Move { x: 0.9, y: 0.5 }, t + Duration::from_secs(16));
        assert_eq!(s.frame(t + Duration::from_secs(16), 0), 10);
        s.handle(
            &Input::Key {
                vk: 0x41,
                down: true,
            },
            t + Duration::from_secs(17),
        );
        assert!(
            s.frame(t + Duration::from_secs(60), 0) < 12,
            "Held keys prevent phone mode"
        );
        s.handle(
            &Input::Key {
                vk: 0x41,
                down: false,
            },
            t + Duration::from_secs(61),
        );
        assert!(s.frame(t + Duration::from_secs(70), 0) < 12);
        assert_eq!(s.frame(t + Duration::from_secs(76), 0), 12);
    }
    #[test]
    fn mouse_travel_is_bounded_and_opposes_both_axes() {
        assert_eq!(mouse_offset([0.5, 0.5]), [0.0, 0.0]);
        assert_eq!(mouse_offset([-5.0, 4.0]), [18.0, -12.0]);
        assert_eq!(mouse_offset([1.0, 0.0]), [-18.0, 12.0]);
    }
    #[test]
    fn keyboard_keycaps_never_overlap() {
        let keys = crate::keyboard_layout::KEYS;
        for (i, (_, a)) in keys.iter().enumerate() {
            for (_, b) in &keys[i + 1..] {
                let overlap = a[0] < b[0] + b[2]
                    && b[0] < a[0] + a[2]
                    && a[1] < b[1] + b[3]
                    && b[1] < a[1] + a[3];
                assert!(!overlap, "Overlapping keycaps {a:?}, {b:?}");
            }
        }
        assert_eq!(keys.iter().filter(|(vk, _)| *vk == 0x20).count(), 1);
    }
    #[test]
    fn keyboard_faces_cat_not_viewer() {
        let a = key_rect(0x41).unwrap();
        let l = key_rect(0x4c).unwrap();
        let space = key_rect(0x20).unwrap();
        let number = key_rect(0x31).unwrap();
        assert!(a[0] > l[0], "A must be on viewer right");
        assert!(space[1] < number[1], "Spacebar must be nearest the cat");
        assert!(!is_left(0x41));
        assert!(is_left(0x4c));
    }
    #[test]
    fn chord_and_release_keep_other_keys_held() {
        let mut s = Interaction::default();
        let t = Instant::now();
        s.handle(
            &Input::Key {
                vk: 0xa2,
                down: true,
            },
            t,
        );
        s.handle(
            &Input::Key {
                vk: 0x53,
                down: true,
            },
            t,
        );
        assert_eq!(s.bubbles.last().unwrap().text, "Ctrl + S");
        s.handle(
            &Input::Key {
                vk: 0x53,
                down: false,
            },
            t,
        );
        assert!(s.keys.contains(&0xa2));
        assert!(!s.keys.contains(&0x53));
        assert_eq!(s.frame(t + Duration::from_secs(2), 0), 7);
    }
    #[test]
    fn shortcut_does_not_toggle_on_repeat_or_release() {
        let mut s = Interaction::default();
        let t = Instant::now();
        assert!(s.handle(
            &Input::Key {
                vk: 0x77,
                down: true
            },
            t
        ));
        assert!(!s.handle(
            &Input::Key {
                vk: 0x77,
                down: true
            },
            t
        ));
        assert!(!s.handle(
            &Input::Key {
                vk: 0x77,
                down: false
            },
            t
        ));
        assert!(s.handle(
            &Input::Key {
                vk: 0x77,
                down: true
            },
            t
        ));
    }
    #[test]
    fn mouse_has_separate_pose_and_wheel_feedback() {
        let mut s = Interaction::default();
        let t = Instant::now();
        s.handle(&Input::Move { x: 0.2, y: 0.8 }, t);
        assert_eq!(s.frame(t, 0), 10);
        s.handle(
            &Input::Button {
                button: 1,
                down: true,
            },
            t,
        );
        assert_eq!(s.frame(t, 0), 11);
        assert_eq!(s.bubbles.last().unwrap().text, "RMB");
        s.handle(
            &Input::Button {
                button: 1,
                down: false,
            },
            t,
        );
        // A quick click remains visible even if down/up arrive in one UI frame.
        assert_eq!(s.frame(t, 0), 11);
        assert_eq!(s.frame(t + Duration::from_millis(250), 0), 10);
        s.handle(
            &Input::Wheel {
                delta: -120,
                horizontal: false,
            },
            t,
        );
        assert_eq!(s.bubbles.last().unwrap().text, "Wheel Down");
        assert_eq!(s.frame(t, 0), 11);
    }
    #[test]
    fn typing_overrides_mouse_and_rapid_input_is_bounded() {
        let mut s = Interaction::default();
        let t = Instant::now();
        for _ in 0..100 {
            s.handle(
                &Input::Key {
                    vk: 0x41,
                    down: true,
                },
                t,
            );
        }
        s.handle(&Input::Move { x: 0.2, y: 0.8 }, t);
        assert_eq!(s.frame(t + Duration::from_millis(70), 0), 7);
        assert_eq!(s.bubbles.len(), 18);
        s.handle(
            &Input::Key {
                vk: 0x41,
                down: false,
            },
            t,
        );
        assert_eq!(s.frame(t + Duration::from_millis(250), 0), 10);
    }
}
