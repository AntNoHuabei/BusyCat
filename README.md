# BusyCat

原创黑猫桌宠原型。Rust + egui/eframe + OpenGL，不依赖 Vulkan、WebView、服务器或图片 API。

## 运行

Windows 10/11，需要 Rust 稳定版及 Visual Studio C++ Build Tools。

```powershell
cd C:\code\BusyCat
cargo run --release
```

首次构建需要下载 crates.io 依赖。正式程序位于 `target/release/busycat.exe`，素材已嵌入，可单独复制 exe。

## 操作

- 拖动猫咪：移动窗口。
- 双击或右键：打开设置。
- F8：切换鼠标穿透；穿透后再次按 F8 恢复操作。
- 在其他程序中打字：猫咪拍键盘，对应键位高亮，上浮显示按下的键；按住时维持动作，松开后复位。
- 支持 Ctrl / Alt / Shift / Win 组合键、功能键、方向键、标点和数字小键盘。显示物理按键名称，不是输入法上屏文字。
- 移动鼠标：画面左侧的鼠标平滑跟随光标，在素材坐标中水平 ±18、垂直 ±12 像素内移动（随窗口缩放）；猫爪末端和点击高亮同步移动，肩膀固定。鼠标左/右/中键、侧键和滚轮分别显示反馈。
- 无操作 15 秒后：猫咪拿出手机，循环滑动、眨眼；任意键鼠操作立即恢复。按住键盘或鼠标时不进入手机状态。
- 设置面板：可关闭闲置刷手机，或将等待时间调整为 5–120 秒；也可调整大小、透明度、鼠标反馈及按键提示。
- 右键菜单或设置面板：退出。

按键气泡默认开启，穿透模式下也会显示，可在设置或右键菜单关闭。输入不写入文件、不联网，内存中的气泡会自动过期。F8 是固定的穿透快捷键，长按不会重复切换。

## 素材

`assets/cat/` 中有 15 组原创 SVG + 透明 PNG（512 × 512），固定锚点 `(256, 450)`，无需逐帧重新定位。鼠标固定在画面左侧。键盘与鼠标均朝向猫：空格靠近猫身，键帽文字面向猫，A 在画面右边，L 在画面左边。爪子和键位高亮按此朝向映射。

| 文件前缀 | 动作 |
| --- | --- |
| idle / idle_tail | 待机 / 摆尾 |
| blink_half / blink_closed | 半闭眼 / 闭眼 |
| left_mid / left_down | 画面左爪中间帧 / 拍下 |
| right_mid / right_down | 画面右爪中间帧 / 拍下 |
| both_mid / both_down | 双爪中间帧 / 拍下 |
| mouse_move / mouse_click | 扶鼠标 / 点击鼠标（运行时手臂动态绘制） |
| phone_hold / phone_swipe / phone_blink | 拿手机 / 刷手机 / 看手机时眨眼 |

- `atlas.png`：6 列 × 3 行，3072 × 1536，按 `animations.json` 中的坐标排列。
- `mouse.png`：独立透明鼠标图层。运行时图集不包含鼠标，防止移动后出现重影；单张动作 PNG / SVG 为完整展示图。
- `keyboard-layout.json` / `src/keyboard_layout.rs`：同一份五排键盘几何数据生成的布局与高亮坐标，空格独占底排，不叠加小键。
- `animations.json`：帧坐标、锚点及建议时长（毫秒）。
- `preview/contact-sheet.png`：动作总览。
- `preview/busycat.gif`：动画演示，米色仅为预览底色。
- `preview/index.html`：不需要构建即可打开的交互演示。

素材由本项目中的几何路径代码绘制，不是 AI 生成，也不是从视频截取。修改 `tools/generate_assets.py` 可以统一调整颜色、造型、动作。

```powershell
python -m pip install -r tools/requirements.txt
python tools/generate_assets.py
```

## 原型边界

全局键鼠监听目前仅实现 Windows；设置暂不持久化，没有托盘和开机启动。透明窗口的最终显示效果取决于本机显卡驱动和桌面合成环境。OpenGL 后端通过 `eframe::Renderer::Glow` 显式选择。

监听线程只投递事件并唤醒 UI；鼠标移动事件节流至最多约每 16 毫秒一次，动画定时刷新约 30 FPS，尚未做到完全的空闲休眠。动画是固定姿态序列帧，瞳孔跟随和按键高亮为实时绘制。常用键位高亮对应五排 60% 键盘布局，无独立数字小键盘；方向键与功能键仍显示气泡。程序不监听 Windows 安全桌面，也忽略系统标记的注入输入。

## 验证

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
.\target\release\busycat.exe --check-input
```

`--check-input` 不打开界面，检查本机键盘与鼠标钩子能否注册。`--demo` 启动键鼠混合动画演示。8 项单元测试覆盖组合键释放、长按 F8、快速输入上限、鼠标点击/滚轮、鼠标移动边界、键帽不重叠、键盘朝向、闲置刷手机及输入恢复。

Typing paws now target the center of the actual keycap using the shared keyboard layout. Each paw selects its most recently held key, supports chords, and lifts after short taps. Git history is initialized locally; no remote is configured.

Paw silhouettes use compact rounded pads, four shallow toe lobes, and short subtle creases rather than long fingers or extended claws.

Settings open in a separate native window beside the cat. Menus support Simplified Chinese (default) and English; Windows supplies Microsoft YaHei for Chinese text. The size slider resizes the cat live from 250 to 800 px. Run with --settings to open settings immediately.

## Logo and system tray

The original BusyCat logo lives in `assets/branding/busycat.svg`; run `python tools/generate_logo.py` to rebuild PNG and multi-resolution ICO files. The EXE embeds the icon through `build.rs` and the window and system tray use matching artwork.

On Windows, right-click the system tray icon for Settings, Show cat, Click-through (F8), Show key labels, Browse phone when idle, and Quit. Double-click the tray icon to restore the cat and open settings. Menu labels follow the selected Chinese/English language and checkmarks follow the current settings. Quitting removes the tray icon; the tray library also restores it after Explorer restarts. If Windows places the icon in the overflow area, expand the tray arrow to find it.
