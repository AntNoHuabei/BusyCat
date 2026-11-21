use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone)]
pub enum Input {
    Key { vk: u32, down: bool },
    Button { button: u8, down: bool },
    Move { x: f32, y: f32 },
    Wheel { delta: i16, horizontal: bool },
    Status(String),
}

pub fn listen(ctx: Option<eframe::egui::Context>) -> Receiver<Input> {
    let (tx, rx) = mpsc::channel();
    #[cfg(target_os = "windows")]
    std::thread::spawn(move || platform::run(tx, ctx));
    #[cfg(not(target_os = "windows"))]
    let _ = tx.send(Input::Status(
        "Global input is available on Windows only".into(),
    ));
    rx
}

#[cfg(target_os = "windows")]
mod platform {
    use super::Input;
    use std::{
        ffi::c_void,
        sync::{OnceLock, mpsc::Sender},
    };
    static SENDER: OnceLock<(Sender<Input>, Option<eframe::egui::Context>)> = OnceLock::new();
    type Hook = *mut c_void;
    type Proc = unsafe extern "system" fn(i32, usize, isize) -> isize;
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct Msg {
        hwnd: *mut c_void,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt: Point,
        private: u32,
    }
    #[repr(C)]
    struct Keyboard {
        vk: u32,
        scan: u32,
        flags: u32,
        time: u32,
        extra: usize,
    }
    #[repr(C)]
    struct Mouse {
        pt: Point,
        data: u32,
        flags: u32,
        time: u32,
        extra: usize,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowsHookExW(
            id: i32,
            callback: Option<Proc>,
            module: *mut c_void,
            thread: u32,
        ) -> Hook;
        fn CallNextHookEx(hook: Hook, code: i32, wparam: usize, lparam: isize) -> isize;
        fn UnhookWindowsHookEx(hook: Hook) -> i32;
        fn GetMessageW(msg: *mut Msg, window: *mut c_void, min: u32, max: u32) -> i32;
        fn GetSystemMetrics(index: i32) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetLastError() -> u32;
    }
    fn emit(event: Input) {
        if let Some((tx, ctx)) = SENDER.get() {
            let _ = tx.send(event);
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        }
    }
    unsafe extern "system" fn keyboard(code: i32, w: usize, l: isize) -> isize {
        if code >= 0 && matches!(w, 0x100 | 0x101 | 0x104 | 0x105) {
            // The OS supplies KBDLLHOOKSTRUCT for this callback.
            let event = unsafe { &*(l as *const Keyboard) };
            if event.flags & 0x10 == 0 {
                emit(Input::Key {
                    vk: event.vk,
                    down: matches!(w, 0x100 | 0x104),
                });
            }
        }
        // Always pass through; never suppress input to other applications.
        unsafe { CallNextHookEx(std::ptr::null_mut(), code, w, l) }
    }
    unsafe extern "system" fn mouse(code: i32, w: usize, l: isize) -> isize {
        if code >= 0 {
            // The OS supplies MSLLHOOKSTRUCT for this callback.
            let event = unsafe { &*(l as *const Mouse) };
            if event.flags & 1 == 0 {
                match w {
                    0x200 => LAST_MOVE.with(|last| {
                        if event.time.wrapping_sub(last.get()) >= 16 {
                            last.set(event.time);
                            let (ox, oy, width, height) = unsafe {
                                (
                                    GetSystemMetrics(76),
                                    GetSystemMetrics(77),
                                    GetSystemMetrics(78),
                                    GetSystemMetrics(79),
                                )
                            };
                            emit(Input::Move {
                                x: ((event.pt.x - ox) as f32 / width.max(1) as f32).clamp(0.0, 1.0),
                                y: ((event.pt.y - oy) as f32 / height.max(1) as f32)
                                    .clamp(0.0, 1.0),
                            });
                        }
                    }),
                    0x201 | 0x202 => emit(Input::Button {
                        button: 0,
                        down: w == 0x201,
                    }),
                    0x204 | 0x205 => emit(Input::Button {
                        button: 1,
                        down: w == 0x204,
                    }),
                    0x207 | 0x208 => emit(Input::Button {
                        button: 2,
                        down: w == 0x207,
                    }),
                    0x20b | 0x20c => emit(Input::Button {
                        button: 2 + (event.data >> 16) as u8,
                        down: w == 0x20b,
                    }),
                    0x20a | 0x20e => emit(Input::Wheel {
                        delta: (event.data >> 16) as i16,
                        horizontal: w == 0x20e,
                    }),
                    _ => {}
                }
            }
        }
        unsafe { CallNextHookEx(std::ptr::null_mut(), code, w, l) }
    }
    thread_local! { static LAST_MOVE: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
    pub fn run(tx: Sender<Input>, ctx: Option<eframe::egui::Context>) {
        let _ = SENDER.set((tx, ctx));
        unsafe {
            let module = GetModuleHandleW(std::ptr::null());
            let keyboard = SetWindowsHookExW(13, Some(keyboard), module, 0);
            let key_error = if keyboard.is_null() {
                GetLastError()
            } else {
                0
            };
            let mouse = SetWindowsHookExW(14, Some(mouse), module, 0);
            let mouse_error = if mouse.is_null() { GetLastError() } else { 0 };
            emit(Input::Status(if !keyboard.is_null() && !mouse.is_null() {
                "Keyboard + mouse connected".into()
            } else {
                format!("Hook errors: keyboard={key_error}, mouse={mouse_error}")
            }));
            let mut msg: Msg = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
            if !keyboard.is_null() {
                UnhookWindowsHookEx(keyboard);
            }
            if !mouse.is_null() {
                UnhookWindowsHookEx(mouse);
            }
        }
    }
}
