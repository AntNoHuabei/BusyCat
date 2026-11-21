use eframe::egui::{Pos2, Rect, Vec2};
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
}
impl Language {
    pub fn text(self, zh: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::Chinese => zh,
            Self::English => en,
        }
    }
}

pub fn position(cat: Rect, work: Rect, size: Vec2) -> Pos2 {
    let gap = 16.0;
    let candidates = [
        Pos2::new(cat.right() + gap, cat.top()),
        Pos2::new(cat.left() - size.x - gap, cat.top()),
        Pos2::new(cat.left(), cat.bottom() + gap),
        Pos2::new(cat.left(), cat.top() - size.y - gap),
    ];
    for p in candidates {
        let r = Rect::from_min_size(p, size);
        if work.contains_rect(r) {
            return p;
        }
    }
    let x = if cat.center().x < work.center().x {
        work.right() - size.x
    } else {
        work.left()
    };
    Pos2::new(
        x,
        cat.top()
            .clamp(work.top(), (work.bottom() - size.y).max(work.top())),
    )
}
// Keep the edge nearest the settings panel fixed so scaling never pushes into it.
pub fn resize_away(cat: Rect, panel: Pos2, panel_size: Vec2, size: f32) -> Rect {
    let panel = Rect::from_min_size(panel, panel_size);
    let mut origin = cat.min;
    if panel.left() >= cat.right() - 1.0 {
        origin.x = cat.right() - size;
    } else if panel.right() <= cat.left() + 1.0 {
        origin.x = cat.left();
    } else if panel.top() >= cat.bottom() - 1.0 {
        origin.y = cat.bottom() - size;
    }
    Rect::from_min_size(origin, Vec2::splat(size))
}
#[cfg(windows)]
pub fn work_area(cat: Rect, scale: f32) -> Rect {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[repr(C)]
    struct MonitorInfo {
        size: u32,
        monitor: WinRect,
        work: WinRect,
        flags: u32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MonitorFromRect(rect: *const WinRect, flags: u32) -> isize;
        fn GetMonitorInfoW(monitor: isize, info: *mut MonitorInfo) -> i32;
    }
    let r = WinRect {
        left: (cat.left() * scale) as i32,
        top: (cat.top() * scale) as i32,
        right: (cat.right() * scale) as i32,
        bottom: (cat.bottom() * scale) as i32,
    };
    let mut info = MonitorInfo {
        size: std::mem::size_of::<MonitorInfo>() as u32,
        monitor: r,
        work: r,
        flags: 0,
    };
    // Both structures have the Win32 ABI; MONITOR_DEFAULTTONEAREST chooses the cat's display.
    if unsafe { GetMonitorInfoW(MonitorFromRect(&r, 2), &mut info) } != 0 {
        return Rect::from_min_max(
            Pos2::new(info.work.left as f32 / scale, info.work.top as f32 / scale),
            Pos2::new(
                info.work.right as f32 / scale,
                info.work.bottom as f32 / scale,
            ),
        );
    }
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0))
}
#[cfg(not(windows))]
pub fn work_area(_: Rect, _: f32) -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resizing_keeps_panel_and_adjacent_edge_fixed() {
        let size = Vec2::new(376.0, 560.0);
        for panel in [Pos2::new(936.0, 200.0), Pos2::new(108.0, 200.0)] {
            let mut cat = Rect::from_min_size(Pos2::new(500.0, 200.0), Vec2::splat(420.0));
            for value in [800.0, 250.0, 600.0, 420.0] {
                let next = resize_away(cat, panel, size, value);
                assert!(!next.intersects(Rect::from_min_size(panel, size)));
                if panel.x > cat.right() {
                    assert_eq!(next.right(), cat.right());
                } else {
                    assert_eq!(next.left(), cat.left());
                }
                cat = next;
            }
        }
    }
    #[test]
    fn panel_uses_free_side() {
        let work = Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0));
        for x in [0.0, 1400.0] {
            let cat = Rect::from_min_size(Pos2::new(x, 200.0), Vec2::splat(420.0));
            let panel = Rect::from_min_size(
                position(cat, work, Vec2::new(360.0, 520.0)),
                Vec2::new(360.0, 520.0),
            );
            assert!(work.contains_rect(panel));
            assert!(!cat.intersects(panel));
        }
    }
}
