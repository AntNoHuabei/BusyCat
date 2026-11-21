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
