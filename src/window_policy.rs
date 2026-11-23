//! Keep the pet visible after Explorer's Show Desktop, without taking keyboard focus.
#[cfg(windows)]
mod native {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::sync::atomic::{AtomicBool, Ordering};

    static VISIBLE: AtomicBool = AtomicBool::new(true);
    static WINDOW: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
    const TIMER: usize = 0xBC01;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetTimer(
            hwnd: isize,
            id: usize,
            ms: u32,
            callback: unsafe extern "system" fn(isize, u32, usize, u32),
        ) -> usize;
        fn IsIconic(hwnd: isize) -> i32;
        fn KillTimer(hwnd: isize, id: usize) -> i32;
        fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn ShowWindow(hwnd: isize, command: i32) -> i32;
        fn SetWindowPos(
            hwnd: isize,
            after: isize,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            flags: u32,
        ) -> i32;
        fn GetForegroundWindow() -> isize;
        fn GetClassNameW(hwnd: isize, name: *mut u16, capacity: i32) -> i32;
        fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
    }

    unsafe extern "system" fn keep_visible(hwnd: isize, _: u32, _: usize, _: u32) {
        if !VISIBLE.load(Ordering::Relaxed) {
            return;
        }
        // A native UI-thread timer also runs while winit suspends minimized redraws.
        unsafe {
            // winit can reapply APPWINDOW after creation or viewport changes.
            let style = GetWindowLongPtrW(hwnd, -20);
            let desired = (style | 0x80) & !0x40000;
            if style != desired {
                SetWindowLongPtrW(hwnd, -20, desired);
            }
            let mut name = [0u16; 128];
            let len = GetClassNameW(GetForegroundWindow(), name.as_mut_ptr(), name.len() as i32);
            let desktop = matches!(
                String::from_utf16_lossy(&name[..len.max(0) as usize]).as_str(),
                "Progman" | "WorkerW"
            );
            if IsIconic(hwnd) != 0 || IsWindowVisible(hwnd) == 0 || desktop {
                if IsIconic(hwnd) != 0 {
                    ShowWindow(hwnd, 4); // SW_SHOWNOACTIVATE: restore without stealing focus.
                }
                SetWindowPos(hwnd, -1, 0, 0, 0, 0, 0x0013 | 0x0040);
            }
        }
    }

    pub fn install(cc: &eframe::CreationContext<'_>) {
        if let Ok(handle) = cc.window_handle()
            && let RawWindowHandle::Win32(handle) = handle.as_raw()
        {
            let hwnd = handle.hwnd.get();
            WINDOW.store(hwnd, Ordering::Relaxed);
            unsafe {
                // Tool windows stay out of both the taskbar and Alt+Tab.
                let style = GetWindowLongPtrW(hwnd, -20);
                SetWindowLongPtrW(hwnd, -20, (style | 0x80) & !0x40000);
                SetTimer(hwnd, TIMER, 100, keep_visible);
            }
        }
    }
    pub fn set_visible(visible: bool) {
        VISIBLE.store(visible, Ordering::Relaxed);
    }
    pub fn shutdown() {
        VISIBLE.store(false, Ordering::Relaxed);
        let hwnd = WINDOW.swap(0, Ordering::Relaxed);
        if hwnd != 0 {
            unsafe {
                KillTimer(hwnd, TIMER);
                ShowWindow(hwnd, 0); // Hide immediately, before renderer teardown.
                PostMessageW(hwnd, 0x10, 0, 0); // WM_CLOSE also works without a redraw.
            }
        }
    }
}
#[cfg(windows)]
pub use native::{install, set_visible, shutdown};
#[cfg(not(windows))]
pub fn install(_: &eframe::CreationContext<'_>) {}
#[cfg(not(windows))]
pub fn set_visible(_: bool) {}
#[cfg(not(windows))]
pub fn shutdown() {}

pub fn force_exit() -> ! {
    #[cfg(windows)]
    unsafe {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn TerminateProcess(process: *mut std::ffi::c_void, code: u32) -> i32;
        }
        // Terminating ourselves bypasses DLL detach and renderer destruction.
        TerminateProcess(GetCurrentProcess(), 0);
    }
    std::process::exit(0)
}
