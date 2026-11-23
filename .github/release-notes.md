BusyCat v0.1.5 · Windows 10/11 x64

### 简体中文
- 最小尺寸降至 125 px，设置滑块支持 125–800 px。
- 退出时保存猫咪尺寸，下次启动自动恢复。
- 校准切换软件后的按键状态，避免残留状态阻止鼠标爪跟随；按住 Ctrl、Alt、Shift、Win 时仍可触发鼠标动作。
- 统一退出流程：先停用保活定时器并立即隐藏猫咪，再保存尺寸、停止监听和关闭窗口。

### English
- Reduce the minimum cat size to 125 px; the slider supports 125–800 px.
- Save cat size on exit and restore it at startup.
- Reconcile held keys after application switches to prevent stale states from blocking the mouse paw; modifier keys allow mouse reactions.
- Disable the visibility timer and hide the cat before saving size, stopping hooks, and closing the window.

### Downloads
- BusyCat.exe: standalone executable / 独立可执行文件
- BusyCat-v0.1.5-windows-x64.zip: EXE + README + MIT LICENSE + icon / 便携包
- SHA256SUMS.txt: SHA-256 checksums / 校验值

F8 toggles click-through. Double-click the cat or tray icon for settings.
F8 切换鼠标穿透；双击猫咪或托盘图标打开设置。

MIT License. Only cat size is persisted; session statistics reset on restart.
仅保存猫咪尺寸；键鼠统计在重启后清空。
