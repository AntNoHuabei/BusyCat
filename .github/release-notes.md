BusyCat v0.1.6 · Windows 10/11 x64

### 简体中文
- 新增单实例限制：同一 Windows 登录会话只运行一个 BusyCat，重复启动直接退出；正常退出或意外终止后可重新启动。
- 托盘退出直接在原生菜单回调执行，保存尺寸后终止自身进程，不再等待窗口刷新或鼠标移到猫身上。
- 设置窗口和猫咪右键菜单退出同样保存尺寸并直接终止进程，跳过渲染器退出清理。
- 托盘其他操作显式请求主窗口刷新。

### English
- Allow only one BusyCat per Windows login session; duplicate launches exit immediately, and the instance guard is released when the process ends.
- Exit directly from the native tray menu callback after saving cat size, without waiting for a viewport redraw or hovering over the cat.
- Settings and pet context-menu exit also save size and terminate the process directly, bypassing renderer teardown.
- Explicitly repaint the root viewport for other tray actions.

### Downloads
- BusyCat.exe: standalone executable / 独立可执行文件
- BusyCat-v0.1.6-windows-x64.zip: EXE + README + MIT LICENSE + icon / 便携包
- SHA256SUMS.txt: SHA-256 checksums / 校验值

升级前请先完全退出旧版，再运行新版。
Fully exit the old version before launching this update.

MIT License. Cat size is persisted; session statistics reset on restart.
保存猫咪尺寸；键鼠统计在重启后清空。
